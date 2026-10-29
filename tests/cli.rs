use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn run(input: &Path, output: &Path, threads: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_hyperrealism-core"))
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(output)
        .arg("--cpu-only")
        .arg("--threads")
        .arg(threads)
        .output()
        .expect("run CLI")
}
#[test]
fn cpu_end_to_end_and_reproducibility() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp =
        std::env::temp_dir().join(format!("hyperrealism-test-{}-{unique}", std::process::id()));
    let input = temp.join("inputs");
    fs::create_dir_all(&input).unwrap();
    for name in ["diffuse.png", "normal.png", "roughness.png"] {
        fs::copy(Path::new("test_assets").join(name), input.join(name)).unwrap();
    }
    fs::write(input.join("material_manifest.json"),r#"{
      "version":1,"materials":[{"id":"test","albedo":"diffuse.png","normal":"normal.png","roughness":"roughness.png"}],
      "textures":[{"name":"iron_block","material":"test","recipe":"metal","tint":[255,255,255],"f0":230}]
    }"#).unwrap();
    let out1 = temp.join("first");
    let out2 = temp.join("second");
    for (out, threads) in [(&out1, "1"), (&out2, "2")] {
        let result = run(&input, out, threads);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let block = out1.join("pack/assets/minecraft/textures/block");
    let a = hyperrealism_core::png_io::read(&block.join("iron_block.png")).unwrap();
    let n = hyperrealism_core::png_io::read(&block.join("iron_block_n.png")).unwrap();
    let s = hyperrealism_core::png_io::read(&block.join("iron_block_s.png")).unwrap();
    assert_eq!(
        (a.width, a.height, n.width, n.height, s.width, s.height),
        (128, 128, 128, 128, 128, 128)
    );
    assert!(s
        .pixels
        .chunks_exact(4)
        .all(|p| p[1] == 230 && p[2] == 0 && p[3] == 255));
    assert!(out1.join("contact_sheet.png").is_file());
    let meta: serde_json::Value =
        serde_json::from_slice(&fs::read(out1.join("pack/pack.mcmeta")).unwrap()).unwrap();
    assert_eq!(meta["pack"]["min_format"], serde_json::json!([97, 1]));
    assert_eq!(meta["pack"]["max_format"], 97);
    let archive = "dist/HyperRealism128-v1.0-MC26.3.zip";
    let bytes = fs::read(out1.join(archive)).unwrap();
    assert_eq!(bytes, fs::read(out2.join(archive)).unwrap());
    assert!(bytes.starts_with(b"PK\x03\x04"));
    assert!(
        !run(&input, &out1, "2").status.success(),
        "must refuse to overwrite"
    );
    fs::write(input.join("material_manifest.json"),r#"{"version":1,"materials":[{"id":"test","albedo":"../secret.png"}],"textures":[{"name":"stone","material":"test","recipe":"stone","tint":[255,255,255]}]}"#).unwrap();
    let failed = temp.join("failed");
    assert!(!run(&input, &failed, "2").status.success());
    assert!(!failed.join("pack").exists());
    fs::remove_dir_all(temp).unwrap();
}
