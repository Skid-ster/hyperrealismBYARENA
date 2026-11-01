//! Structural checks for generated 26.3 LabPBR packs. Passing these checks
//! does not substitute for visual or shader-specific testing.
use crate::png_io;
use std::{collections::BTreeSet, fs, path::Path};

fn rgba_dimensions(path: &Path, width: usize, height: usize) -> Result<png_io::Rgba, String> {
    let file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let reader = png::Decoder::new(file)
        .read_info()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let info = reader.info();
    if (info.width, info.height, info.color_type, info.bit_depth)
        != (
            width as u32,
            height as u32,
            png::ColorType::Rgba,
            png::BitDepth::Eight,
        )
    {
        return Err(format!(
            "{}: expected {width}x{height} 8-bit RGBA PNG",
            path.display()
        ));
    }
    png_io::read(path)
}

fn rgba_at(path: &Path, size: usize) -> Result<png_io::Rgba, String> {
    rgba_dimensions(path, size, size)
}

/// Verify the unpacked generated pack, returning the number of base textures.
pub fn pack(root: &Path) -> Result<usize, String> {
    let meta_path = root.join("pack.mcmeta");
    let bytes = fs::read(&meta_path).map_err(|e| format!("{}: {e}", meta_path.display()))?;
    let meta: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", meta_path.display()))?;
    if meta["pack"]["min_format"] != serde_json::json!([97, 1])
        || meta["pack"]["max_format"] != serde_json::json!(97)
    {
        return Err("pack.mcmeta: expected 26.3-only format [97,1] through 97".into());
    }
    let properties = root.join("assets/minecraft/optifine/texture.properties");
    let properties_text =
        fs::read_to_string(&properties).map_err(|e| format!("{}: {e}", properties.display()))?;
    if !properties_text
        .lines()
        .any(|line| line.trim() == "format=lab-pbr/1.3")
    {
        return Err(format!(
            "{}: missing LabPBR 1.3 declaration",
            properties.display()
        ));
    }
    let block = root.join("assets/minecraft/textures/block");
    let mut files = BTreeSet::new();
    for entry in fs::read_dir(&block).map_err(|e| format!("{}: {e}", block.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
            continue;
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "non-UTF8 texture name")?;
        if let Some(stem) = name.strip_suffix(".png") {
            files.insert(stem.to_owned());
        }
    }
    let base: Vec<_> = files
        .iter()
        .filter(|name| !name.ends_with("_n") && !name.ends_with("_s"))
        .collect();
    if base.is_empty() {
        return Err("pack has no block textures".into());
    }
    let first_path = block.join(format!("{}.png", base[0]));
    let first = png_io::read(&first_path)?;
    let size = first.width;
    if (size != 128 && size != 256 && size != 512) || first.height != size {
        return Err("pack resolution must be 128x128, 256x256, or 512x512".into());
    }
    drop(first);
    for name in &base {
        let color = block.join(format!("{name}.png"));
        rgba_at(&color, size)?;
        let normal_path = block.join(format!("{name}_n.png"));
        let normal = rgba_at(&normal_path, size)?;
        for pixel in normal.pixels.chunks_exact(4) {
            let x = f32::from(pixel[0]) / 127.5 - 1.0;
            let y = f32::from(pixel[1]) / 127.5 - 1.0;
            if x * x + y * y > 1.02 {
                return Err(format!("{}: invalid normal XY", normal_path.display()));
            }
            if pixel[3] == 0 {
                return Err(format!(
                    "{}: zero height can break POM",
                    normal_path.display()
                ));
            }
        }
        let spec_path = block.join(format!("{name}_s.png"));
        let spec = rgba_at(&spec_path, size)?;
        for p in spec.pixels.chunks_exact(4) {
            if (238..=254).contains(&p[1]) {
                return Err(format!(
                    "{}: reserved LabPBR material ID {}",
                    spec_path.display(),
                    p[1]
                ));
            }
            if p[1] >= 230 && p[2] != 0 {
                return Err(format!(
                    "{}: metal has nonzero reserved blue channel",
                    spec_path.display()
                ));
            }
        }
    }
    for name in &files {
        if let Some(base_name) = name.strip_suffix("_n").or_else(|| name.strip_suffix("_s")) {
            if !files.contains(base_name) {
                return Err(format!("{name}.png has no matching albedo"));
            }
        }
    }
    Ok(base.len())
}

fn verify_entity_triple(texture_root: &Path, stem: &str) -> Result<(), String> {
    let color_path = texture_root.join(format!("{stem}.png"));
    let color = png_io::read(&color_path)?;
    let width = color.width;
    let height = color.height;
    let supported_size = [128, 256, 512].contains(&width);
    let supported_aspect = height == width
        || height.saturating_mul(2) == width
        || width.saturating_mul(2) == height;
    if !supported_size || !supported_aspect {
        return Err(format!(
            "{}: entity atlas dimensions do not preserve a supported vanilla UV aspect ratio",
            color_path.display()
        ));
    }
    let color = rgba_dimensions(&color_path, width, height)?;
    let normal_path = texture_root.join(format!("{stem}_n.png"));
    let specular_path = texture_root.join(format!("{stem}_s.png"));
    let normal = rgba_dimensions(&normal_path, color.width, color.height)?;
    let specular = rgba_dimensions(&specular_path, color.width, color.height)?;
    for pixel in normal.pixels.chunks_exact(4) {
        let x = f32::from(pixel[0]) / 127.5 - 1.0;
        let y = f32::from(pixel[1]) / 127.5 - 1.0;
        if x * x + y * y > 1.02 {
            return Err(format!(
                "{}: invalid LabPBR entity normal XY",
                normal_path.display()
            ));
        }
        if pixel[3] == 0 {
            return Err(format!(
                "{}: zero LabPBR entity height",
                normal_path.display()
            ));
        }
    }
    for pixel in specular.pixels.chunks_exact(4) {
        if (238..=254).contains(&pixel[1]) {
            return Err(format!(
                "{}: reserved LabPBR material ID",
                specular_path.display()
            ));
        }
        if pixel[1] >= 230 && pixel[2] != 0 {
            return Err(format!(
                "{}: metal has nonzero reserved blue channel",
                specular_path.display()
            ));
        }
    }
    Ok(())
}

fn collect_entity_pngs(
    directory: &Path,
    root: &Path,
    stems: &mut BTreeSet<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|e| format!("{}: {e}", directory.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if file_type.is_dir() {
            collect_entity_pngs(&path, root, stems)?;
        } else if file_type.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("png")
        {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| format!("{}: {e}", path.display()))?
                .with_extension("");
            let stem = relative
                .to_str()
                .ok_or_else(|| format!("non-UTF8 entity texture path: {}", path.display()))?
                .replace('\\', "/");
            stems.insert(stem);
        }
    }
    Ok(())
}

/// Verify all entity atlas triples in an unpacked pack without a manifest.
pub fn entity_pack(root: &Path) -> Result<usize, String> {
    let texture_root = root.join("assets/minecraft/textures/entity");
    if !texture_root.exists() {
        return Ok(0);
    }
    if !texture_root.is_dir() {
        return Err(format!(
            "{}: expected entity texture directory",
            texture_root.display()
        ));
    }
    let mut files = BTreeSet::new();
    collect_entity_pngs(&texture_root, &texture_root, &mut files)?;
    let bases: BTreeSet<_> = files
        .iter()
        .filter(|stem| !stem.ends_with("_n") && !stem.ends_with("_s"))
        .cloned()
        .collect();
    for stem in &files {
        if let Some(base) = stem.strip_suffix("_n").or_else(|| stem.strip_suffix("_s")) {
            if !bases.contains(base) {
                return Err(format!("entity texture {stem}.png has no matching albedo"));
            }
        }
    }
    for base in &bases {
        for suffix in ["_n", "_s"] {
            if !files.contains(&format!("{base}{suffix}")) {
                return Err(format!("entity texture {base}.png is missing {suffix} map"));
            }
        }
        verify_entity_triple(&texture_root, base)?;
    }
    Ok(bases.len())
}

/// Verify block and entity LabPBR maps in a generated pack.
pub fn full_pack(root: &Path) -> Result<(usize, usize), String> {
    let block_count = pack(root)?;
    let entity_count = entity_pack(root)?;
    Ok((block_count, entity_count))
}
