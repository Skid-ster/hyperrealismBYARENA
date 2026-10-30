//! Compact connected glass in the OptiFine-compatible `optifine/ctm` path.
//! Continuity 3.0.1+26.3 source scans this path and registers `ctm_compact`
//! with a five-tile minimum; see `docs/CONTINUITY_SUPPORT.md`. Five tiles are
//! sliced into quadrants by the CTM renderer. Ordinary fallback textures remain
//! available to clients without a connected-texture implementation.
use crate::{manifest::Texture, png_io};
use std::{fs, path::Path};

fn edge(tile: usize, x: usize, y: usize, size: usize, width: usize) -> bool {
    let horizontal = y < width || y >= size - width;
    let vertical = x < width || x >= size - width;
    match tile {
        0 => horizontal || vertical, // unconnected
        1 => false,                  // fully surrounded
        2 => vertical,               // horizontal connections
        3 => horizontal,             // vertical connections
        4 => horizontal && vertical, // diagonal gap corners
        _ => unreachable!(),
    }
}

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
    let sample = ((size / 2) * size) * 4;
    let color = &albedo[sample..sample + 3];
    let center = ((size / 2) * size + size / 2) * 4;
    let n = &normal[center..center + 4];
    let s = &spec[center..center + 4];
    let clear = texture.recipe == "glass_clear";
    let width = (size / 64).max(1);
    for tile in 0..5 {
        let mut a = Vec::with_capacity(size * size * 4);
        let mut nm = Vec::with_capacity(size * size * 4);
        let mut sm = Vec::with_capacity(size * size * 4);
        for y in 0..size {
            for x in 0..size {
                let frame = edge(tile, x, y, size, width);
                a.extend_from_slice(color);
                a.push(if clear {
                    if frame {
                        255
                    } else {
                        0
                    }
                } else if frame {
                    153
                } else {
                    102
                });
                nm.extend_from_slice(n);
                sm.extend_from_slice(s);
            }
        }
        for (suffix, bytes) in [("", &a), ("_n", &nm), ("_s", &sm)] {
            png_io::write(&dir.join(format!("{tile}{suffix}.png")), size, size, bytes)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::edge;
    #[test]
    fn compact_tiles_have_correct_edges() {
        let p = |t, x, y| edge(t, x, y, 128, 2);
        assert!(p(0, 0, 64) && p(0, 64, 0));
        assert!(!p(1, 0, 0));
        assert!(p(2, 0, 64) && !p(2, 64, 0));
        assert!(p(3, 64, 0) && !p(3, 0, 64));
        assert!(p(4, 0, 0) && !p(4, 0, 64));
    }
}
