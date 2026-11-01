use crate::{
    labpbr::{normal_pixel, normals_from_height, specular_pixel, F0},
    manifest::{self, Material, Texture},
    png_io::{self, Rgba},
    resample::resize_wrap,
    zip,
};
use rayon::prelude::*;
use std::{fs, path::{Path, PathBuf}, time::Instant};

const WORK: usize = 512;
#[derive(Clone, Debug)]
pub struct Options {
    pub resolution: usize,
    pub seed: u32,
    pub name: String,
    pub include_normal: bool,
    pub include_specular: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self { resolution: 128, seed: 0, name: "HyperRealism".into(), include_normal: true, include_specular: true }
    }
}
impl Options {
    fn validate(&self) -> Result<(), String> {
        if ![64, 128, 256].contains(&self.resolution) { return Err("resolution must be 64, 128, or 256".into()); }
        if self.name.trim().is_empty() || self.name.len() > 60 || self.name.chars().any(|c| c.is_control() || "<>:\"/\\|?*".contains(c)) {
            return Err("pack name must be 1..60 characters without path or control characters".into());
        }
        Ok(())
    }
    fn seed_for(&self, name: &str) -> u32 {
        name.bytes().fold(self.seed, |hash, byte| hash.wrapping_mul(16777619) ^ u32::from(byte))
    }
}
fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
}
fn linear_to_srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let x = if v <= 0.0031308 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 };
    (x * 255.0).round() as u8
}
fn unit(v: f32) -> u8 { (v.clamp(0.0, 1.0) * 255.0).round() as u8 }
struct Images { albedo: Rgba, rough: Option<Rgba>, normal: Option<Rgba> }
fn load(root: &Path, m: &Material) -> Result<Images, String> {
    let load_one = |s: &str| png_io::read(&manifest::safe_file(root, s)?);
    Ok(Images {
        albedo: load_one(&m.albedo).map_err(|e| format!("albedo: {e}"))?,
        rough: m.roughness.as_deref().map(load_one).transpose().map_err(|e| format!("roughness: {e}"))?,
        normal: m.normal.as_deref().map(load_one).transpose().map_err(|e| format!("normal: {e}"))?,
    })
}
/// Bilinear, wrap-aware source sampling. Data is decoded in its supplied color space.
fn sample(img: &Rgba, x: usize, y: usize) -> [f32; 4] {
    let fx = (x as f32 + 0.5) * img.width as f32 / WORK as f32 - 0.5;
    let fy = (y as f32 + 0.5) * img.height as f32 / WORK as f32 - 0.5;
    let ix = fx.floor() as i32; let iy = fy.floor() as i32;
    let tx = fx - fx.floor(); let ty = fy - fy.floor();
    let mut out = [0.0; 4];
    for (dy, wy) in [(0, 1.0 - ty), (1, ty)] {
        for (dx, wx) in [(0, 1.0 - tx), (1, tx)] {
            let px = (ix + dx).rem_euclid(img.width as i32) as usize;
            let py = (iy + dy).rem_euclid(img.height as i32) as usize;
            let i = (py * img.width + px) * 4;
            for c in 0..4 { out[c] += img.pixels[i + c] as f32 / 255.0 * wx * wy; }
        }
    }
    out
}
fn hash(x: usize, y: usize, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393).wrapping_add((y as u32).wrapping_mul(668265263)).wrapping_add(seed);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 65535) as f32 / 65535.0
}
/// Periodic value noise: all lattice coordinates wrap, so opposite edges match.
fn noise(x: usize, y: usize, cells: usize, seed: u32) -> f32 {
    let fx = x as f32 * cells as f32 / WORK as f32;
    let fy = y as f32 * cells as f32 / WORK as f32;
    let ix = fx.floor() as usize; let iy = fy.floor() as usize;
    let tx = fx.fract(); let ty = fy.fract();
    let tx = tx * tx * (3.0 - 2.0 * tx); let ty = ty * ty * (3.0 - 2.0 * ty);
    let h = |dx, dy| hash((ix + dx) % cells, (iy + dy) % cells, seed);
    let a = h(0, 0) * (1.0 - tx) + h(1, 0) * tx;
    let b = h(0, 1) * (1.0 - tx) + h(1, 1) * tx;
    a * (1.0 - ty) + b * ty
}
fn tile_pattern(recipe: &str, name: &str, x: usize, y: usize, seed: u32) -> (f32, f32, f32) {
    let n = noise(x, y, 8, seed) * 0.55 + noise(x, y, 32, seed ^ 41) * 0.3 + noise(x, y, 128, seed ^ 97) * 0.15;
    match recipe {
        "brick" => {
            let row = y / 64; let xx = (x + (row % 2) * 64) % 128;
            let edge = (y % 64).min(63 - y % 64).min(xx).min(127 - xx);
            if edge < 3 { (0.53 + n * 0.12, -0.14, 1.0) }
            else { let bevel = if edge < 7 { 0.1 } else { 0.0 }; (0.77 + n * 0.3 + hash(xx / 128, row, seed) * 0.12, 0.035 + bevel, 1.0) }
        }
        "plank" => {
            let row = y / 128; let seam = y % 128 < 3 || (x + row * 113) % 512 < 3;
            let grain = noise(x, y * 12, 16, seed);
            if seam { (0.42, -0.14, 1.0) } else { (0.76 + n * 0.14 + grain * 0.2, (grain - 0.5) * 0.035, 1.0) }
        }
        "moss" => (0.62 + n * 0.58, (n - 0.5) * 0.14, 1.0),
        "glass" => {
            let edge = x.min(511 - x).min(y).min(511 - y);
            let glint = ((x + y) % 256) < 3 && x > 30 && x < 460 && y > 30 && y < 460;
            if edge < 4 { (0.8, 0.04, 0.8) } else if glint { (1.1, 0.012, 0.28) } else { (1.0, 0.0, 0.045) }
        }
        "metal" => {
            let brush = noise(x * 8, y * 8, 64, seed);
            let edge = x.min(511 - x).min(y).min(511 - y);
            if edge < 4 { (0.65, -0.05, 1.0) } else { (0.87 + brush * 0.13 + n * 0.05, (brush - 0.5) * 0.008, 1.0) }
        }
        "gem" => {
            let xx = x % 128; let yy = y % 128;
            let edge = xx.min(127 - xx).min(yy).min(127 - yy);
            let facet = ((x / 128 + y / 128) % 2) as f32 * 0.12;
            if edge < 5 { (0.65, -0.09, 1.0) } else if edge < 15 { (1.14, 0.09, 1.0) } else { (0.78 + facet + n * 0.13, facet * 0.15, 1.0) }
        }
        "leaves" => {
            let cx = (x % 64) as f32 - 32.0; let cy = (y % 64) as f32 - 32.0;
            let leaf = cx * cx / 600.0 + cy * cy / 260.0 < 0.75 + n * 0.55;
            (0.62 + n * 0.55, (n - 0.5) * 0.09, if leaf { 1.0 } else { 0.0 })
        }
        _ if name == "cobblestone" => {
            let row = y / 128; let xx = (x + (row % 2) * 64) % 128; let yy = y % 128;
            let edge = xx.min(127 - xx).min(yy).min(127 - yy);
            let joint = edge < 3 + (n * 6.0) as usize;
            if joint { (0.48 + n * 0.16, -0.14, 1.0) } else { (0.7 + n * 0.4, 0.06 + (n - 0.5) * 0.07, 1.0) }
        }
        _ if name == "deepslate" => {
            let layer = noise(x * 2, y * 16, 32, seed);
            (0.65 + n * 0.5, (layer - 0.5) * 0.07, 1.0)
        }
        _ => (0.72 + n * 0.43, (n - 0.5) * 0.085, 1.0)
    }
}
fn resize(values: &[f32], size: usize) -> Result<Vec<f32>, String> {
    if size == WORK { return Ok(values.to_vec()); }
    resize_wrap(values, WORK, WORK, size, size).map_err(str::to_string)
}
type PackedMaps = (Vec<u8>, Vec<u8>, Vec<u8>);
fn build(t: &Texture, m: &Material, img: &Images, options: &Options) -> Result<PackedMaps, String> {
    let size = options.resolution; let n = WORK * WORK;
    let seed = options.seed_for(&t.name);
    let mut channels: Vec<Vec<f32>> = (0..7).map(|_| vec![0.0; n]).collect();
    for y in 0..WORK {
        for x in 0..WORK {
            let i = y * WORK + x; let source = sample(&img.albedo, x, y);
            let (shade, relief, opacity) = tile_pattern(&t.recipe, &t.name, x, y, seed);
            let alpha = source[3] * opacity; channels[3][i] = alpha;
            for c in 0..3 {
                channels[c][i] = srgb_to_linear(source[c]) * srgb_to_linear(t.tint[c] as f32 / 255.0) * shade * alpha;
            }
            channels[4][i] = (0.5 + 0.12 * (source[0] + source[1] + source[2] - 1.5) + relief).clamp(0.0, 1.0);
            channels[5][i] = img.rough.as_ref().map(|r| sample(r, x, y)[0]).unwrap_or(0.8) * t.roughness_scale;
            channels[6][i] = (1.0 - relief.abs() * 1.2).clamp(0.0, 1.0);
        }
    }
    let normals = normals_from_height(&channels[4], WORK, WORK, t.normal_strength).map_err(str::to_string)?;
    let mut normal_channels: Vec<Vec<f32>> = (0..3).map(|_| vec![0.0; n]).collect();
    for y in 0..WORK {
        for x in 0..WORK {
            let i = y * WORK + x; let mut normal = normals[i];
            if let Some(map) = &img.normal {
                let p = sample(map, x, y); let nx = p[0] * 2.0 - 1.0;
                let mut ny = p[1] * 2.0 - 1.0;
                if m.normal_y == "opengl" { ny = -ny; }
                let scale = t.normal_strength / 2.5;
                normal[0] += nx * scale; normal[1] += ny * scale;
            }
            let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt().max(1e-12);
            for c in 0..3 { normal_channels[c][i] = normal[c] / len; }
        }
    }
    let channels: Vec<Vec<f32>> = channels.par_iter().map(|c| resize(c, size)).collect::<Result<_, _>>()?;
    let ns: Vec<Vec<f32>> = normal_channels.par_iter().map(|c| resize(c, size)).collect::<Result<_, _>>()?;
    let mut albedo = Vec::with_capacity(size * size * 4);
    let mut normal = Vec::with_capacity(size * size * 4);
    let mut spec = Vec::with_capacity(size * size * 4);
    for i in 0..size * size {
        let alpha = channels[3][i].clamp(0.0, 1.0);
        for c in channels.iter().take(3) { albedo.push(if alpha > 1e-6 { linear_to_srgb(c[i] / alpha) } else { 0 }); }
        albedo.push(unit(alpha));
        normal.extend_from_slice(&normal_pixel([ns[0][i], ns[1][i], ns[2][i]], channels[6][i], channels[4][i]));
        spec.extend_from_slice(&specular_pixel(channels[5][i], F0(t.f0), t.porosity, t.emissive));
    }
    Ok((albedo, normal, spec))
}
/// Builds into an exclusive staging directory; existing output is never overwritten.
pub fn generate(input: &Path, output: &Path) -> Result<(), String> { generate_with_options(input, output, &Options::default()) }
pub fn generate_with_options(input: &Path, output: &Path, options: &Options) -> Result<(), String> {
    options.validate()?;
    let path = if input.is_dir() { input.join("material_manifest.json") } else { input.to_path_buf() };
    let root = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let data = manifest::read(&path)?; manifest::validate(&data, root)?;
    let names: std::collections::HashSet<_> = data.textures.iter().map(|t| &t.name).collect();
    for t in &data.textures {
        if options.include_normal && names.contains(&format!("{}_n", t.name)) { return Err(format!("{}: normal output name collides with another texture", t.name)); }
        if options.include_specular && names.contains(&format!("{}_s", t.name)) { return Err(format!("{}: specular output name collides with another texture", t.name)); }
    }
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    for name in ["pack", "dist", "contact_sheet.png", "build_report.json"] {
        // Also reject broken symlinks, which Path::exists() does not detect.
        if output.join(name).symlink_metadata().is_ok() { return Err(format!("refusing to overwrite {}", output.join(name).display())); }
    }
    let stage = output.join(format!(".hyperrealism-stage-{}", std::process::id()));
    fs::create_dir(&stage).map_err(|e| format!("{}: {e}", stage.display()))?;
    let result = build_stage(&data, root, &stage, options);
    if let Err(err) = result { let _ = fs::remove_dir_all(&stage); return Err(err); }
    let outputs = ["pack", "dist", "contact_sheet.png", "build_report.json"];
    let mut moved = Vec::new();
    for name in outputs {
        if let Err(e) = fs::rename(stage.join(name), output.join(name)) {
            for previous in moved.iter().rev() { let _ = fs::rename(output.join(previous), stage.join(previous)); }
            let _ = fs::remove_dir_all(&stage);
            return Err(format!("could not install {name}: {e}"));
        }
        moved.push(name);
    }
    fs::remove_dir(&stage).map_err(|e| e.to_string())?;
    println!("Generated {} textures in {}", data.textures.len(), output.display());
    Ok(())
}
fn build_stage(data: &manifest::Manifest, root: &Path, stage: &Path, options: &Options) -> Result<(), String> {
    let start = Instant::now(); let size = options.resolution;
    let pack = stage.join("pack"); let block = pack.join("assets/minecraft/textures/block");
    let optifine = pack.join("assets/minecraft/optifine");
    fs::create_dir_all(&block).map_err(|e| e.to_string())?;
    fs::create_dir_all(&optifine).map_err(|e| e.to_string())?;
    let description = serde_json::to_string(&options.name).map_err(|e| e.to_string())?;
    fs::write(pack.join("pack.mcmeta"), format!("{{\n  \"pack\": {{\n    \"description\": {description},\n    \"min_format\": [97, 1],\n    \"max_format\": 97\n  }}\n}}\n")).map_err(|e| e.to_string())?;
    fs::write(optifine.join("texture.properties"), "format=lab-pbr/1.3\n").map_err(|e| e.to_string())?;
    let materials = manifest::by_id(data);
    let cols = 4; let rows = data.textures.len().div_ceil(cols);
    let mut sheet = vec![30; cols * size * rows * size * 4];
    for i in 0..rows * size { for x in 0..cols * size { sheet[(i * cols * size + x) * 4 + 3] = 255; } }
    let mut outputs = Vec::new();
    for (index, t) in data.textures.iter().enumerate() {
        let m = materials[t.material.as_str()];
        let images = load(root, m).map_err(|e| format!("{}: {e}", t.name))?;
        let (a, n, s) = build(t, m, &images, options).map_err(|e| format!("{}: {e}", t.name))?;
        let mut maps = vec![("", &a)];
        if options.include_normal { maps.push(("_n", &n)); }
        if options.include_specular { maps.push(("_s", &s)); }
        for (suffix, bytes) in maps {
            let filename = format!("{}{suffix}.png", t.name);
            png_io::write(&block.join(&filename), size, size, bytes)?;
            outputs.push(serde_json::json!({"path": format!("assets/minecraft/textures/block/{filename}"), "width": size, "height": size}));
        }
        let ox = index % cols * size; let oy = index / cols * size;
        for y in 0..size {
            let dst = ((oy + y) * cols * size + ox) * 4; let src = y * size * 4;
            sheet[dst..dst + size * 4].copy_from_slice(&a[src..src + size * 4]);
        }
        println!("PROGRESS {}/{} {}", index + 1, data.textures.len(), t.name);
    }
    png_io::write(&stage.join("contact_sheet.png"), cols * size, rows * size, &sheet)?;
    let dist: PathBuf = stage.join("dist"); fs::create_dir_all(&dist).map_err(|e| e.to_string())?;
    let archive = format!("HyperRealism{}-v1.0-MC26.3.zip", size);
    zip::write(&pack, &dist.join(&archive))?;
    let mut input_hashes = Vec::new();
    for m in &data.materials {
        for file in std::iter::once(&m.albedo).chain(m.normal.iter()).chain(m.roughness.iter()) {
            let path = manifest::safe_file(root, file)?;
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(&bytes);
            input_hashes.push(serde_json::json!({"material": m.id, "file": file, "crc32": format!("{:08x}", hasher.finalize())}));
        }
    }
    let report = serde_json::json!({
        "schema_version": 1, "pack_name": options.name, "backend": "cpu", "seed": options.seed,
        "resolution": size, "texture_count": data.textures.len(), "files": outputs,
        "archive": format!("dist/{archive}"), "normal_maps": options.include_normal,
        "specular_maps": options.include_specular, "input_checksums": input_hashes,
        "elapsed_ms": start.elapsed().as_millis(), "minecraft_validation": "not performed; target metadata 97.1",
        "source_provenance": "User-supplied maps; bundled test fixtures and catalog are synthetic, not Poly Haven assets."
    });
    fs::write(stage.join("build_report.json"), serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(())
}
