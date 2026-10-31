//! Compact connected glass in the OptiFine-compatible `optifine/ctm` path.
//! Continuity 3.0.1+26.3 source scans this path and registers `ctm_compact`
//! with a five-tile minimum; see `docs/CONTINUITY_SUPPORT.md`. Five tiles are
//! sliced into quadrants by the CTM renderer. Ordinary fallback textures remain
//! available to clients without a connected-texture implementation.
use crate::{manifest::Texture, png_io};
use std::{fs, path::Path};

/// Write complete albedo/normal/specular triples next to the CTM properties.
/// Tile colors come from the already-generated fallback, preserving its palette.
pub fn write(
    root: &Path,
    texture: &Texture,
    size: usize,
    albedo: &[u8],
    normal: &[u8],
    spec: &[u8],
) -> Result<(), String> {
    let dir = root
        .join("assets/minecraft/optifine/ctm")
        .join(&texture.name);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // Separate block and pane rules mirror the Continuity default pack: a
    // pane only connects to the same pane block and never targets pane caps.
    let properties = |block: &str, faces: &str| {
        format!("# Requires an OptiFine-compatible connected-texture mod.\nmatchTiles={}\nmatchBlocks={}\nmethod=ctm_compact\nconnect=block\n{}tiles=0-4\n", texture.name, block, faces)
    };
    fs::write(dir.join("glass.properties"), properties(&texture.name, ""))
        .map_err(|e| e.to_string())?;
    if texture.name != "tinted_glass" {
        fs::write(
            dir.join("pane.properties"),
            properties(&format!("{}_pane", texture.name), "faces=sides\n"),
        )
        .map_err(|e| e.to_string())?;
    }
    let sample = ((size / 2) * size + size / 2) * 4;
    let color = &albedo[sample..sample + 3];
    let alpha = albedo[sample + 3];
    let n = &normal[sample..sample + 4];
    let s = &spec[sample..sample + 4];
    // Keep the compact tile set for compatibility, but repeat one uniform
    // color/alpha on every tile so neither block edges nor seams gain a border.
    for tile in 0..5 {
        let mut a = Vec::with_capacity(size * size * 4);
        let mut nm = Vec::with_capacity(size * size * 4);
        let mut sm = Vec::with_capacity(size * size * 4);
        for _ in 0..size * size {
            a.extend_from_slice(color);
            a.push(alpha);
            nm.extend_from_slice(n);
            sm.extend_from_slice(s);
        }
        for (suffix, bytes) in [("", &a), ("_n", &nm), ("_s", &sm)] {
            png_io::write(&dir.join(format!("{tile}{suffix}.png")), size, size, bytes)?;
        }
    }
    Ok(())
}
