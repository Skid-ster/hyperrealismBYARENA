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
        .arg("--resolution")
        .arg("128")
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
    let entity_width = 128;
    let entity_height = 64;
    let entity_pixels = entity_width * entity_height;
    hyperrealism_core::png_io::write(
        &input.join("entity.png"),
        entity_width,
        entity_height,
        &[180, 110, 75, 255].repeat(entity_pixels),
    )
    .unwrap();
    hyperrealism_core::png_io::write(
        &input.join("entity_normal.png"),
        entity_width,
        entity_height,
        &[128, 128, 0, 255].repeat(entity_pixels),
    )
    .unwrap();
    hyperrealism_core::png_io::write(
        &input.join("entity_specular.png"),
        entity_width,
        entity_height,
        &[128, 40, 0, 255].repeat(entity_pixels),
    )
    .unwrap();
    fs::write(input.join("material_manifest.json"),r#"{
      "version":1,"materials":[{"id":"test","albedo":"diffuse.png","normal":"normal.png","roughness":"roughness.png"}],
      "textures":[{"name":"iron_block","material":"test","recipe":"metal","tint":[255,255,255],"f0":230}],
      "entity_textures":[{"path":"entity/test/test","albedo":"entity.png","normal":"entity_normal.png","specular":"entity_specular.png","source":"test source","license":"test license"}]
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
        meta["pack"]["description"],
        "HyperRealism 128×128 — LabPBR 1.3 (experimental)"
    );
    assert_eq!(
        fs::read_to_string(out1.join("pack/assets/minecraft/optifine/texture.properties")).unwrap(),
        "format=lab-pbr/1.3\n"
    );
    let pack_readme = fs::read_to_string(out1.join("pack/README.txt")).unwrap();
    assert!(pack_readme.contains("replaces 1 block textures"));
    assert!(pack_readme.contains("replaces 1 entity texture paths"));
    let entity_root = out1.join("pack/assets/minecraft/textures/entity/test");
    assert_eq!(
        fs::read(entity_root.join("test.png")).unwrap(),
        fs::read(input.join("entity.png")).unwrap()
    );
    assert_eq!(
        fs::read(entity_root.join("test_n.png")).unwrap(),
        fs::read(input.join("entity_normal.png")).unwrap()
    );
    assert_eq!(
        fs::read(entity_root.join("test_s.png")).unwrap(),
        fs::read(input.join("entity_specular.png")).unwrap()
    );
    let entity_image = hyperrealism_core::png_io::read(&entity_root.join("test.png")).unwrap();
    assert_eq!(
        (entity_image.width, entity_image.height),
        (entity_width, entity_height)
    );
    let verify = Command::new(env!("CARGO_BIN_EXE_hyperrealism-core"))
        .arg("--verify-pack")
        .arg(out1.join("pack"))
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(String::from_utf8_lossy(&verify.stdout).contains("1 entity LabPBR"));
    let incomplete = temp.join("incomplete-entity-pack");
    let incomplete_root = incomplete.join("assets/minecraft/textures/entity/test");
    fs::create_dir_all(&incomplete_root).unwrap();
    fs::copy(entity_root.join("test.png"), incomplete_root.join("test.png")).unwrap();
    fs::copy(
        entity_root.join("test_n.png"),
        incomplete_root.join("test_n.png"),
    )
    .unwrap();
    assert!(hyperrealism_core::verify::entity_pack(&incomplete).is_err());
    let entity_credits = fs::read_to_string(out1.join("pack/ENTITY_TEXTURES.txt")).unwrap();
    assert!(entity_credits.contains("Path: entity/test/test"));
    assert!(entity_credits.contains("License: test license"));
    let credits = fs::read_to_string(out1.join("pack/MATERIAL_CREDITS.txt")).unwrap();
    assert!(credits.contains("Material: test"));
    assert!(credits.contains("not specified"));
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
fn default_256_resolution_downsamples_entity_uvs_and_preserves_labpbr_ids() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "hyperrealism-256-{}-{unique}",
        std::process::id()
    ));
    let input = temp.join("input");
    fs::create_dir_all(&input).unwrap();
    for name in ["diffuse.png", "normal.png", "roughness.png"] {
        fs::copy(Path::new("test_assets").join(name), input.join(name)).unwrap();
    }

    let (width, height) = (512, 256);
    let mut albedo = Vec::with_capacity(width * height * 4);
    let mut normal = Vec::with_capacity(width * height * 4);
    let mut specular = Vec::with_capacity(width * height * 4);
    for _y in 0..height {
        for x in 0..width {
            let color = if x < width / 2 {
                [175, 105, 70, 255]
            } else {
                [210, 160, 120, 255]
            };
            albedo.extend_from_slice(&color);
            normal.extend_from_slice(&[128, 128, 255, 128]);
            specular.extend_from_slice(if x < width / 2 {
                &[128, 10, 0, 255]
            } else {
                &[128, 231, 0, 0]
            });
        }
    }
    for (name, pixels) in [
        ("entity.png", albedo),
        ("entity_normal.png", normal),
        ("entity_specular.png", specular),
    ] {
        hyperrealism_core::png_io::write(&input.join(name), width, height, &pixels).unwrap();
    }
    fs::write(
        input.join("material_manifest.json"),
        r#"{"version":1,"materials":[{"id":"test","albedo":"diffuse.png","normal":"normal.png","roughness":"roughness.png"}],"textures":[{"name":"iron_block","material":"test","recipe":"metal","tint":[255,255,255],"f0":230}],"entity_textures":[{"path":"entity/test/test","albedo":"entity.png","normal":"entity_normal.png","specular":"entity_specular.png","source":"test source","license":"test license"}]}"#,
    )
    .unwrap();

    let output = temp.join("output");
    let result = Command::new(env!("CARGO_BIN_EXE_hyperrealism-core"))
        .arg("--input")
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .arg("--cpu-only")
        .arg("--threads")
        .arg("2")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let pack = output.join("pack");
    assert_eq!(hyperrealism_core::verify::full_pack(&pack).unwrap(), (1, 1));
    let block = hyperrealism_core::png_io::read(
        &pack.join("assets/minecraft/textures/block/iron_block.png"),
    )
    .unwrap();
    assert_eq!((block.width, block.height), (256, 256));
    let entity = pack.join("assets/minecraft/textures/entity/test/test");
    for name in ["test.png", "test_n.png", "test_s.png"] {
        let image = hyperrealism_core::png_io::read(&entity.with_file_name(name)).unwrap();
        assert_eq!((image.width, image.height), (256, 128));
    }
    let spec = hyperrealism_core::png_io::read(&entity.with_file_name("test_s.png")).unwrap();
    let ids: std::collections::HashSet<u8> =
        spec.pixels.chunks_exact(4).map(|p| p[1]).collect();
    let emissions: std::collections::HashSet<u8> =
        spec.pixels.chunks_exact(4).map(|p| p[3]).collect();
    assert_eq!(ids, [10, 231].into_iter().collect());
    assert_eq!(emissions, [0, 255].into_iter().collect());
    let meta: serde_json::Value =
        serde_json::from_slice(&fs::read(pack.join("pack.mcmeta")).unwrap()).unwrap();
    assert_eq!(
        meta["pack"]["description"],
        "HyperRealism 256×256 — LabPBR 1.3 (experimental)"
    );

    let explicit_output = temp.join("explicit-256");
    let explicit = Command::new(env!("CARGO_BIN_EXE_hyperrealism-core"))
        .arg("--input")
        .arg(&input)
        .arg("--output")
        .arg(&explicit_output)
        .arg("--cpu-only")
        .arg("--resolution")
        .arg("256")
        .output()
        .unwrap();
    assert!(
        explicit.status.success(),
        "{}",
        String::from_utf8_lossy(&explicit.stderr)
    );
    assert_eq!(
        hyperrealism_core::verify::full_pack(&explicit_output.join("pack")).unwrap(),
        (1, 1)
    );
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
    // Exercise the optional 512px output and inspect its LabPBR values after a
    // 1K source is downsampled; test DX/GL normals, AO, height and smoothness.
    let output512 = temp.join("output512");
    let result512 = Command::new(env!("CARGO_BIN_EXE_hyperrealism-core"))
        .arg("--input")
        .arg(&input)
        .arg("--output")
        .arg(&output512)
        .arg("--cpu-only")
        .arg("--resolution")
        .arg("512")
        .arg("--threads")
        .arg("2")
        .output()
        .unwrap();
    assert!(
        result512.status.success(),
        "{}",
        String::from_utf8_lossy(&result512.stderr)
    );
    let pack512 = output512.join("pack");
    assert_eq!(hyperrealism_core::verify::pack(&pack512).unwrap(), 2);
    let block512 = pack512.join("assets/minecraft/textures/block");
    let albedo512 = read(&block512.join("iron_block.png")).unwrap();
    let dx512 = read(&block512.join("iron_block_n.png")).unwrap();
    let gl512 = read(&block512.join("gold_block_n.png")).unwrap();
    let spec512 = read(&block512.join("iron_block_s.png")).unwrap();
    for img in [&albedo512, &dx512, &gl512, &spec512] {
        assert_eq!((img.width, img.height), (512, 512));
    }
    assert!(albedo512
        .pixels
        .chunks_exact(4)
        .all(|p| (186..=189).contains(&p[0])));
    assert!(dx512
        .pixels
        .chunks_exact(4)
        .all(|p| p[1] > 180 && (152..=154).contains(&p[2]) && (63..=65).contains(&p[3])));
    assert!(gl512
        .pixels
        .chunks_exact(4)
        .all(|p| p[1] < 75 && (152..=154).contains(&p[2]) && (63..=65).contains(&p[3])));
    assert!(spec512
        .pixels
        .chunks_exact(4)
        .all(|p| (126..=128).contains(&p[0]) && p[1] == 230 && p[3] == 255));
    // A single mismatched map must fail verification, not silently mix sizes.
    fs::copy(
        block.join("gold_block_n.png"),
        block512.join("gold_block_n.png"),
    )
    .unwrap();
    assert!(hyperrealism_core::verify::pack(&pack512).is_err());
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
            .arg("--resolution")
            .arg("128")
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
    assert_eq!(hashes.len(), 170);
    let leaf_material = m
        .materials
        .iter()
        .find(|material| material.id == "tree_leaves")
        .expect("shared foliage material");
    assert_eq!(
        leaf_material.height.as_deref(),
        Some("materials/tree_leaves/height.png")
    );
    let derived_height_count = m
        .materials
        .iter()
        .filter(|material| material.height_from_normal)
        .count();
    assert_eq!(derived_height_count, 26);
    assert!(m.materials.iter().all(|material| {
        material.roughness_bias.is_finite() && (-0.5..=0.5).contains(&material.roughness_bias)
    }));
    let material = |id: &str| {
        m.materials
            .iter()
            .find(|material| material.id == id)
            .unwrap_or_else(|| panic!("missing material profile: {id}"))
    };
    assert!(material("procedural_glass").roughness_bias < 0.0);
    assert!(material("metal_plate_02").roughness_bias < 0.0);
    assert!(material("brown_mud_dry").roughness_bias > 0.0);
    assert!(material("aerial_sand").roughness_bias > 0.0);
    for stone in ["cobble_photo", "mossy_cobble_photo"] {
        assert!(material(stone).roughness_from_height);
        assert!(material(stone).height.is_some());
    }
    assert_eq!(
        leaf_material.ao.as_deref(),
        Some("materials/tree_leaves/ao.png")
    );
    assert!(hashes.contains_key("materials/tree_leaves/height.png"));
    assert!(hashes.contains_key("materials/tree_leaves/ao.png"));
    assert_eq!(
        m.textures
            .iter()
            .filter(|texture| texture.material == "tree_leaves")
            .count(),
        9
    );
    let audit: serde_json::Value =
        serde_json::from_slice(&fs::read("catalog/vanilla_26.3_audit.json").unwrap()).unwrap();
    let entries = audit["textures"].as_object().unwrap();
    assert_eq!(entries.len(), m.textures.len());
    assert_eq!(m.textures.len(), 364);
    for (name, rounded_target) in [
        ("chiseled_deepslate", [54, 54, 55]),
        ("chiseled_red_sandstone", [183, 97, 28]),
        ("chiseled_sandstone", [216, 203, 155]),
        ("chiseled_stone_bricks", [120, 119, 120]),
        ("chiseled_tuff", [89, 94, 87]),
        ("chiseled_tuff_bricks", [99, 103, 96]),
        ("chiseled_tuff_bricks_top", [111, 114, 107]),
        ("chiseled_tuff_top", [94, 99, 91]),
        ("chiseled_copper", [184, 101, 74]),
        ("exposed_chiseled_copper", [155, 119, 101]),
        ("weathered_chiseled_copper", [105, 151, 111]),
        ("oxidized_chiseled_copper", [84, 162, 132]),
    ] {
        let texture = m.textures.iter().find(|texture| texture.name == name).unwrap();
        assert_eq!(texture.recipe, "carved");
        assert_eq!(texture.target_color, Some(rounded_target));
        assert_eq!(entries[name]["target_rgb"], serde_json::json!(rounded_target));
    }
    for (name, ore_id) in [
        ("raw_copper_block", 10),
        ("raw_gold_block", 231),
        ("raw_iron_block", 10),
    ] {
        let texture = m.textures.iter().find(|texture| texture.name == name).unwrap();
        assert_eq!(texture.material, "rock_face");
        assert_eq!(texture.recipe, "ore");
        assert_eq!(texture.ore_f0, Some(ore_id));
    }
    let mut entity_paths: Vec<_> = m.entity_textures.iter().map(|entity| entity.path.as_str()).collect();
    entity_paths.sort_unstable();
    assert_eq!(
        entity_paths,
        vec![
            "entity/chicken/chicken_temperate",
            "entity/cow/cow_temperate",
            "entity/pig/pig_temperate",
            "entity/sheep/sheep",
            "entity/sheep/sheep_wool",
        ]
    );
    assert!(m.entity_textures.iter().all(|entity| {
        entity.source.as_deref().unwrap_or_default().contains("Original procedural")
            && [entity.albedo.as_str(), entity.normal.as_str(), entity.specular.as_str()]
                .iter()
                .all(|path| hashes.contains_key(*path))
    }));
    let glass: Vec<_> = m
        .textures
        .iter()
        .filter(|texture| texture.name.contains("glass"))
        .collect();
    assert_eq!(glass.len(), 35); // clear, stained, tinted and pane-cap sprites
    for texture in glass {
        let source = entries[&texture.name]["target_rgb"].as_array().unwrap();
        let slightly_darker: [u8; 3] = std::array::from_fn(|channel| {
            (source[channel].as_u64().unwrap() as f32 * 0.96).round() as u8
        });
        assert_eq!(texture.target_color, Some(slightly_darker), "{}", texture.name);
    }
    assert!((m.pbr_strength - 0.2).abs() < 1e-6);
    assert!(m.textures.iter().all(|t| !t.name.starts_with("water_")));
    for wood in [
        "acacia",
        "bamboo",
        "birch",
        "cherry",
        "copper",
        "crimson",
        "dark_oak",
        "exposed_copper",
        "iron",
        "jungle",
        "mangrove",
        "oak",
        "oxidized_copper",
        "pale_oak",
        "poplar",
        "spruce",
        "warped",
        "weathered_copper",
    ] {
        for suffix in ["door_top", "door_bottom", "trapdoor"] {
            let name = format!("{wood}_{suffix}");
            let entry = m
                .textures
                .iter()
                .find(|t| t.name == name)
                .expect("door coverage");
            assert!(
                entry.layout.is_some(),
                "missing construction layout: {name}"
            );
        }
    }
    let toggled: Vec<_> = m
        .textures
        .iter()
        .filter(|t| {
            t.name == "lever"
                || t.name.ends_with("_door_top")
                || t.name.ends_with("_door_bottom")
                || t.name.ends_with("_trapdoor")
        })
        .collect();
    assert_eq!(toggled.len(), 55);
    let lever = toggled.iter().find(|t| t.name == "lever").unwrap();
    assert_eq!(lever.material, "oak_veneer_02_vertical");
    assert_eq!(lever.target_color, Some([111, 93, 67]));
    assert_eq!(entries["lever"]["git_blob_sha1"], "47121f2ecc25270b3b90f2bcbb2aaf6460a5c9ab");

    // Five bamboo texture paths were missing from Preview 11. They now use
    // the same CC0 maple-veneer inputs with a dedicated, periodic bamboo-board
    // recipe and version-pinned vanilla average-color targets.
    for (name, expected_color) in [
        ("bamboo_planks", [193, 173, 80]),
        ("bamboo_fence", [207, 186, 89]),
        ("bamboo_fence_particle", [207, 186, 85]),
        ("bamboo_fence_gate", [206, 186, 87]),
        ("bamboo_fence_gate_particle", [207, 186, 85]),
    ] {
        let texture = m.textures.iter().find(|t| t.name == name).unwrap();
        assert_eq!(texture.material, "white_maple_veneer");
        assert_eq!(texture.recipe, "bamboo");
        assert_eq!(texture.target_color, Some(expected_color));
    }
    for name in ["bamboo_fence_particle", "bamboo_fence_gate_particle"] {
        let texture = m.textures.iter().find(|t| t.name == name).unwrap();
        assert_eq!(texture.pattern_seed.as_deref(), Some("bamboo_particle"));
    }
    assert_eq!(
        entries["bamboo_fence_particle"]["git_blob_sha1"],
        entries["bamboo_fence_gate_particle"]["git_blob_sha1"]
    );

    // The tested preview uses a real cobble scan and its aligned height map,
    // not the previous thin Voronoi wireframe overlay.
    for (name, expected) in [
        ("cobblestone", "cobble_photo"),
        ("mossy_cobblestone", "mossy_cobble_photo"),
    ] {
        let texture = m.textures.iter().find(|t| t.name == name).unwrap();
        assert_eq!(texture.material, expected);
        assert_eq!(texture.recipe, "material");
        let material = m.materials.iter().find(|mat| mat.id == expected).unwrap();
        assert_eq!(
            material.height.as_deref(),
            Some("materials/cobble_photo/height.png")
        );
    }
    for t in &m.textures {
        let entry = &entries[&t.name];
        assert_eq!(entry["dimensions"], serde_json::json!([16, 16]));
        assert_eq!(entry["has_mcmeta"], t.mipmap_strategy.is_some());
        let audited = entry["target_rgb"].as_array().unwrap();
        let expected: [u8; 3] = std::array::from_fn(|channel| {
            let base = audited[channel].as_u64().unwrap() as f32;
            (if t.name.contains("glass") {
                (base * 0.96).round()
            } else {
                base
            }) as u8
        });
        assert_eq!(t.target_color, Some(expected), "{}", t.name);
    }
}

#[test]
fn geological_surfaces_and_ore_host_rock_use_rock_face_material() {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read("work/material_manifest.json").unwrap()).unwrap();
    let textures: std::collections::HashMap<&str, &str> = manifest["textures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            (
                t["name"].as_str().unwrap(),
                t["material"].as_str().unwrap(),
            )
        })
        .collect();
    for name in [
        "stone",
        "andesite",
        "tuff",
        "coal_ore",
        "copper_ore",
        "diamond_ore",
        "emerald_ore",
        "gold_ore",
        "iron_ore",
        "lapis_ore",
        "redstone_ore",
        "deepslate_coal_ore",
        "deepslate_copper_ore",
        "deepslate_diamond_ore",
        "deepslate_emerald_ore",
        "deepslate_gold_ore",
        "deepslate_iron_ore",
        "deepslate_lapis_ore",
        "deepslate_redstone_ore",
    ] {
        assert_eq!(textures[name], "rock_face", "{name}");
    }
}

#[test]
fn wood_and_wool_materials_match_species_and_grain_orientation() {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read("work/material_manifest.json").unwrap()).unwrap();
    assert_eq!(manifest["pbr_strength"], 0.2);
    let textures: std::collections::HashMap<&str, &str> = manifest["textures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| (
            t["name"].as_str().unwrap(),
            t["material"].as_str().unwrap(),
        ))
        .collect();
    for (name, material) in [
        ("oak_log", "bark_brown_01"),
        ("spruce_log", "bark_willow_02"),
        ("dark_oak_log", "bark_platanus"),
        ("acacia_log", "bark_willow_02"),
        ("jungle_log", "bark_platanus"),
        ("dark_oak_planks", "walnut_horizontal"),
        ("stripped_dark_oak_log", "walnut"),
        ("dark_oak_door_top", "walnut"),
        ("dark_oak_door_bottom", "walnut"),
        ("dark_oak_trapdoor", "walnut"),
        ("pale_oak_planks", "white_maple_veneer"),
        ("pale_oak_log_top", "white_maple_veneer"),
        ("stripped_pale_oak_log", "white_maple_veneer_vertical"),
        ("stripped_pale_oak_log_top", "white_maple_veneer"),
        ("pale_oak_door_top", "white_maple_veneer_vertical"),
        ("pale_oak_door_bottom", "white_maple_veneer_vertical"),
        ("pale_oak_trapdoor", "white_maple_veneer_vertical"),
    ] {
        assert_eq!(textures[name], material, "{name}");
    }
    for color in [
        "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
        "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black",
    ] {
        let name = format!("{color}_wool");
        assert_eq!(textures[name.as_str()], "poly_wool_herringbone", "{name}");
    }
    let materials: std::collections::HashMap<&str, &serde_json::Value> = manifest["materials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| (m["id"].as_str().unwrap(), m))
        .collect();
    let walnut_horizontal = "walnut_horizontal";
    assert_eq!(materials[walnut_horizontal]["rotate_cw"], true);
    assert_eq!(materials[walnut_horizontal]["normal_y"], "opengl");
    for map in ["albedo", "normal", "roughness", "ao"] {
        assert!(materials[walnut_horizontal][map].is_string(), "walnut: {map}");
    }
    for source in ["bark_brown_01", "bark_platanus"] {
        assert_eq!(materials[source]["normal_y"], "opengl", "{source}");
        for map in ["albedo", "normal", "roughness"] {
            assert!(materials[source][map].is_string(), "{source}: {map}");
        }
    }
    for texture in manifest["textures"].as_array().unwrap() {
        assert!(texture.get("pbr_strength").is_none() || texture["pbr_strength"].is_null());
    }
}

#[test]
fn nether_surface_coverage_uses_matching_cc0_materials_and_keeps_animations() {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read("work/material_manifest.json").unwrap()).unwrap();
    let textures: std::collections::HashMap<&str, &serde_json::Value> = manifest["textures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| (t["name"].as_str().unwrap(), t))
        .collect();
    for (name, material) in [
        ("netherrack", "nether_lava_003"),
        ("nether_gold_ore", "nether_lava_003"),
        ("nether_quartz_ore", "nether_lava_003"),
        ("blackstone", "nether_dark_rock"),
        ("blackstone_top", "nether_dark_rock"),
        ("polished_blackstone", "nether_dark_rock"),
        ("basalt_side", "nether_dark_rock"),
        ("polished_basalt_top", "nether_dark_rock"),
        ("obsidian", "nether_dark_rock"),
        ("nether_bricks", "brick_red"),
        ("red_nether_bricks", "brick_red"),
        ("quartz_block_side", "concrete_wall"),
        ("soul_sand", "brown_mud_dry"),
        ("soul_soil", "brown_mud_03"),
        ("crimson_planks", "cherry_veneer"),
        ("warped_planks", "white_maple_veneer"),
        ("crimson_nylium", "forest_ground"),
        ("warped_nylium", "forest_ground"),
        ("crying_obsidian", "nether_lava_flow"),
    ] {
        assert_eq!(textures[name]["material"], material, "{name}");
    }
    assert_eq!(
        textures["crying_obsidian"]["emissive_map"],
        "materials/nether_lava_flow/emission.jpg"
    );
    assert_eq!(textures["glowstone"]["emissive"], 112);
    assert_eq!(textures["shroomlight"]["emissive"], 72);
    let materials: std::collections::HashMap<&str, &serde_json::Value> = manifest["materials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| (m["id"].as_str().unwrap(), m))
        .collect();
    for id in ["nether_lava_003", "nether_dark_rock", "nether_lava_flow"] {
        assert_eq!(materials[id]["license"], "CC0-1.0", "{id}");
        assert_eq!(materials[id]["normal_y"], "opengl", "{id}");
        for map in ["albedo", "normal", "arm"] {
            assert!(materials[id][map].is_string(), "{id}: {map}");
        }
    }
    for name in [
        "magma",
        "nether_portal",
        "crimson_stem",
        "warped_stem",
        "crimson_fungus",
        "warped_fungus",
    ] {
        assert!(
            !textures.contains_key(name),
            "animated/cutout texture was modified: {name}"
        );
    }
    assert_eq!(manifest["pbr_strength"], 0.2);
    assert!(manifest["textures"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t.get("pbr_strength").is_none() || t["pbr_strength"].is_null()));
}

#[test]
fn exhaustive_26_3_block_texture_inventory_is_explicit_about_gaps() {
    let catalog: serde_json::Value = serde_json::from_slice(
        &fs::read("catalog/vanilla_26.3_full_block_coverage.json").unwrap(),
    )
    .unwrap();
    assert_eq!(catalog["version"], 1);
    assert_eq!(
        catalog["source_commit"],
        "6c088630459d8ca369ef0d6c07636d7d63a3b0f1"
    );
    let summary = &catalog["summary"];
    assert_eq!(summary["blockstates"], 1290);
    assert_eq!(summary["block_model_files"], 2885);
    assert_eq!(summary["unique_block_texture_references"], 1272);
    assert_eq!(summary["currently_covered_block_texture_references"], 364);
    assert_eq!(summary["uncovered_block_texture_references"], 908);
    assert_eq!(summary["states_with_uncovered_textures"], 596);
    assert!(summary["metadata_resolution_problems"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(summary["block_texture_references_not_referenced_by_any_blockstate"]
        .as_array()
        .unwrap()
        .is_empty());

    let states = catalog["blockstates"].as_object().unwrap();
    assert_eq!(states.len(), 1290);
    for (block, state) in states {
        let referenced = state["block_textures"].as_array().unwrap();
        let missing = state["uncovered_block_textures"].as_array().unwrap();
        assert!(
            missing.iter().all(|name| referenced.contains(name)),
            "{block}: uncovered list must be a subset of the block texture references"
        );
    }
    assert_eq!(catalog["uncovered_block_textures"].as_array().unwrap().len(), 908);
    let covered: std::collections::HashSet<&str> = catalog["covered_block_textures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap())
        .collect();
    for sprite in [
        "glass",
        "glass_pane_top",
        "black_stained_glass",
        "black_stained_glass_pane_top",
    ] {
        assert!(covered.contains(sprite), "26.3 sprite metadata missed {sprite}");
    }
}

#[test]
fn representative_26_3_models_share_the_covered_texture_paths() {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read("work/material_manifest.json").unwrap()).unwrap();
    let names: std::collections::HashSet<&str> = manifest["textures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    let audit: serde_json::Value = serde_json::from_slice(
        &fs::read("catalog/vanilla_26.3_variant_texture_refs.json").unwrap(),
    )
    .unwrap();
    assert_eq!(audit["version"], "26.3");
    assert_eq!(
        audit["source_commit"],
        "6c088630459d8ca369ef0d6c07636d7d63a3b0f1"
    );
    let blocks = audit["blocks"].as_object().unwrap();
    assert_eq!(blocks.len(), 160);
    for (block, textures) in blocks {
        let textures = textures.as_array().unwrap();
        assert!(
            !textures.is_empty(),
            "{block}: model audit has no block textures"
        );
        for texture in textures {
            let texture = texture.as_str().unwrap();
            assert!(
                names.contains(texture),
                "{block} references uncovered {texture}"
            );
        }
    }

    // In 26.3, every state/model variant of cobblestone stairs, slabs and
    // walls resolves to the same cobblestone sprite. PBR sidecars are emitted
    // once for that path, so the geometry shares the exact same material.
    for block in [
        "cobblestone",
        "cobblestone_stairs",
        "cobblestone_slab",
        "cobblestone_wall",
    ] {
        assert_eq!(blocks[block], serde_json::json!(["cobblestone"]));
    }
    for (block, expected) in [
        ("stone_stairs", "stone"),
        ("stone_slab", "stone"),
        ("stone_brick_stairs", "stone_bricks"),
        ("stone_brick_slab", "stone_bricks"),
        ("stone_brick_wall", "stone_bricks"),
        ("brick_stairs", "bricks"),
        ("brick_slab", "bricks"),
        ("brick_wall", "bricks"),
        ("oak_stairs", "oak_planks"),
        ("oak_slab", "oak_planks"),
        ("oak_fence", "oak_planks"),
        ("oak_fence_gate", "oak_planks"),
        ("cobbled_deepslate_stairs", "cobbled_deepslate"),
        ("cobbled_deepslate_slab", "cobbled_deepslate"),
        ("cobbled_deepslate_wall", "cobbled_deepslate"),
    ] {
        assert_eq!(blocks[block], serde_json::json!([expected]), "{block}");
    }
    // Models with distinct faces intentionally keep their separate vanilla
    // texture paths (sandstone top/bottom and deepslate/blackstone end caps).
    assert_eq!(
        blocks["sandstone_stairs"],
        serde_json::json!(["sandstone", "sandstone_bottom", "sandstone_top"])
    );
    assert_eq!(
        blocks["deepslate"],
        serde_json::json!(["deepslate", "deepslate_top"])
    );
    assert_eq!(
        blocks["blackstone_stairs"],
        serde_json::json!(["blackstone", "blackstone_top"])
    );
    for (block, expected) in [
        (
            "ancient_debris",
            serde_json::json!(["ancient_debris_side", "ancient_debris_top"]),
        ),
        (
            "basalt",
            serde_json::json!(["basalt_side", "basalt_top"]),
        ),
        (
            "polished_basalt",
            serde_json::json!(["polished_basalt_side", "polished_basalt_top"]),
        ),
        (
            "quartz_block",
            serde_json::json!(["quartz_block_side", "quartz_block_top"]),
        ),
        (
            "crimson_nylium",
            serde_json::json!(["crimson_nylium", "crimson_nylium_side", "netherrack"]),
        ),
        (
            "warped_nylium",
            serde_json::json!(["netherrack", "warped_nylium", "warped_nylium_side"]),
        ),
    ] {
        assert_eq!(blocks[block], expected, "{block}");
    }
    assert_eq!(
        blocks["oak_door"],
        serde_json::json!(["oak_door_bottom", "oak_door_top"])
    );
    assert_eq!(blocks["oak_trapdoor"], serde_json::json!(["oak_trapdoor"]));
    assert_eq!(blocks["lever"], serde_json::json!(["cobblestone", "lever"]));
    let states = audit["interactive_states"].as_object().unwrap();
    assert_eq!(states["lever"]["variant_count"], 24);
    assert_eq!(
        states["lever"]["powered_false_models"],
        serde_json::json!(["minecraft:block/lever_on"])
    );
    assert_eq!(
        states["lever"]["powered_true_models"],
        serde_json::json!(["minecraft:block/lever"])
    );
    assert_eq!(states["oak_door"]["variant_count"], 32);
    assert_eq!(states["oak_trapdoor"]["variant_count"], 16);
}

#[test]
fn shared_pattern_seed_generates_identical_material_aliases() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "hyperrealism-shared-pattern-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&temp).unwrap();
    for name in ["diffuse.png", "normal.png", "roughness.png"] {
        fs::copy(Path::new("test_assets").join(name), temp.join(name)).unwrap();
    }
    fs::write(
        temp.join("material_manifest.json"),
        r#"{
          "version":1,
          "materials":[{"id":"base","albedo":"diffuse.png","normal":"normal.png","roughness":"roughness.png"}],
          "textures":[
            {"name":"bamboo_fence_particle","material":"base","recipe":"bamboo","pattern_seed":"bamboo_particle","tint":[255,255,255],"target_color":[207,186,85]},
            {"name":"bamboo_fence_gate_particle","material":"base","recipe":"bamboo","pattern_seed":"bamboo_particle","tint":[255,255,255],"target_color":[207,186,85]}
          ]
        }"#,
    )
    .unwrap();
    let output = temp.join("output");
    let result = run(&temp, &output, "2");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let block = output.join("pack/assets/minecraft/textures/block");
    for suffix in ["", "_n", "_s"] {
        assert_eq!(
            fs::read(block.join(format!("bamboo_fence_particle{suffix}.png"))).unwrap(),
            fs::read(block.join(format!("bamboo_fence_gate_particle{suffix}.png"))).unwrap(),
            "shared vanilla sprite must have identical generated {suffix} maps"
        );
    }
    fs::remove_dir_all(temp).unwrap();
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

#[test]
fn packed_arm_uses_occlusion_and_roughness_not_metallic() {
    use hyperrealism_core::png_io::{read, write};
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp =
        std::env::temp_dir().join(format!("hyperrealism-arm-{}-{unique}", std::process::id()));
    fs::create_dir_all(&temp).unwrap();
    let mut albedo = Vec::new();
    let mut arm = Vec::new();
    for _ in 0..64 * 64 {
        albedo.extend_from_slice(&[128, 128, 128, 255]);
        arm.extend_from_slice(&[64, 144, 255, 255]);
    }
    write(&temp.join("color.png"), 64, 64, &albedo).unwrap();
    write(&temp.join("arm.png"), 64, 64, &arm).unwrap();
    let base = serde_json::json!({"version":1,"materials":[{"id":"base","albedo":"color.png","arm":"arm.png"}],"textures":[{"name":"stone","material":"base","recipe":"material","tint":[255,255,255],"f0":10}]});
    fs::write(
        temp.join("material_manifest.json"),
        serde_json::to_vec(&base).unwrap(),
    )
    .unwrap();
    let output = temp.join("out");
    let r = run(&temp, &output, "2");
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let n = read(&output.join("pack/assets/minecraft/textures/block/stone_n.png")).unwrap();
    let s = read(&output.join("pack/assets/minecraft/textures/block/stone_s.png")).unwrap();
    assert!(n.pixels.chunks_exact(4).all(|p| p[2] == 64));
    let smooth = ((1.0_f32 - (144.0_f32 / 255.0).sqrt()) * 255.0).round() as u8;
    assert!(s
        .pixels
        .chunks_exact(4)
        .all(|p| p[0] == smooth && p[1] == 10));
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("build_report.json")).unwrap()).unwrap();
    assert!(report["input_sha256"]["arm.png"].is_string());
    let mut invalid = base.clone();
    invalid["materials"][0]["roughness"] = "arm.png".into();
    fs::write(
        temp.join("material_manifest.json"),
        serde_json::to_vec(&invalid).unwrap(),
    )
    .unwrap();
    let failed = temp.join("invalid");
    assert!(!run(&temp, &failed, "2").status.success());
    assert!(!failed.exists());
    for (field, value) in [
        ("roughness_bias", serde_json::json!(0.75)),
        ("height_from_normal", serde_json::json!(true)),
        ("roughness_from_height", serde_json::json!(true)),
    ] {
        let mut invalid = base.clone();
        invalid["materials"][0][field] = value;
        fs::write(
            temp.join("material_manifest.json"),
            serde_json::to_vec(&invalid).unwrap(),
        )
        .unwrap();
        assert!(!run(&temp, &failed, "2").status.success(), "accepted {field}");
        assert!(!failed.exists());
    }
    let input16 = temp.join("arm16.png");
    {
        let file = fs::File::create(&input16).unwrap();
        let mut encoder = png::Encoder::new(file, 1, 1);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Sixteen);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[0x12, 0x34, 0xAB, 0xCD, 0xFF, 0xFE])
            .unwrap();
    }
    let (_, _, red) = hyperrealism_core::png_io::read_linear_channel(&input16, 0).unwrap();
    let (_, _, green) = hyperrealism_core::png_io::read_linear_channel(&input16, 1).unwrap();
    assert!((red[0] - 0x1234 as f32 / 65535.0).abs() < 1e-7);
    assert!((green[0] - 0xABCD as f32 / 65535.0).abs() < 1e-7);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn pack_verifier_detects_broken_or_invalid_pbr_maps() {
    use hyperrealism_core::png_io::{read, write};
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "hyperrealism-verify-{}-{unique}",
        std::process::id()
    ));
    let input = temp.join("input");
    fs::create_dir_all(&input).unwrap();
    fs::copy("test_assets/sample.jpg", input.join("albedo.jpg")).unwrap();
    fs::write(input.join("material_manifest.json"),r#"{"version":1,"materials":[{"id":"base","albedo":"albedo.jpg"}],"textures":[{"name":"stone","material":"base","recipe":"material","tint":[255,255,255]}]}"#).unwrap();
    let out = temp.join("out");
    let run_result = run(&input, &out, "2");
    assert!(
        run_result.status.success(),
        "{}",
        String::from_utf8_lossy(&run_result.stderr)
    );
    let pack = out.join("pack");
    assert_eq!(hyperrealism_core::verify::pack(&pack).unwrap(), 1);
    let check = Command::new(env!("CARGO_BIN_EXE_hyperrealism-core"))
        .arg("--verify-pack")
        .arg(&pack)
        .output()
        .unwrap();
    assert!(check.status.success());
    let block = pack.join("assets/minecraft/textures/block");
    let spec = block.join("stone_s.png");
    let original = read(&spec).unwrap();
    let mut bad = original.pixels.clone();
    bad[1] = 240;
    write(&spec, 128, 128, &bad).unwrap();
    assert!(hyperrealism_core::verify::pack(&pack)
        .unwrap_err()
        .contains("reserved LabPBR"));
    bad[1] = 230;
    bad[2] = 12;
    write(&spec, 128, 128, &bad).unwrap();
    assert!(hyperrealism_core::verify::pack(&pack)
        .unwrap_err()
        .contains("metal"));
    write(&spec, 128, 128, &original.pixels).unwrap();
    let normal = block.join("stone_n.png");
    let data = read(&normal).unwrap();
    let mut bad = data.pixels.clone();
    bad[3] = 0;
    write(&normal, 128, 128, &bad).unwrap();
    assert!(hyperrealism_core::verify::pack(&pack)
        .unwrap_err()
        .contains("zero height"));
    write(&normal, 128, 128, &data.pixels).unwrap();
    fs::remove_file(&spec).unwrap();
    assert!(hyperrealism_core::verify::pack(&pack).is_err());
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn grass_wood_and_wool_maps_keep_real_detail_and_biome_tint() {
    use hyperrealism_core::png_io::read;
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "hyperrealism-grass-wood-{}-{unique}",
        std::process::id()
    ));
    let input = temp.join("input");
    fs::create_dir_all(&input).unwrap();
    let all: serde_json::Value =
        serde_json::from_slice(&fs::read("work/material_manifest.json").unwrap()).unwrap();
    let materials: Vec<_> = all["materials"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| {
            m["id"] == "leafy_grass"
                || m["id"] == "oak_veneer_02"
                || m["id"] == "poly_wool_herringbone"
        })
        .cloned()
        .collect();
    let textures: Vec<_> = all["textures"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| {
            t["name"] == "grass_block_top"
                || t["name"] == "oak_planks"
                || t["name"] == "white_wool"
        })
        .cloned()
        .collect();
    for material in &materials {
        for key in ["albedo", "normal", "roughness", "ao"] {
            if let Some(name) = material[key].as_str() {
                let target = input.join(name);
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::copy(Path::new("work").join(name), target).unwrap();
            }
        }
    }
    fs::write(
        input.join("material_manifest.json"),
        serde_json::to_vec(&serde_json::json!({
            "version": 1, "materials": materials, "textures": textures
        }))
        .unwrap(),
    )
    .unwrap();
    let output = temp.join("output");
    let result = run(&input, &output, "2");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let block = output.join("pack/assets/minecraft/textures/block");
    let grass = read(&block.join("grass_block_top.png")).unwrap();
    assert!(grass
        .pixels
        .chunks_exact(4)
        .all(|p| p[0] == p[1] && p[1] == p[2] && p[3] == 255));
    // Do not replace the separate biome-tinted cutout with an opaque texture.
    assert!(!block.join("grass_block_side_overlay.png").exists());
    for name in ["grass_block_top", "oak_planks", "white_wool"] {
        let n = read(&block.join(format!("{name}_n.png"))).unwrap();
        let s = read(&block.join(format!("{name}_s.png"))).unwrap();
        let varied = |pixels: &[u8], channel| {
            let (mut lo, mut hi) = (255u8, 0u8);
            for p in pixels.chunks_exact(4) {
                lo = lo.min(p[channel]);
                hi = hi.max(p[channel]);
            }
            hi - lo > 3
        };
        assert!(
            varied(&n.pixels, 0) && varied(&n.pixels, 1),
            "normal detail missing: {name}"
        );
        assert!(varied(&s.pixels, 0), "roughness detail missing: {name}");
        if name == "oak_planks" {
            assert!(varied(&n.pixels, 2), "AO detail missing: {name}");
        }
    }
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn overworld_cutouts_and_scanned_terrain_are_generated() {
    use hyperrealism_core::png_io::read;
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!("overworld-{}-{unique}", std::process::id()));
    let input = temp.join("input");
    fs::create_dir_all(&input).unwrap();
    let all: serde_json::Value =
        serde_json::from_slice(&fs::read("work/material_manifest.json").unwrap()).unwrap();
    let materials: Vec<_> = all["materials"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| {
            [
                "tree_leaves",
                "grass_side_overlay",
                "procedural_glass",
                "brown_mud_dry",
                "snow_02",
            ]
            .iter()
            .any(|id| m["id"] == *id)
        })
        .cloned()
        .collect();
    let textures: Vec<_> = all["textures"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| {
            t["connected_glass"] == true
                || [
                    "oak_leaves",
                    "grass_block_side",
                    "grass_block_side_overlay",
                    "snow",
                    "blue_stained_glass_pane_top",
                ]
                .iter()
                .any(|name| t["name"] == *name)
        })
        .cloned()
        .collect();
    for mat in &materials {
        for key in ["albedo", "normal", "roughness", "arm", "height", "ao"] {
            if let Some(name) = mat[key].as_str() {
                let dest = input.join(name);
                fs::create_dir_all(dest.parent().unwrap()).unwrap();
                fs::copy(Path::new("work").join(name), dest).unwrap();
            }
        }
    }
    fs::write(
        input.join("material_manifest.json"),
        serde_json::to_vec(&serde_json::json!({
            "version": 1, "materials": materials, "textures": textures
        }))
        .unwrap(),
    )
    .unwrap();
    let output = temp.join("output");
    let result = run(&input, &output, "2");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let block = output.join("pack/assets/minecraft/textures/block");
    let side = read(&block.join("grass_block_side.png")).unwrap();
    assert!(side.pixels.chunks_exact(4).all(|p| p[3] == 255));
    for name in ["oak_leaves", "grass_block_side_overlay"] {
        let color = read(&block.join(format!("{name}.png"))).unwrap();
        let opaque = color.pixels.chunks_exact(4).filter(|p| p[3] >= 128).count();
        assert!(
            opaque > 100 && opaque < color.width * color.height - 100,
            "cutout alpha missing: {name}"
        );
        assert!(
            color
                .pixels
                .chunks_exact(4)
                .filter(|p| p[3] > 220)
                .all(|p| p[0] == p[1] && p[1] == p[2]),
            "biome tint base not neutral: {name}"
        );
        let normal = read(&block.join(format!("{name}_n.png"))).unwrap();
        let spec = read(&block.join(format!("{name}_s.png"))).unwrap();
        assert!(normal
            .pixels
            .chunks_exact(4)
            .any(|p| p[0] != 128 || p[1] != 128));
        assert!(spec.pixels.chunks_exact(4).any(|p| p[0] != 0));
        if name == "oak_leaves" {
            assert!(spec.pixels.chunks_exact(4).all(|p| p[2] == 128));
        }
    }
    let meta: serde_json::Value =
        serde_json::from_slice(&fs::read(block.join("oak_leaves.png.mcmeta")).unwrap()).unwrap();
    assert_eq!(meta["texture"]["mipmap_strategy"], "dark_cutout");
    let clear = read(&block.join("glass.png")).unwrap();
    let stained = read(&block.join("blue_stained_glass.png")).unwrap();
    let pane = read(&block.join("blue_stained_glass_pane_top.png")).unwrap();
    let alpha = |image: &hyperrealism_core::png_io::Rgba, x: usize, y: usize| {
        image.pixels[(y * image.width + x) * 4 + 3]
    };
    let clear_body = alpha(&clear, 64, 64);
    assert!((20..50).contains(&clear_body));
    assert_eq!(alpha(&clear, 0, 64), clear_body);
    let stained_body = alpha(&stained, 64, 64);
    assert!((90..120).contains(&stained_body));
    assert_eq!(alpha(&stained, 0, 64), stained_body);
    assert_eq!(alpha(&pane, 5, 64), 0);
    assert!(alpha(&pane, 64, 64) > 200);
    let glass_meta: serde_json::Value =
        serde_json::from_slice(&fs::read(block.join("glass.png.mcmeta")).unwrap()).unwrap();
    assert_eq!(glass_meta["texture"]["mipmap_strategy"], "mean");
    // Continuity's OptiFine-compatible compact loader consumes properties in
    // assets/minecraft/optifine/ctm. Verify every covered clear/stained/tinted
    // glass sprite has all five aligned LabPBR tiles, plus pane side rules.
    let ctm = output.join("pack/assets/minecraft/optifine/ctm");
    let connected: Vec<_> = textures
        .iter()
        .filter(|t| t["connected_glass"] == true)
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(connected.len(), 18);
    for name in connected {
        let dir = ctm.join(name);
        let properties = fs::read_to_string(dir.join("glass.properties")).unwrap();
        assert!(properties.contains("method=ctm_compact"));
        assert!(properties.contains("tiles=0-4"));
        assert!(properties.contains("connect=block"));
        assert!(properties.contains(&format!("matchTiles={name}")));
        assert!(properties.contains(&format!("matchBlocks={name}")));
        if name != "tinted_glass" {
            let pane_rule = fs::read_to_string(dir.join("pane.properties")).unwrap();
            assert!(pane_rule.contains(&format!("matchBlocks={name}_pane")));
            assert!(pane_rule.contains(&format!("matchTiles={name}")));
            assert!(pane_rule.contains("faces=sides"));
        } else {
            assert!(!dir.join("pane.properties").exists());
        }
        for i in 0..5 {
            for suffix in ["", "_n", "_s"] {
                let image = read(&dir.join(format!("{i}{suffix}.png"))).unwrap();
                assert_eq!((image.width, image.height), (128, 128));
                if suffix.is_empty() {
                    assert_eq!(
                        alpha(&image, 0, 64),
                        alpha(&image, 64, 64),
                        "CTM rim unexpectedly varies for {name}, tile {i}"
                    );
                }
            }
        }
        if name == "glass" {
            for i in 0..5 {
                let tile = read(&dir.join(format!("{i}.png"))).unwrap();
                let body = alpha(&tile, 64, 64);
                assert!((20..50).contains(&body));
                assert_eq!(alpha(&tile, 0, 64), body);
            }
        }
        if name == "blue_stained_glass" {
            let tile = read(&dir.join("0.png")).unwrap();
            let body = alpha(&tile, 64, 64);
            assert!((90..120).contains(&body));
            assert_eq!(alpha(&tile, 0, 64), body);
        }
        let gloss = read(&dir.join("1_s.png")).unwrap();
        assert!(gloss.pixels.chunks_exact(4).all(|p| p[0] > 0 && p[1] == 10));
    }
    assert!(!ctm.join("blue_stained_glass_pane_top").exists());
    let zip = fs::File::open(output.join("dist/HyperRealism128-v1.0-MC26.3.zip")).unwrap();
    let mut zip = zip::ZipArchive::new(zip).unwrap();
    assert!(zip
        .by_name("assets/minecraft/optifine/ctm/glass/1_s.png")
        .is_ok());
    drop(zip);
    let snow_normal = read(&block.join("snow_n.png")).unwrap();
    assert!(
        snow_normal.pixels.chunks_exact(4).map(|p| p[3]).min()
            != snow_normal.pixels.chunks_exact(4).map(|p| p[3]).max()
    );
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn pbr_strength_softens_relief_ao_and_gloss_without_changing_color_or_ids() {
    use hyperrealism_core::png_io::{read, write};
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!("pbr-strength-{}-{unique}", std::process::id()));
    fs::create_dir_all(&temp).unwrap();
    let image = |name: &str, pixel: [u8; 4]| {
        let mut data = Vec::new();
        for _ in 0..64 * 64 {
            data.extend_from_slice(&pixel);
        }
        write(&temp.join(name), 64, 64, &data).unwrap();
    };
    image("color.png", [120, 150, 180, 255]);
    image("normal.png", [128, 192, 238, 255]);
    image("height.png", [64, 64, 64, 255]);
    image("ao.png", [153, 153, 153, 255]);
    image("rough.png", [64, 64, 64, 255]);
    let mut m = serde_json::json!({"version":1,
        "materials":[{"id":"base","albedo":"color.png","normal":"normal.png","height":"height.png","ao":"ao.png","roughness":"rough.png"}],
        "textures":[{"name":"gold_block","material":"base","recipe":"material","tint":[255,255,255],"f0":231}]
    });
    for (name, strength) in [("full", 1.0), ("soft", 0.4)] {
        m["pbr_strength"] = strength.into();
        fs::write(
            temp.join("material_manifest.json"),
            serde_json::to_vec(&m).unwrap(),
        )
        .unwrap();
        let output = temp.join(name);
        let result = run(&temp, &output, "2");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let pixel = |which: &str, suffix: &str| {
        let p = temp.join(which).join(format!(
            "pack/assets/minecraft/textures/block/gold_block{suffix}.png"
        ));
        let image = read(&p).unwrap();
        image.pixels[..4].to_vec()
    };
    assert_eq!(pixel("full", ""), pixel("soft", ""));
    let (full, soft) = (pixel("full", "_n"), pixel("soft", "_n"));
    let slope = |p: &[u8]| {
        let x = p[0] as f32 / 127.5 - 1.0;
        let y = p[1] as f32 / 127.5 - 1.0;
        y / (1.0 - x * x - y * y).max(0.0001).sqrt()
    };
    assert!((slope(&soft) / slope(&full) - 0.4).abs() < 0.03);
    assert!((i32::from(soft[3]) - 102).abs() <= 1);
    assert!((i32::from(soft[2]) - 214).abs() <= 1);
    let (full_s, soft_s) = (pixel("full", "_s"), pixel("soft", "_s"));
    assert!((i32::from(soft_s[0]) - (f32::from(full_s[0]) * 0.4).round() as i32).abs() <= 1);
    assert_eq!(&full_s[1..], &soft_s[1..]);
    // Per-texture overrides are forbidden: every block must use the same
    // manifest-wide strength, so adding this unknown field must be rejected.
    m["pbr_strength"] = 0.4.into();
    m["textures"][0]["pbr_strength"] = 0.65.into();
    fs::write(
        temp.join("material_manifest.json"),
        serde_json::to_vec(&m).unwrap(),
    )
    .unwrap();
    assert!(!run(&temp, &temp.join("override"), "2").status.success());
    m["textures"][0]
        .as_object_mut()
        .unwrap()
        .remove("pbr_strength");
    for invalid in [-0.1, 1.1] {
        m["pbr_strength"] = invalid.into();
        fs::write(
            temp.join("material_manifest.json"),
            serde_json::to_vec(&m).unwrap(),
        )
        .unwrap();
        assert!(!run(&temp, &temp.join("invalid"), "2").status.success());
    }
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn rotated_wood_keeps_all_maps_and_normal_vectors_aligned() {
    use hyperrealism_core::material::{load, WORK};
    let root = Path::new("work");
    let manifest = hyperrealism_core::manifest::read(&root.join("material_manifest.json")).unwrap();
    let original = manifest
        .materials
        .iter()
        .find(|m| m.id == "oak_veneer_02")
        .unwrap();
    let rotated = manifest
        .materials
        .iter()
        .find(|m| m.id == "oak_veneer_02_vertical")
        .unwrap();
    let a = load(root, original).unwrap();
    let b = load(root, rotated).unwrap();
    for &(x, y) in &[(0, 0), (17, 111), (511, 511), (256, 309)] {
        let dst = y * WORK + x;
        let src = (WORK - 1 - x) * WORK + y;
        for c in 0..4 {
            assert!((b.albedo[c][dst] - a.albedo[c][src]).abs() < 1e-6);
        }
        assert!(
            (b.roughness.as_ref().unwrap()[dst] - a.roughness.as_ref().unwrap()[src]).abs() < 1e-6
        );
        assert!((b.ao.as_ref().unwrap()[dst] - a.ao.as_ref().unwrap()[src]).abs() < 1e-6);
        let a = a.normal.as_ref().unwrap();
        let b = b.normal.as_ref().unwrap();
        assert!((b[0][dst] + a[1][src]).abs() < 1e-6);
        assert!((b[1][dst] - a[0][src]).abs() < 1e-6);
        assert!((b[2][dst] - a[2][src]).abs() < 1e-6);
    }
}

#[test]
fn shaped_block_layouts_preserve_cutouts_hardware_and_pbr() {
    use hyperrealism_core::png_io::read;
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!("shaped-{}-{unique}", std::process::id()));
    let input = temp.join("input");
    fs::create_dir_all(&input).unwrap();
    let all: serde_json::Value =
        serde_json::from_slice(&fs::read("work/material_manifest.json").unwrap()).unwrap();
    let names = [
        "oak_door_top",
        "oak_door_bottom",
        "pale_oak_door_bottom",
        "oak_trapdoor",
        "copper_door_top",
        "copper_door_bottom",
        "lever",
        "ladder",
        "iron_bars",
    ];
    let textures: Vec<_> = all["textures"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| names.iter().any(|n| t["name"] == *n))
        .cloned()
        .collect();
    let materials: Vec<_> = all["materials"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| textures.iter().any(|t| t["material"] == m["id"]))
        .cloned()
        .collect();
    for m in &materials {
        for key in ["albedo", "normal", "roughness", "arm", "height", "ao"] {
            if let Some(path) = m[key].as_str() {
                let dest = input.join(path);
                fs::create_dir_all(dest.parent().unwrap()).unwrap();
                fs::copy(Path::new("work").join(path), dest).unwrap();
            }
        }
    }
    for t in &textures {
        let path = t["layout"].as_str().unwrap();
        let dest = input.join(path);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::copy(Path::new("work").join(path), dest).unwrap();
    }
    let manifest = serde_json::json!({"version":1,"pbr_strength":0.4,"materials":materials,"textures":textures});
    fs::write(
        input.join("material_manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let output = temp.join("output");
    let result = run(&input, &output, "2");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let block = output.join("pack/assets/minecraft/textures/block");
    let at = |name: &str, suffix: &str, x: usize, y: usize| -> Vec<u8> {
        let im = read(&block.join(format!("{name}{suffix}.png"))).unwrap();
        im.pixels[(y * im.width + x) * 4..(y * im.width + x) * 4 + 4].to_vec()
    };
    // Holes are really transparent; rails/panels remain opaque, not black glass.
    assert!(at("oak_door_top", "", 32, 32)[3] < 5);
    assert!(at("oak_door_top", "", 64, 32)[3] > 245);
    assert!(at("oak_door_bottom", "", 32, 32)[3] > 245);
    assert!(at("oak_trapdoor", "", 32, 32)[3] < 5);
    assert!(at("ladder", "", 64, 64)[3] < 5);
    assert!(at("ladder", "", 20, 64)[3] > 245);
    assert!(at("iron_bars", "", 40, 60)[3] < 5);
    assert!(at("iron_bars", "", 20, 60)[3] > 245);
    assert!(at("copper_door_top", "", 48, 32)[3] < 5);
    // The vanilla lever model's 2x10 UV island remains transparent elsewhere;
    // its wooden grip and iron collar receive aligned, distinct LabPBR values.
    assert!(at("lever", "", 64, 60)[3] > 245);
    assert!(at("lever", "", 40, 60)[3] < 5);
    assert_eq!(at("lever", "_s", 64, 74)[1], 230);
    assert_eq!(at("lever", "_s", 64, 100)[1], 10);
    let lever_normal = read(&block.join("lever_n.png")).unwrap();
    assert!(lever_normal
        .pixels
        .chunks_exact(4)
        .any(|p| p[0] != 128 || p[1] != 128));
    // Wood keeps dielectric F0; the edge-aligned knob becomes a discrete iron
    // ID. Its old, three-pixel-inboard location is intentionally plain wood.
    assert_eq!(at("oak_door_bottom", "_s", 5, 28)[1], 230);
    assert_eq!(at("oak_door_bottom", "_s", 25, 28)[1], 10);
    assert_eq!(at("oak_door_bottom", "_s", 5, 82)[1], 10);
    assert_eq!(at("pale_oak_door_bottom", "_s", 5, 28)[1], 230);
    assert_eq!(at("pale_oak_door_bottom", "_s", 25, 28)[1], 10);
    assert_eq!(at("copper_door_bottom", "_s", 5, 28)[1], 230);
    assert_eq!(at("oak_door_bottom", "_s", 64, 64)[1], 10);
    assert_eq!(at("iron_bars", "_s", 20, 60)[1], 230);
    assert_eq!(at("copper_door_top", "_s", 16, 40)[1], 234);
    let normal = read(&block.join("oak_door_bottom_n.png")).unwrap();
    assert!(
        normal.pixels.chunks_exact(4).map(|p| p[3]).min()
            != normal.pixels.chunks_exact(4).map(|p| p[3]).max()
    );
    assert!(hyperrealism_core::verify::pack(&output.join("pack")).is_ok());
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn malformed_construction_layout_is_rejected() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!("bad-layout-{}-{unique}", std::process::id()));
    fs::create_dir_all(&temp).unwrap();
    fs::copy("test_assets/diffuse.png", temp.join("diffuse.png")).unwrap();
    hyperrealism_core::png_io::write(&temp.join("bad-layout.png"), 2, 2, &[128; 16]).unwrap();
    fs::write(temp.join("material_manifest.json"), r#"{"version":1,"materials":[{"id":"base","albedo":"diffuse.png"}],"textures":[{"name":"door_top","material":"base","recipe":"material","tint":[255,255,255],"layout":"bad-layout.png","layout_metal_id":230}]}"#).unwrap();
    let result = run(&temp, &temp.join("output"), "2");
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("construction layout"));
    fs::remove_dir_all(temp).unwrap();
}
