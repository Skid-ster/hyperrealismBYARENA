//! Small, deterministic procedural overlays for periodic 512×512 materials.
//! Real material maps provide the photographic detail; these routines add
//! coarse construction and relief. They are not a complete texture catalog.
const PERIOD: usize = 512;

#[derive(Clone, Copy)]
pub struct Pattern {
    pub shade: f32,
    pub relief: f32,
    pub opacity: f32,
    pub moss: f32,
    /// Fractional ore inclusion coverage prior to resampling.
    pub ore: f32,
}

/// Stable FNV-1a seed; independent of thread scheduling and platform HashMap seeds.
pub fn seed(name: &str) -> u32 {
    name.bytes().fold(2166136261u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(16777619)
    })
}
fn hash(mut x: u32) -> f32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^= x >> 16;
    (x >> 8) as f32 / 16777215.0
}
fn point(x: usize, y: usize, cells: usize, seed: u32) -> f32 {
    hash(
        seed ^ ((x % cells) as u32).wrapping_mul(0x9e3779b9)
            ^ ((y % cells) as u32).wrapping_mul(0x85ebca6b),
    )
}
/// Periodic bilinear value noise, sampled with smoothstep interpolation.
fn noise(x: usize, y: usize, cells: usize, seed: u32) -> f32 {
    let size = PERIOD / cells;
    let ix = (x % PERIOD) / size;
    let iy = (y % PERIOD) / size;
    let fx = (x % size) as f32 / size as f32;
    let fy = (y % size) as f32 / size as f32;
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let a = lerp(
        point(ix, iy, cells, seed),
        point(ix + 1, iy, cells, seed),
        sx,
    );
    let b = lerp(
        point(ix, iy + 1, cells, seed),
        point(ix + 1, iy + 1, cells, seed),
        sx,
    );
    lerp(a, b, sy)
}
fn brick(x: usize, y: usize, seed: u32) -> (f32, f32) {
    let row = (y % PERIOD) / 128;
    let shifted = (x + (row % 2) * 64) % PERIOD;
    let column = shifted / 128;
    let joint = (y % 128) < 6 || (shifted % 128) < 6;
    let variation = point(column, row, 4, seed);
    if joint {
        (0.52, -0.15)
    } else {
        (0.91 + 0.17 * variation, 0.03 + 0.02 * variation)
    }
}

/// A periodic 8×8 jittered Voronoi diagram. Neighbor cells use wrapped hash
/// coordinates but unwrapped positions, so edge cells continue across tiles.
fn cobble(x: usize, y: usize, seed: u32) -> (f32, f32) {
    let cx = (x / 64) as i32;
    let cy = (y / 64) as i32;
    let mut first = f32::MAX;
    let mut second = f32::MAX;
    let mut variation = 0.5;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let gx = cx + dx;
            let gy = cy + dy;
            let wx = gx.rem_euclid(8) as usize;
            let wy = gy.rem_euclid(8) as usize;
            let px = gx as f32 * 64.0 + 12.0 + 40.0 * point(wx, wy, 8, seed);
            let py = gy as f32 * 64.0 + 12.0 + 40.0 * point(wx, wy, 8, seed ^ 0x84d3);
            let delta_x = x as f32 - px;
            let delta_y = y as f32 - py;
            let dist = delta_x * delta_x + delta_y * delta_y;
            if dist < first {
                second = first;
                first = dist;
                variation = point(wx, wy, 8, seed ^ 0xc932);
            } else if dist < second {
                second = dist;
            }
        }
    }
    if second.sqrt() - first.sqrt() < 4.0 {
        (0.55, -0.12)
    } else {
        (0.91 + 0.18 * variation, 0.025 + 0.02 * variation)
    }
}
fn cracked_brick(x: usize, y: usize, seed: u32) -> (f32, f32) {
    let (shade, relief) = brick(x, y, seed);
    let row = y / 128;
    let shifted = (x + (row % 2) * 64) % PERIOD;
    let column = shifted / 128;
    let local_x = (shifted % 128) as f32;
    let local_y = (y % 128) as f32;
    let choice = point(column, row, 4, seed ^ 0x6512);
    let center = 30.0
        + 60.0 * choice
        + 0.22 * (local_y - 64.0)
        + 4.0 * ((local_y / 18.0) + choice * 6.0).sin();
    let broken =
        choice > 0.35 && local_y > 14.0 && local_y < 110.0 && (local_x - center).abs() < 1.6;
    if broken && relief > 0.0 {
        (shade * 0.48, relief - 0.10)
    } else {
        (shade, relief)
    }
}

pub fn evaluate(recipe: &str, x: usize, y: usize, seed: u32) -> Pattern {
    let x = x % PERIOD;
    let y = y % PERIOD;
    let n = noise(x, y, 8, seed);
    let fine = noise(x, y, 32, seed ^ 0x7a39);
    let (shade, relief, opacity, moss) = match recipe {
        "material" => (1.0, 0.0, 1.0, 0.0),
        "brick" => {
            let (s, h) = brick(x, y, seed);
            (s, h, 1.0, 0.0)
        }
        "cracked_brick" => {
            let (s, h) = cracked_brick(x, y, seed);
            (s, h, 1.0, 0.0)
        }
        "cobble" => {
            let (s, h) = cobble(x, y, seed);
            (s, h, 1.0, 0.0)
        }
        "moss_cobble" => {
            let (s, h) = cobble(x, y, seed);
            let m = ((n - 0.36) * 2.0).clamp(0.0, 0.65);
            (s, h + m * 0.04, 1.0, m)
        }
        "moss_brick" => {
            let (s, h) = brick(x, y, seed);
            let m = ((n - 0.36) * 2.0).clamp(0.0, 0.65);
            (s, h + m * 0.04, 1.0, m)
        }
        "plank" => {
            let row = y / 128;
            let seam = y % 128 < 5;
            let segment = (x + (row % 2) * 128) % PERIOD;
            let end_joint = segment % 256 < 4;
            if seam || end_joint {
                (0.5, -0.12, 1.0, 0.0)
            } else {
                let variation = point(segment / 256, row, 4, seed);
                (0.91 + variation * 0.13, 0.02, 1.0, 0.0)
            }
        }
        "moss" => {
            let m = ((n - 0.32) * 1.8).clamp(0.0, 0.7);
            (1.0, m * 0.06, 1.0, m)
        }
        "glass" => {
            let edge = x % 128 < 5 || y % 128 < 5;
            if edge {
                (0.8, 0.03, 0.75, 0.0)
            } else {
                (1.0, 0.0, 0.25, 0.0)
            }
        }
        // Fine, periodic albedo grain prevents procedural ore deposits from
        // becoming uniform, flat-color patches.
        "ore" => (
            0.72 + 0.50 * noise(x, y, 128, seed ^ 0x9b21),
            0.008 * (fine - 0.5),
            1.0,
            0.0,
        ),
        "metal" => (0.97 + 0.04 * n, 0.003 * (fine - 0.5), 1.0, 0.0),
        "gem" => (0.90 + 0.16 * n, 0.03 * n, 1.0, 0.0),
        "leaves" => {
            let a = if (x / 16 + y / 16) % 7 == 0 && n < 0.6 {
                0.0
            } else {
                1.0
            };
            (0.92 + 0.08 * n, 0.03 * n, a, 0.0)
        }
        _ => (0.96 + 0.08 * n, 0.008 * (fine - 0.5), 1.0, 0.0),
    };
    // A pair of periodic scales creates scattered irregular mineral inclusions.
    // Coverage is continuous for color filtering; material IDs remain discrete.
    let ore = if recipe == "ore" {
        ((0.60 * n + 0.40 * fine - 0.62) * 15.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Pattern {
        shade,
        relief: relief + ore * 0.045,
        opacity,
        moss,
        ore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_and_periodic() {
        for r in [
            "stone",
            "ore",
            "brick",
            "cracked_brick",
            "cobble",
            "moss_cobble",
            "moss",
            "moss_brick",
            "plank",
            "glass",
            "metal",
            "gem",
            "leaves",
            "material",
        ] {
            for &(x, y) in &[(0, 0), (71, 27), (511, 511)] {
                let p = evaluate(r, x, y, seed("a"));
                let q = evaluate(r, x + 512, y + 512, seed("a"));
                assert_eq!(
                    (p.shade, p.relief, p.opacity, p.moss, p.ore),
                    (q.shade, q.relief, q.opacity, q.moss, q.ore)
                );
                assert!(
                    p.shade.is_finite()
                        && p.opacity >= 0.0
                        && p.opacity <= 1.0
                        && p.moss >= 0.0
                        && p.moss <= 1.0
                );
            }
        }
        assert_eq!(seed("stone"), seed("stone"));
        assert_ne!(seed("stone"), seed("bricks"));
    }
    #[test]
    fn geometry_has_joints_and_cracks() {
        let seed = seed("stone_bricks");
        let mut joints = 0;
        let mut interiors = 0;
        let mut cracks = 0;
        for y in 0..PERIOD {
            for x in 0..PERIOD {
                let p = evaluate("cobble", x, y, seed);
                if p.relief < 0.0 {
                    joints += 1;
                } else {
                    interiors += 1;
                }
                let plain = evaluate("brick", x, y, seed);
                let cracked = evaluate("cracked_brick", x, y, seed);
                if cracked.relief < plain.relief {
                    cracks += 1;
                }
            }
        }
        assert!(
            joints > 1000 && interiors > 1000,
            "Voronoi stone must have grout and interiors"
        );
        assert!(cracks > 50, "cracked bricks must contain visible marks");
    }
}
