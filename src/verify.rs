//! Structural checks for generated 26.3 LabPBR packs. Passing these checks
//! does not substitute for visual or shader-specific testing.
use crate::png_io;
use std::{collections::BTreeSet, fs, path::Path};

fn rgba_at(path: &Path, size: usize) -> Result<png_io::Rgba, String> {
    let file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let reader = png::Decoder::new(file)
        .read_info()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let info = reader.info();
    if (info.width, info.height, info.color_type, info.bit_depth)
        != (
            size as u32,
            size as u32,
            png::ColorType::Rgba,
            png::BitDepth::Eight,
        )
    {
        return Err(format!(
            "{}: expected {size}x{size} 8-bit RGBA PNG",
            path.display()
        ));
    }
    png_io::read(path)
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
    if (size != 128 && size != 512) || first.height != size {
        return Err("pack resolution must be 128x128 or 512x512".into());
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
