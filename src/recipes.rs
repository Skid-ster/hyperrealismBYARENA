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
        "bamboo" => {
            // Bamboo-laminate boards: long subtle fibers, staggered board
            // joints and a restrained node band. The scan underneath supplies
            // fine photographic grain; this pattern adds broader identity.
            let row = y / 128;
            let local_y = (y % 128) as f32;
            let shifted_x = (x + (row % 2) * 128) % PERIOD;
            let local_x = (shifted_x % 256) as f32;
            let seam = y % 128 < 3 || shifted_x % 256 < 3;
            let node_center = 104.0 + 32.0 * point(shifted_x / 256, row, 4, seed ^ 0x71b3);
            let node = (1.0 - (local_x - node_center).abs() / 3.0).clamp(0.0, 1.0);
            let fiber = (local_y * std::f32::consts::TAU / 21.0
                + 0.35 * noise(x, y, 32, seed ^ 0x528d))
                .sin();
            if seam {
                (0.58, -0.075, 1.0, 0.0)
            } else {
                let shade = (0.99 + 0.025 * fiber + 0.02 * (fine - 0.5) - 0.09 * node)
                    .clamp(0.82, 1.08);
                let relief = 0.006 * fiber - 0.035 * node + 0.008 * (n - 0.5);
                (shade, relief, 1.0, 0.0)
            }
        }
        "log_end" | "stripped_log_end" => {
            // A wood cross section, not plank joints: subtly irregular annual
            // rings and a darker bark rim. The scan underneath supplies grain.
            let dx = (x as f32 - 256.0) / 256.0;
            let dy = (y as f32 - 256.0) / 256.0;
            let radius = (dx * dx + dy * dy).sqrt();
            let square_radius = dx.abs().max(dy.abs());
            let warped = radius + (n - 0.5) * 0.035;
            let ring = (warped * 46.0).sin().abs().powi(8);
            let bark = if recipe == "stripped_log_end" {
                0.0
            } else {
                ((square_radius - 0.78) * 12.0).clamp(0.0, 1.0)
            };
            let shade = (1.0 - 0.18 * ring) * (1.0 - 0.45 * bark);
            let relief = 0.015 * ring + 0.035 * bark * (fine - 0.5);
            (shade, relief, 1.0, 0.0)
        }
        "moss" => {
            let m = ((n - 0.32) * 1.8).clamp(0.0, 0.7);
            (1.0, m * 0.06, 1.0, m)
        }
        // Use a uniform translucent surface: no bright edge band or frame.
        // Clear glass stays faintly visible at 12% opacity; stained glass keeps
        // its established 40% translucent body opacity without a darker rim.
        "glass" | "glass_clear" => (1.0, 0.0, 0.12, 0.0),
        "glass_stained" => (1.0, 0.0, 0.40, 0.0),
        // Pane top/bottom sprites are only two vanilla pixels wide.
        "glass_pane_cap" => {
            let strip = (224..288).contains(&x);
            (1.0, 0.0, if strip { 1.0 } else { 0.0 }, 0.0)
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
        "metal_cut" => {
            // Four-by-four copper sheet panels with narrow recessed joints.
            // The 128px repeat maps to four Minecraft texels per panel at
            // 512px resolution; a 7px joint is deliberately finer than a full
            // source texel so the scan remains the dominant surface detail.
            let local_x = x % 128;
            let local_y = y % 128;
            let seam = local_x < 7 || local_y < 7;
            let bevel = !seam && (local_x < 12 || local_y < 12);
            if seam {
                (0.68, -0.055, 1.0, 0.0)
            } else if bevel {
                (0.91, -0.018, 1.0, 0.0)
            } else {
                let variation = point(x / 128, y / 128, 4, seed);
                let panel_noise = noise(x, y, 64, seed ^ 0xa417);
                let shade = 0.98 + 0.035 * variation + 0.012 * (panel_noise - 0.5);
                let relief = 0.004 * (fine - 0.5) + 0.0015 * (n - 0.5);
                (shade, relief, 1.0, 0.0)
            }
        }
        "carved" => {
            // Shallow hand-carved diamond bands add repeatable embossed
            // identity without copying the low-resolution vanilla glyph.
            let dx = (x as f32 + 0.5 - 256.0).abs();
            let dy = (y as f32 + 0.5 - 256.0).abs();
            let distance = dx + dy;
            let nearest = [42.0, 86.0, 130.0, 174.0, 218.0]
                .iter()
                .map(|radius| (distance - radius).abs())
                .fold(f32::MAX, f32::min);
            if nearest < 2.5 {
                (0.74, -0.045, 1.0, 0.0)
            } else if nearest < 6.5 {
                (0.90, -0.018, 1.0, 0.0)
            } else if nearest < 9.5 {
                (0.98, 0.006, 1.0, 0.0)
            } else {
                (0.99 + 0.018 * (fine - 0.5), 0.002 * (n - 0.5), 1.0, 0.0)
            }
        }
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
    fn glass_has_uniform_translucency_without_rim_bands() {
        let sample = |r, x, y| evaluate(r, x, y, 1).opacity;
        for &(x, y) in &[(0, 256), (3, 256), (4, 256), (256, 256), (511, 511)] {
            assert_eq!(sample("glass", x, y), 0.12);
            assert_eq!(sample("glass_clear", x, y), 0.12);
            assert_eq!(sample("glass_stained", x, y), 0.40);
        }
        // Pane-cap transparency is a required vanilla UV silhouette, not a
        // decorative rim, and remains unchanged.
        assert_eq!(sample("glass_pane_cap", 0, 256), 0.0);
        assert_eq!(sample("glass_pane_cap", 256, 256), 1.0);
    }
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
            "bamboo",
            "log_end",
            "stripped_log_end",
            "glass",
            "glass_clear",
            "glass_stained",
            "glass_pane_cap",
            "metal",
            "metal_cut",
            "carved",
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
    fn bamboo_recipe_has_periodic_board_joints_and_subtle_relief() {
        let seed = seed("bamboo_planks");
        let mut minimum_shade = f32::MAX;
        let mut maximum_shade = f32::MIN;
        let mut recessed = 0;
        let mut raised = 0;
        for y in (0..PERIOD).step_by(4) {
            for x in (0..PERIOD).step_by(4) {
                let p = evaluate("bamboo", x, y, seed);
                minimum_shade = minimum_shade.min(p.shade);
                maximum_shade = maximum_shade.max(p.shade);
                if p.relief < 0.0 {
                    recessed += 1;
                }
                if p.relief > 0.0 {
                    raised += 1;
                }
                assert_eq!(
                    (p.shade, p.relief),
                    (
                        evaluate("bamboo", x + PERIOD, y, seed).shade,
                        evaluate("bamboo", x, y + PERIOD, seed).relief
                    )
                );
            }
        }
        assert!(maximum_shade - minimum_shade > 0.1);
        assert!(recessed > 0 && raised > 0);
    }

    #[test]
    fn cut_copper_has_recessed_joints_and_periodic_panels() {
        let seed = seed("cut_copper");
        let joint = evaluate("metal_cut", 2, 64, seed);
        let bevel = evaluate("metal_cut", 9, 64, seed);
        let face = evaluate("metal_cut", 64, 64, seed);
        assert!(joint.shade < bevel.shade && bevel.shade < face.shade);
        assert!(joint.relief < bevel.relief);
        for &(x, y) in &[(2, 64), (9, 64), (64, 64), (127, 127)] {
            let p = evaluate("metal_cut", x, y, seed);
            let q = evaluate("metal_cut", x + PERIOD, y + PERIOD, seed);
            assert_eq!((p.shade, p.relief), (q.shade, q.relief));
        }
    }

    #[test]
    fn carved_recipe_adds_shallow_embossed_bands_and_repeats() {
        let seed = seed("chiseled_sandstone");
        let groove = evaluate("carved", 298, 256, seed);
        let bevel = evaluate("carved", 303, 256, seed);
        let face = evaluate("carved", 310, 256, seed);
        assert!(groove.shade < bevel.shade && bevel.shade < face.shade);
        assert!(groove.relief < bevel.relief && bevel.relief < face.relief);
        for &(x, y) in &[(0, 0), (298, 256), (303, 256), (511, 511)] {
            let p = evaluate("carved", x, y, seed);
            let q = evaluate("carved", x + PERIOD, y + PERIOD, seed);
            assert_eq!((p.shade, p.relief), (q.shade, q.relief));
        }
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
