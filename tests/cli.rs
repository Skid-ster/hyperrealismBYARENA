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

#[test]
fn real_material_channels_and_input_downsampling() {
    use hyperrealism_core::png_io::{read, read_material, write};
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp =
        std::env::temp_dir().join(format!("hyperrealism-maps-{}-{unique}", std::process::id()));
    let input = temp.join("inputs");
    fs::create_dir_all(&input).unwrap();
    let size = 1024;
    let mut color = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let v = if (x + y) % 2 == 0 { 0 } else { 255 };
            color.extend_from_slice(&[v, v, v, 255]);
        }
    }
    write(&input.join("albedo.png"), size, size, &color).unwrap();
    let constant = |path: &str, value: [u8; 4]| {
        let mut pixels = Vec::new();
        for _ in 0..64 * 64 {
            pixels.extend_from_slice(&value);
        }
        write(&input.join(path), 64, 64, &pixels).unwrap();
    };
    constant("height.png", [64, 64, 64, 255]);
    constant("ao.png", [153, 153, 153, 255]);
    constant("roughness.png", [64, 64, 64, 255]);
    constant("normal.png", [128, 192, 238, 255]);
    fs::write(input.join("material_manifest.json"),r#"{
      "version":1,
      "materials":[
        {"id":"dx","albedo":"albedo.png","normal":"normal.png","normal_y":"directx","roughness":"roughness.png","height":"height.png","ao":"ao.png"},
        {"id":"gl","albedo":"albedo.png","normal":"normal.png","normal_y":"opengl","roughness":"roughness.png","height":"height.png","ao":"ao.png"}
      ],
      "textures":[
        {"name":"iron_block","material":"dx","recipe":"material","tint":[255,255,255],"f0":230},
        {"name":"gold_block","material":"gl","recipe":"material","tint":[255,255,255],"f0":231}
      ]
    }"#).unwrap();
    let output = temp.join("output");
    let result = run(&input, &output, "2");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let block = output.join("pack/assets/minecraft/textures/block");
    let a = read(&block.join("iron_block.png")).unwrap();
    let dx = read(&block.join("iron_block_n.png")).unwrap();
    let gl = read(&block.join("gold_block_n.png")).unwrap();
    let s = read(&block.join("iron_block_s.png")).unwrap();
    // A 1-pixel checkerboard must be filtered in linear space, not sampled
    // by nearest-neighbour (which would pick black everywhere).
    assert!(a
        .pixels
        .chunks_exact(4)
        .all(|p| (186..=189).contains(&p[0])));
    assert!(dx
        .pixels
        .chunks_exact(4)
        .all(|p| p[1] > 180 && (152..=154).contains(&p[2]) && (63..=65).contains(&p[3])));
    assert!(gl
        .pixels
        .chunks_exact(4)
        .all(|p| p[1] < 75 && (152..=154).contains(&p[2]) && (63..=65).contains(&p[3])));
    assert!(s
        .pixels
        .chunks_exact(4)
        .all(|p| (126..=128).contains(&p[0]) && p[1] == 230 && p[3] == 255));
    let jpeg = read_material(Path::new("test_assets/sample.jpg")).unwrap();
    assert_eq!((jpeg.width, jpeg.height), (32, 32));
    assert!(jpeg
        .pixels
        .chunks_exact(4)
        .all(|p| (98..=102).contains(&p[0])
            && (148..=152).contains(&p[1])
            && (198..=202).contains(&p[2])
            && p[3] == 255));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn local_client_jar_preflight_checks_targets_before_writing() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp =
        std::env::temp_dir().join(format!("hyperrealism-jar-{}-{unique}", std::process::id()));
    let input = temp.join("input");
    fs::create_dir_all(&input).unwrap();
    fs::copy("test_assets/sample.jpg", input.join("albedo.jpg")).unwrap();
    fs::write(input.join("material_manifest.json"),r#"{"version":1,"materials":[{"id":"base","albedo":"albedo.jpg","source":"synthetic fixture","license":"CC0"}],"checksums":{"albedo.jpg":"1a49aa3a280b16ec050937f5d1a6ea55e3f49e73fd48430e5a4cb0fd3101923f"},"textures":[{"name":"stone","material":"base","recipe":"material","tint":[255,255,255],"target_color":[120,150,180]}]}"#).unwrap();
    let tree = temp.join("jar-tree");
    let blocks = tree.join("assets/minecraft/textures/block");
    fs::create_dir_all(&blocks).unwrap();
    fs::write(blocks.join("stone.png"), b"fixture").unwrap();
    let jar = temp.join("client.jar");
    hyperrealism_core::zip::write(&tree, &jar).unwrap();
    let exec = |out: &Path| {
        Command::new(env!("CARGO_BIN_EXE_hyperrealism-core"))
            .arg("--input")
            .arg(&input)
            .arg("--output")
            .arg(out)
            .arg("--cpu-only")
            .arg("--threads")
            .arg("2")
            .arg("--minecraft-jar")
            .arg(&jar)
            .output()
            .unwrap()
    };
    let valid = exec(&temp.join("ok"));
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    let albedo = hyperrealism_core::png_io::read(
        &temp.join("ok/pack/assets/minecraft/textures/block/stone.png"),
    )
    .unwrap();
    assert!(albedo
        .pixels
        .chunks_exact(4)
        .all(|p| (119..=121).contains(&p[0])
            && (149..=151).contains(&p[1])
            && (179..=181).contains(&p[2])));
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.join("ok/build_report.json")).unwrap()).unwrap();
    assert_eq!(
        report["input_sha256"]["albedo.jpg"],
        "1a49aa3a280b16ec050937f5d1a6ea55e3f49e73fd48430e5a4cb0fd3101923f"
    );
    assert_eq!(report["materials"][0]["license"], "CC0");
    fs::write(input.join("albedo.jpg"), b"changed").unwrap();
    let failed_hash = temp.join("bad-hash");
    let failed = exec(&failed_hash);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("SHA-256 mismatch"));
    assert!(!failed_hash.exists());
    fs::copy("test_assets/sample.jpg", input.join("albedo.jpg")).unwrap();
    fs::write(blocks.join("stone.png.mcmeta"), br#"{"animation":{}}"#).unwrap();
    hyperrealism_core::zip::write(&tree, &jar).unwrap();
    let fail_path = temp.join("animated");
    let fail = exec(&fail_path);
    assert!(!fail.status.success());
    assert!(String::from_utf8_lossy(&fail.stderr).contains("metadata/animation"));
    assert!(!fail_path.exists());
    fs::remove_file(blocks.join("stone.png")).unwrap();
    fs::remove_file(blocks.join("stone.png.mcmeta")).unwrap();
    hyperrealism_core::zip::write(&tree, &jar).unwrap();
    let fail_path = temp.join("missing");
    let fail = exec(&fail_path);
    assert!(!fail.status.success());
    assert!(String::from_utf8_lossy(&fail.stderr).contains("not found in client JAR"));
    assert!(!fail_path.exists());
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn scalar_png_16_bit_precision_is_preserved() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "hyperrealism-16bit-{}-{unique}.png",
        std::process::id()
    ));
    {
        let file = fs::File::create(&temp).unwrap();
        let mut encoder = png::Encoder::new(file, 2, 1);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Sixteen);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[0x80, 0x00, 0x80, 0x01]).unwrap();
    }
    let (w, h, values) = hyperrealism_core::png_io::read_linear_scalar(&temp).unwrap();
    assert_eq!((w, h), (2, 1));
    assert!((values[0] - 32768.0 / 65535.0).abs() < 1e-7);
    assert!((values[1] - 32769.0 / 65535.0).abs() < 1e-7);
    assert!(values[1] > values[0]);
    fs::remove_file(temp).unwrap();
}

#[test]
fn bundled_catalog_is_consistent_with_audited_26_3_inventory() {
    let root = Path::new("work");
    let m = hyperrealism_core::manifest::read(&root.join("material_manifest.json")).unwrap();
    hyperrealism_core::manifest::validate(&m, root).unwrap();
    let hashes = hyperrealism_core::manifest::input_hashes(&m, root).unwrap();
    assert_eq!(hashes.len(), 20);
    let audit: serde_json::Value =
        serde_json::from_slice(&fs::read("catalog/vanilla_26.3_audit.json").unwrap()).unwrap();
    let entries = audit["textures"].as_object().unwrap();
    assert_eq!(entries.len(), m.textures.len());
    assert_eq!(m.textures.len(), 80);
    for t in &m.textures {
        let entry = &entries[&t.name];
        assert_eq!(entry["dimensions"], serde_json::json!([16, 16]));
        assert_eq!(entry["has_mcmeta"], false);
        assert_eq!(
            entry["target_rgb"],
            serde_json::json!(t.target_color.unwrap())
        );
    }
}
