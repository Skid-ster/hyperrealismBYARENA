use crate::{
    labpbr::{normal_pixel, normals_from_height, specular_pixel, F0},
    manifest::{self, Material, Texture},
    png_io::{self, Rgba},
    resample::resize_wrap,
    zip,
};
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};
const WORK: usize = 512;
const SIZE: usize = 128;
const ARCHIVE: &str = "HyperRealism128-v1.0-MC26.3.zip";

fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn linear_to_srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let x = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (x * 255.0).round() as u8
}
fn unit(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

struct Images {
    albedo: Rgba,
    rough: Option<Rgba>,
    normal: Option<Rgba>,
}
fn load(root: &Path, m: &Material) -> Result<Images, String> {
    let load_one = |s: &str| png_io::read(&manifest::safe_file(root, s)?);
    Ok(Images {
        albedo: load_one(&m.albedo)?,
        rough: m.roughness.as_deref().map(load_one).transpose()?,
        normal: m.normal.as_deref().map(load_one).transpose()?,
    })
}
fn sample(img: &Rgba, x: usize, y: usize) -> [f32; 4] {
    let ix = x * img.width / WORK;
    let iy = y * img.height / WORK;
    let i = (iy * img.width + ix) * 4;
    [
        img.pixels[i] as f32 / 255.0,
        img.pixels[i + 1] as f32 / 255.0,
        img.pixels[i + 2] as f32 / 255.0,
        img.pixels[i + 3] as f32 / 255.0,
    ]
}
fn tile_pattern(recipe: &str, x: usize, y: usize) -> (f32, f32, f32) {
    let wave = (((x as f32 * std::f32::consts::TAU / 128.0).sin()
        + (y as f32 * std::f32::consts::TAU / 128.0).cos())
        * 0.5
        + 0.5)
        .clamp(0.0, 1.0);
    match recipe {
        "brick" => {
            let row = y / 128;
            let xx = (x + (row % 2) * 64) % 128;
            let mortar = y % 128 < 7 || xx < 7;
            if mortar {
                (0.5, -0.15, 1.0)
            } else {
                (1.0, 0.05, 1.0)
            }
        }
        "plank" => {
            let seam = y % 128 < 5;
            if seam {
                (0.45, -0.12, 1.0)
            } else {
                (0.91 + 0.09 * wave, 0.04, 1.0)
            }
        }
        "moss" => (0.8 + 0.2 * wave, 0.05 * wave, 1.0),
        "glass" => {
            let edge = x % 128 < 5 || y % 128 < 5;
            if edge {
                (0.8, 0.03, 0.75)
            } else {
                (1.0, 0.0, 0.25)
            }
        }
        "metal" => (0.92 + 0.08 * wave, 0.0, 1.0),
        "gem" => (0.8 + 0.2 * wave, 0.08 * wave, 1.0),
        "leaves" => {
            let a = if ((x / 16 + y / 16) % 7 == 0) && wave < 0.6 {
                0.0
            } else {
                1.0
            };
            (0.85 + 0.15 * wave, 0.04 * wave, a)
        }
        _ => (0.9 + 0.1 * wave, 0.0, 1.0),
    }
}
fn resize(values: &[f32]) -> Result<Vec<f32>, String> {
    resize_wrap(values, WORK, WORK, SIZE, SIZE).map_err(str::to_string)
}
type PackedMaps = (Vec<u8>, Vec<u8>, Vec<u8>);

fn build(t: &Texture, m: &Material, img: &Images) -> Result<PackedMaps, String> {
    let n = WORK * WORK;
    let mut channels: Vec<Vec<f32>> = (0..7).map(|_| vec![0.0; n]).collect();
    for y in 0..WORK {
        for x in 0..WORK {
            let i = y * WORK + x;
            let source = sample(&img.albedo, x, y);
            let (shade, relief, opacity) = tile_pattern(&t.recipe, x, y);
            let alpha = source[3] * opacity;
            channels[3][i] = alpha;
            for c in 0..3 {
                channels[c][i] = srgb_to_linear(source[c])
                    * srgb_to_linear(t.tint[c] as f32 / 255.0)
                    * shade
                    * alpha;
            }
            channels[4][i] =
                (0.5 + 0.22 * (source[0] + source[1] + source[2] - 1.5) + relief).clamp(0.0, 1.0);
            channels[5][i] = img
                .rough
                .as_ref()
                .map(|r| sample(r, x, y)[0])
                .unwrap_or(0.8);
            channels[6][i] = (1.0 - relief.abs() * 1.2).clamp(0.0, 1.0);
        }
    }
    let normals = normals_from_height(&channels[4], WORK, WORK, 2.5).map_err(str::to_string)?;
    let mut normal_channels: Vec<Vec<f32>> = (0..3).map(|_| vec![0.0; n]).collect();
    for y in 0..WORK {
        for x in 0..WORK {
            let i = y * WORK + x;
            let mut normal = normals[i];
            if let Some(map) = &img.normal {
                let p = sample(map, x, y);
                let nx = p[0] * 2.0 - 1.0;
                let mut ny = p[1] * 2.0 - 1.0;
                if m.normal_y == "opengl" {
                    ny = -ny;
                }
                normal[0] += nx;
                normal[1] += ny;
            }
            let len =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            for c in 0..3 {
                normal_channels[c][i] = normal[c] / len;
            }
        }
    }
    let channels: Vec<Vec<f32>> = channels
        .par_iter()
        .map(|c| resize(c))
        .collect::<Result<_, _>>()?;
    let ns: Vec<Vec<f32>> = normal_channels
        .par_iter()
        .map(|c| resize(c))
        .collect::<Result<_, _>>()?;
    let mut albedo = Vec::with_capacity(SIZE * SIZE * 4);
    let mut normal = Vec::with_capacity(SIZE * SIZE * 4);
    let mut spec = Vec::with_capacity(SIZE * SIZE * 4);
    for i in 0..SIZE * SIZE {
        let alpha = channels[3][i].clamp(0.0, 1.0);
        for c in channels.iter().take(3) {
            albedo.push(if alpha > 1e-6 {
                linear_to_srgb(c[i] / alpha)
            } else {
                0
            });
        }
        albedo.push(unit(alpha));
        normal.extend_from_slice(&normal_pixel(
            [ns[0][i], ns[1][i], ns[2][i]],
            channels[6][i],
            channels[4][i],
        ));
        spec.extend_from_slice(&specular_pixel(
            channels[5][i],
            F0(t.f0),
            t.porosity,
            t.emissive,
        ));
    }
    Ok((albedo, normal, spec))
}

/// Builds into an exclusive staging directory; existing pack/dist output is never overwritten.
pub fn generate(input: &Path, output: &Path) -> Result<(), String> {
    let path = if input.is_dir() {
        input.join("material_manifest.json")
    } else {
        input.to_path_buf()
    };
    let root = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let data = manifest::read(&path)?;
    manifest::validate(&data, root)?;
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    for name in ["pack", "dist", "contact_sheet.png"] {
        if output.join(name).exists() {
            return Err(format!(
                "refusing to overwrite {}",
                output.join(name).display()
            ));
        }
    }
    let stage = output.join(format!(".hyperrealism-stage-{}", std::process::id()));
    fs::create_dir(&stage).map_err(|e| format!("{}: {e}", stage.display()))?;
    let result = build_stage(&data, root, &stage);
    if let Err(err) = result {
        let _ = fs::remove_dir_all(&stage);
        return Err(err);
    }
    // Move completed outputs only after all generation has succeeded.
    let mut moved = Vec::new();
    for name in ["pack", "dist", "contact_sheet.png"] {
        if let Err(e) = fs::rename(stage.join(name), output.join(name)) {
            for previous in moved.iter().rev() {
                let _ = fs::rename(output.join(previous), stage.join(previous));
            }
            let _ = fs::remove_dir_all(&stage);
            return Err(format!("could not install {name}: {e}"));
        }
        moved.push(name);
    }
    fs::remove_dir(&stage).map_err(|e| e.to_string())?;
    println!(
        "Generated {} textures in {}",
        data.textures.len(),
        output.display()
    );
    Ok(())
}
fn build_stage(data: &manifest::Manifest, root: &Path, stage: &Path) -> Result<(), String> {
    let pack = stage.join("pack");
    let block = pack.join("assets/minecraft/textures/block");
    let optifine = pack.join("assets/minecraft/optifine");
    fs::create_dir_all(&block).map_err(|e| e.to_string())?;
    fs::create_dir_all(&optifine).map_err(|e| e.to_string())?;
    fs::write(pack.join("pack.mcmeta"),"{\n  \"pack\": {\n    \"description\": \"HyperRealism128 CPU preview\",\n    \"min_format\": [97, 1],\n    \"max_format\": 97\n  }\n}\n").map_err(|e|e.to_string())?;
    fs::write(optifine.join("texture.properties"), "format=lab-pbr/1.3\n")
        .map_err(|e| e.to_string())?;
    let materials = manifest::by_id(data);
    let cols = 4;
    let rows = data.textures.len().div_ceil(cols);
    let mut sheet = vec![0; cols * SIZE * rows * SIZE * 4];
    for (index, t) in data.textures.iter().enumerate() {
        println!("{} ({})", t.name, t.recipe);
        let m = materials[t.material.as_str()];
        let images = load(root, m)?;
        let (a, n, s) = build(t, m, &images).map_err(|e| format!("{}: {e}", t.name))?;
        for (suffix, bytes) in [("", &a), ("_n", &n), ("_s", &s)] {
            png_io::write(
                &block.join(format!("{}{suffix}.png", t.name)),
                SIZE,
                SIZE,
                bytes,
            )?;
        }
        let ox = index % cols * SIZE;
        let oy = index / cols * SIZE;
        for y in 0..SIZE {
            let dst = ((oy + y) * cols * SIZE + ox) * 4;
            let src = y * SIZE * 4;
            sheet[dst..dst + SIZE * 4].copy_from_slice(&a[src..src + SIZE * 4]);
        }
    }
    png_io::write(
        &stage.join("contact_sheet.png"),
        cols * SIZE,
        rows * SIZE,
        &sheet,
    )?;
    let dist: PathBuf = stage.join("dist");
    fs::create_dir_all(&dist).map_err(|e| e.to_string())?;
    zip::write(&pack, &dist.join(ARCHIVE))?;
    Ok(())
}
