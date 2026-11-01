use std::{fs, path::Path, process::Command};

fn temp(name: &str) -> std::path::PathBuf {
    let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("hyperrealism-{name}-{}-{unique}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn resolution_toggles_seed_and_validation() {
    let root = temp("options");
    let input = root.join("input");
    fs::create_dir_all(&input).unwrap();
    for name in ["diffuse.png", "normal.png", "roughness.png"] {
        fs::copy(Path::new("test_assets").join(name), input.join(name)).unwrap();
    }
    fs::write(input.join("material_manifest.json"), r#"{"version":1,"materials":[{"id":"m","albedo":"diffuse.png","normal":"normal.png","roughness":"roughness.png"}],"textures":[{"name":"stone","material":"m","recipe":"stone","tint":[255,255,255]}]}"#).unwrap();
    let exe = env!("CARGO_BIN_EXE_hyperrealism-core");
    let run = |out: &Path, extra: &[&str]| {
        Command::new(exe).arg("--input").arg(&input).arg("--output").arg(out).arg("--cpu-only").args(extra).output().unwrap()
    };
    for (resolution, flags) in [("64", vec!["--no-normal"]), ("256", vec!["--no-specular"])] {
        let out = root.join(resolution);
        let result = run(&out, &["--resolution", resolution, "--seed", "7", "--name", "Option Test", "--threads", "1", flags[0]]);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let block = out.join("pack/assets/minecraft/textures/block");
        let albedo = hyperrealism_core::png_io::read(&block.join("stone.png")).unwrap();
        assert_eq!((albedo.width, albedo.height), (resolution.parse().unwrap(), resolution.parse().unwrap()));
        assert_eq!(block.join("stone_n.png").exists(), resolution != "64");
        assert_eq!(block.join("stone_s.png").exists(), resolution != "256");
        assert!(out.join(format!("dist/HyperRealism{resolution}-v1.0-MC26.3.zip")).is_file());
        let report: serde_json::Value = serde_json::from_slice(&fs::read(out.join("build_report.json")).unwrap()).unwrap();
        assert_eq!(report["resolution"], resolution.parse::<u64>().unwrap());
        assert_eq!(report["seed"], 7);
        assert_eq!(report["pack_name"], "Option Test");
    }
    let a = root.join("seed-a");
    let b = root.join("seed-b");
    assert!(run(&a, &["--resolution", "64", "--seed", "1"]).status.success());
    assert!(run(&b, &["--resolution", "64", "--seed", "2"]).status.success());
    assert_ne!(fs::read(a.join("pack/assets/minecraft/textures/block/stone.png")).unwrap(), fs::read(b.join("pack/assets/minecraft/textures/block/stone.png")).unwrap());
    for flags in [vec!["--resolution", "512"], vec!["--seed", "-1"], vec!["--threads", "0"], vec!["--unknown"]] {
        assert!(!run(&root.join("invalid"), &flags).status.success());
    }
    fs::remove_dir_all(root).unwrap();
}
