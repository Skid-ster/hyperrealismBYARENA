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
    assert_eq!(
        fs::read_to_string(out1.join("pack/assets/minecraft/optifine/texture.properties")).unwrap(),
        "format=lab-pbr/1.3\n"
    );
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
    let static_metadata = br#"{"texture":{"mipmap_strategy":"dark_cutout"}}"#;
    fs::write(blocks.join("stone.png.mcmeta"), static_metadata).unwrap();
    hyperrealism_core::zip::write(&tree, &jar).unwrap();
    let out_meta = temp.join("static-metadata");
    let valid = exec(&out_meta);
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    assert_eq!(
        fs::read(out_meta.join("pack/assets/minecraft/textures/block/stone.png.mcmeta")).unwrap(),
        static_metadata
    );
    {
        use std::io::Read;
        let zip = fs::File::open(out_meta.join("dist/HyperRealism128-v1.0-MC26.3.zip")).unwrap();
        let mut zip = zip::ZipArchive::new(zip).unwrap();
        let mut bytes = Vec::new();
        zip.by_name("assets/minecraft/textures/block/stone.png.mcmeta")
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(bytes, static_metadata);
    }
    fs::write(blocks.join("stone.png.mcmeta"), br#"{"animation":{}}"#).unwrap();
    hyperrealism_core::zip::write(&tree, &jar).unwrap();
    let fail_path = temp.join("animated");
    let fail = exec(&fail_path);
    assert!(!fail.status.success());
    assert!(String::from_utf8_lossy(&fail.stderr).contains("animation or unknown metadata"));
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
    assert_eq!(m.textures.len(), 96);
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

#[test]
fn labpbr_masks_preserve_metal_ids_emission_and_subsurface() {
    use hyperrealism_core::png_io::{read, write};
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "hyperrealism-masks-{}-{unique}",
        std::process::id()
    ));
    let input = temp.join("input");
    fs::create_dir_all(&input).unwrap();
    let mut metal = Vec::new();
    let mut emission = Vec::new();
    for y in 0..512 {
        for x in 0..512 {
            let m = if x >= 256 { 255 } else { 0 };
            let e = if y >= 256 { 255 } else { 0 };
            metal.extend_from_slice(&[m, m, m, 255]);
            emission.extend_from_slice(&[e, e, e, 255]);
        }
    }
    write(&input.join("metal.png"), 512, 512, &metal).unwrap();
    write(&input.join("emission.png"), 512, 512, &emission).unwrap();
    fs::copy("test_assets/sample.jpg", input.join("albedo.jpg")).unwrap();
    fs::write(input.join("material_manifest.json"),r#"{
      "version":1,
      "materials":[{"id":"base","albedo":"albedo.jpg"}],
      "textures":[{"name":"stone","material":"base","recipe":"material","tint":[255,255,255],
        "f0":10,"f0_mask":"metal.png","masked_f0":230,"subsurface":200,"emissive_map":"emission.png"},
        {"name":"gold_ore","material":"base","recipe":"ore","tint":[255,255,255],"f0":10,"ore_color":[200,160,60],"ore_f0":231}]
    }"#).unwrap();
    let out = temp.join("out");
    let result = run(&input, &out, "2");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let image = read(&out.join("pack/assets/minecraft/textures/block/stone_s.png")).unwrap();
    let at = |x: usize, y: usize| &image.pixels[(y * 128 + x) * 4..(y * 128 + x + 1) * 4];
    assert_eq!(&at(32, 32)[1..], &[10, 200, 0]);
    assert_eq!(&at(96, 32)[1..], &[230, 0, 0]);
    assert_eq!(&at(32, 96)[1..], &[10, 200, 254]);
    assert_eq!(&at(96, 96)[1..], &[230, 0, 254]);
    assert!(image
        .pixels
        .chunks_exact(4)
        .all(|p| p[1] == 10 || p[1] == 230));
    assert!(image.pixels.chunks_exact(4).all(|p| p[3] != 255));
    assert!(image.pixels.chunks_exact(4).any(|p| p[3] > 0 && p[3] < 254));
    let ore = read(&out.join("pack/assets/minecraft/textures/block/gold_ore_s.png")).unwrap();
    assert!(ore.pixels.chunks_exact(4).any(|p| p[1] == 10));
    assert!(ore.pixels.chunks_exact(4).any(|p| p[1] == 231));
    assert!(ore
        .pixels
        .chunks_exact(4)
        .all(|p| p[1] == 10 || p[1] == 231));
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(out.join("build_report.json")).unwrap()).unwrap();
    assert!(
        report["input_sha256"]["metal.png"].is_string()
            && report["input_sha256"]["emission.png"].is_string()
    );
    for (i, extra) in [
        serde_json::json!({"f0":230,"porosity":12}),
        serde_json::json!({"f0_mask":"metal.png"}),
        serde_json::json!({"masked_f0":230}),
        serde_json::json!({"f0_mask":"metal.png","masked_f0":240}),
        serde_json::json!({"subsurface":64}),
        serde_json::json!({"subsurface":100,"porosity":1}),
        serde_json::json!({"emissive_map":"emission.png","emissive":100}),
        serde_json::json!({"ore_color":[200,160,60],"ore_f0":231}),
    ]
    .into_iter()
    .enumerate()
    {
        let mut texture = serde_json::json!({"name":"stone","material":"base","recipe":"material","tint":[255,255,255]});
        texture
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let manifest = serde_json::json!({"version":1,"materials":[{"id":"base","albedo":"albedo.jpg"}],"textures":[texture]});
        fs::write(
            input.join("material_manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let failed = temp.join(format!("invalid-{i}"));
        assert!(
            !run(&input, &failed, "2").status.success(),
            "invalid case {i}"
        );
        assert!(!failed.exists());
    }
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn transparent_albedo_is_filtered_without_color_bleeding() {
    use hyperrealism_core::png_io::{read, write};
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "hyperrealism-alpha-{}-{unique}",
        std::process::id()
    ));
    let input = temp.join("input");
    fs::create_dir_all(&input).unwrap();
    let mut pixels = Vec::new();
    for _y in 0..512 {
        for x in 0..512 {
            pixels.extend_from_slice(if x < 256 {
                &[255, 0, 0, 255]
            } else {
                &[0, 255, 0, 0]
            });
        }
    }
    write(&input.join("color.png"), 512, 512, &pixels).unwrap();
    fs::write(input.join("material_manifest.json"),r#"{"version":1,"materials":[{"id":"base","albedo":"color.png"}],"textures":[{"name":"stone","material":"base","recipe":"material","tint":[255,255,255]}]}"#).unwrap();
    let output = temp.join("out");
    let result = run(&input, &output, "2");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let a = read(&output.join("pack/assets/minecraft/textures/block/stone.png")).unwrap();
    assert!(a.pixels.chunks_exact(4).any(|p| p[3] == 0));
    assert!(a.pixels.chunks_exact(4).any(|p| p[3] == 255));
    assert!(a
        .pixels
        .chunks_exact(4)
        .filter(|p| p[3] > 0)
        .all(|p| p[0] >= 253 && p[1] == 0 && p[2] == 0));
    fs::remove_dir_all(temp).unwrap();
}
