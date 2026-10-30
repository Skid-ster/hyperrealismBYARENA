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
    let pack_readme = fs::read_to_string(out1.join("pack/README.txt")).unwrap();
    assert!(pack_readme.contains("replaces 1 block textures"));
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
    // The new default is 512px; inspect its actual LabPBR values after a 1K
    // source is downsampled. Test DX and GL normals, AO, height and smoothness.
    let output512 = temp.join("output512");
    let result512 = Command::new(env!("CARGO_BIN_EXE_hyperrealism-core"))
        .arg("--input")
        .arg(&input)
        .arg("--output")
        .arg(&output512)
        .arg("--cpu-only")
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
    assert_eq!(hashes.len(), 87);
    let audit: serde_json::Value =
        serde_json::from_slice(&fs::read("catalog/vanilla_26.3_audit.json").unwrap()).unwrap();
    let entries = audit["textures"].as_object().unwrap();
    assert_eq!(entries.len(), m.textures.len());
    assert_eq!(m.textures.len(), 242);
    assert!((m.pbr_strength - 0.4).abs() < 1e-6);
    assert!(m.textures.iter().all(|t| !t.name.starts_with("water_")));
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
    let mut invalid = base;
    invalid["materials"][0]["roughness"] = "arm.png".into();
    fs::write(
        temp.join("material_manifest.json"),
        serde_json::to_vec(&invalid).unwrap(),
    )
    .unwrap();
    let failed = temp.join("invalid");
    assert!(!run(&temp, &failed, "2").status.success());
    assert!(!failed.exists());
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
fn grass_and_wood_preview_maps_keep_real_detail_and_biome_tint() {
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
        .filter(|m| m["id"] == "leafy_grass" || m["id"] == "oak_veneer_02")
        .cloned()
        .collect();
    let textures: Vec<_> = all["textures"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| t["name"] == "grass_block_top" || t["name"] == "oak_planks")
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
    for name in ["grass_block_top", "oak_planks"] {
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
            [
                "oak_leaves",
                "grass_block_side",
                "grass_block_side_overlay",
                "snow",
                "glass",
                "blue_stained_glass",
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
    assert_eq!(alpha(&clear, 64, 64), 0);
    assert!(alpha(&clear, 0, 64) > 200);
    assert!((90..120).contains(&alpha(&stained, 64, 64)));
    assert_eq!(alpha(&pane, 5, 64), 0);
    assert!(alpha(&pane, 64, 64) > 200);
    let glass_meta: serde_json::Value =
        serde_json::from_slice(&fs::read(block.join("glass.png.mcmeta")).unwrap()).unwrap();
    assert_eq!(glass_meta["texture"]["mipmap_strategy"], "mean");
    // CTM tiles and maps must be packaged, not just listed in properties.
    let ctm = output.join("pack/assets/minecraft/optifine/ctm");
    for name in ["glass", "blue_stained_glass"] {
        let dir = ctm.join(name);
        let properties = fs::read_to_string(dir.join("glass.properties")).unwrap();
        assert!(properties.contains("method=ctm_compact"));
        assert!(properties.contains(&format!("matchTiles={name}")));
        assert!(properties.contains(&format!("matchBlocks={name}")));
        let pane_rule = fs::read_to_string(dir.join("pane.properties")).unwrap();
        assert!(pane_rule.contains(&format!("matchBlocks={name}_pane")));
        assert!(pane_rule.contains("faces=sides"));
        for i in 0..5 {
            for suffix in ["", "_n", "_s"] {
                let image = read(&dir.join(format!("{i}{suffix}.png"))).unwrap();
                assert_eq!((image.width, image.height), (128, 128));
            }
        }
        let frame = read(&dir.join("0.png")).unwrap();
        let interior = read(&dir.join("1.png")).unwrap();
        if name == "glass" {
            assert_eq!(alpha(&frame, 0, 64), 255);
            assert_eq!(alpha(&frame, 64, 64), 0);
            assert_eq!(alpha(&interior, 0, 64), 0);
        } else {
            assert_eq!(alpha(&frame, 0, 64), 153);
            assert_eq!(alpha(&interior, 64, 64), 102);
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
    // A texture-level override must affect only its PBR channels.
    m["pbr_strength"] = 0.4.into();
    m["textures"][0]["pbr_strength"] = 0.65.into();
    fs::write(
        temp.join("material_manifest.json"),
        serde_json::to_vec(&m).unwrap(),
    )
    .unwrap();
    let result = run(&temp, &temp.join("override"), "2");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(pixel("override", ""), pixel("soft", ""));
    let over = pixel("override", "_n");
    assert!(slope(&over) > slope(&soft) && slope(&over) < slope(&full));
    let over_s = pixel("override", "_s");
    assert!(over_s[0] > soft_s[0] && over_s[0] < full_s[0]);
    assert_eq!(&over_s[1..], &soft_s[1..]);
    m["textures"][0]["pbr_strength"] = serde_json::Value::Null;
    for invalid in [-0.1, 1.1] {
        m["pbr_strength"] = invalid.into();
        fs::write(
            temp.join("material_manifest.json"),
            serde_json::to_vec(&m).unwrap(),
        )
        .unwrap();
        assert!(!run(&temp, &temp.join("invalid"), "2").status.success());
    }
    m["pbr_strength"] = 0.4.into();
    m["textures"][0]["pbr_strength"] = 1.1.into();
    fs::write(
        temp.join("material_manifest.json"),
        serde_json::to_vec(&m).unwrap(),
    )
    .unwrap();
    assert!(!run(&temp, &temp.join("invalid_override"), "2")
        .status
        .success());
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
