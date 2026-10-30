use crate::{
    labpbr::{normal_pixel, normals_from_height, specular_pixel, F0},
    manifest::{self, Texture},
    material::{self, Prepared},
    png_io, recipes,
    resample::resize_wrap,
    zip,
};
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};
const WORK: usize = material::WORK;
const SHEET_TILE: usize = 128;
const ARCHIVE: &str = "HyperRealism128-v1.0-MC26.3.zip";

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

fn resize(values: &[f32], size: usize) -> Result<Vec<f32>, String> {
    if size == WORK {
        return Ok(values.to_vec());
    }
    resize_wrap(values, WORK, WORK, size, size).map_err(str::to_string)
}
type PackedMaps = (Vec<u8>, Vec<u8>, Vec<u8>);

fn build(
    t: &Texture,
    img: &Prepared,
    root: &Path,
    size: usize,
    pbr_strength: f32,
) -> Result<PackedMaps, String> {
    let n = WORK * WORK;
    let mut channels: Vec<Vec<f32>> = (0..7).map(|_| vec![0.0; n]).collect();
    let mut relief_map = vec![0.0; n];
    let mut ore_map = if t.recipe == "ore" {
        Some(vec![0.0; n])
    } else {
        None
    };
    let ore_color = t
        .ore_color
        .map(|rgb| rgb.map(|v| material::srgb_to_linear(v as f32 / 255.0)));
    let tint = t.tint.map(|v| material::srgb_to_linear(v as f32 / 255.0));
    // The target matches the approximate average in linear light. It is not a
    // pixel-by-pixel vanilla palette copy; clipping and recipe shading affect it.
    let gain: [f32; 3] = std::array::from_fn(|c| {
        let target_gain = t
            .target_color
            .map(|rgb| {
                let target = material::srgb_to_linear(rgb[c] as f32 / 255.0);
                if img.mean_linear[c] > 1e-6 {
                    (target / img.mean_linear[c]).min(16.0)
                } else {
                    0.0
                }
            })
            .unwrap_or(1.0);
        tint[c] * target_gain
    });
    let seed = recipes::seed(&t.name);
    for y in 0..WORK {
        for x in 0..WORK {
            let i = y * WORK + x;
            let pattern = recipes::evaluate(&t.recipe, x, y, seed);
            let (shade, relief, opacity) = (pattern.shade, pattern.relief, pattern.opacity);
            let source_alpha = img.albedo[3][i];
            let alpha = source_alpha * opacity;
            channels[3][i] = alpha;
            // Normalized photographic luminance gives ore inclusions some of
            // the underlying scan's fine structure rather than a flat fill.
            let luminance = if source_alpha > 1e-6 {
                (0.2126 * img.albedo[0][i] + 0.7152 * img.albedo[1][i] + 0.0722 * img.albedo[2][i])
                    / source_alpha
            } else {
                0.5
            };
            let mean_luminance = 0.2126 * img.mean_linear[0]
                + 0.7152 * img.mean_linear[1]
                + 0.0722 * img.mean_linear[2];
            let mineral_detail = if mean_luminance > 1e-6 {
                (luminance / mean_luminance).clamp(0.55, 1.45)
            } else {
                1.0
            };
            for c in 0..3 {
                let base = img.albedo[c][i] * gain[c] * shade * opacity;
                // Moss is a separate, seed-stable coherent layer, not a
                // solid green tint over the entire material.
                let moss = [0.045, 0.12, 0.025][c] * alpha;
                let surface = base * (1.0 - pattern.moss) + moss * pattern.moss;
                channels[c][i] = if let Some(color) = ore_color {
                    let grain = pattern.shade * mineral_detail * color[c] * alpha;
                    surface * (1.0 - pattern.ore) + grain * pattern.ore
                } else {
                    surface
                };
            }
            // With no height map, a mild luminance heuristic preserves the old
            // prototype's behavior. A normal map already supplies fine detail,
            // so its generated normal receives only recipe-level relief.
            let base_height = img
                .height
                .as_ref()
                .map(|v| v[i])
                .unwrap_or(0.5 + 0.22 * (luminance - 0.5));
            channels[4][i] = (base_height + relief).clamp(0.0, 1.0);
            channels[5][i] = img.roughness.as_ref().map(|v| v[i]).unwrap_or(0.8);
            channels[6][i] = (1.0 - relief.abs() * 1.2).clamp(0.0, 1.0)
                * img.ao.as_ref().map(|v| v[i]).unwrap_or(1.0);
            relief_map[i] = relief;
            if let Some(map) = &mut ore_map {
                map[i] = pattern.ore;
            }
        }
    }
    let height_for_normals = if img.normal.is_some() {
        &relief_map
    } else {
        &channels[4]
    };
    let normals =
        normals_from_height(height_for_normals, WORK, WORK, 2.5).map_err(str::to_string)?;
    let mut normal_channels: Vec<Vec<f32>> = (0..3).map(|_| vec![0.0; n]).collect();
    for i in 0..n {
        let detail = normals[i];
        let normal = if let Some(map) = &img.normal {
            let base = [map[0][i], map[1][i], map[2][i]];
            combine_normals(base, detail)
        } else {
            detail
        };
        for c in 0..3 {
            normal_channels[c][i] = normal[c];
        }
    }
    let channels: Vec<Vec<f32>> = channels
        .par_iter()
        .map(|c| resize(c, size))
        .collect::<Result<_, _>>()?;
    let ns: Vec<Vec<f32>> = normal_channels
        .par_iter()
        .map(|c| resize(c, size))
        .collect::<Result<_, _>>()?;
    // Filter continuous masks first, then threshold material IDs. Never
    // interpolate IDs or the 255 no-emission sentinel.
    let id_mask = t
        .f0_mask
        .as_deref()
        .map(|p| material::scalar(root, p).and_then(|v| resize(&v, size)))
        .transpose()?;
    let ore_mask = ore_map.as_ref().map(|v| resize(v, size)).transpose()?;
    let emission = t
        .emissive_map
        .as_deref()
        .map(|p| material::scalar(root, p).and_then(|v| resize(&v, size)))
        .transpose()?;
    let mut albedo = Vec::with_capacity(size * size * 4);
    let mut normal = Vec::with_capacity(size * size * 4);
    let mut spec = Vec::with_capacity(size * size * 4);
    for i in 0..size * size {
        let alpha = channels[3][i].clamp(0.0, 1.0);
        for c in channels.iter().take(3) {
            albedo.push(if alpha > 1e-6 {
                linear_to_srgb(c[i] / alpha)
            } else {
                0
            });
        }
        albedo.push(unit(alpha));
        // Scale tangent slopes (rather than encoded XY bytes) and compress the
        // displacement around its midplane. AO fades toward unoccluded white.
        // Keep normals unit length via normal_pixel and never emit zero height.
        normal.extend_from_slice(&normal_pixel(
            [ns[0][i] * pbr_strength, ns[1][i] * pbr_strength, ns[2][i]],
            1.0 - (1.0 - channels[6][i].clamp(0.0, 1.0)) * pbr_strength,
            (0.5 + (channels[4][i] - 0.5) * pbr_strength).max(1.0 / 255.0),
        ));
        let id = if ore_mask.as_ref().map(|m| m[i] >= 0.5).unwrap_or(false) {
            t.ore_f0.expect("validated ore F0")
        } else if id_mask.as_ref().map(|m| m[i] >= 0.5).unwrap_or(false) {
            t.masked_f0.expect("validated masked F0")
        } else {
            t.f0
        };
        let blue = if id >= 230 {
            0
        } else {
            t.subsurface.unwrap_or(t.porosity)
        };
        let emissive = emission
            .as_ref()
            .map(|v| (v[i].clamp(0.0, 1.0) * 254.0).round() as u8)
            .unwrap_or(t.emissive);
        // LabPBR red is perceptual smoothness (1 - sqrt(roughness)). Scale
        // that value, not the metal-ID, F0, subsurface, or emission channels.
        let smoothness = (1.0 - channels[5][i].clamp(0.0, 1.0).sqrt()) * pbr_strength;
        let roughness = (1.0 - smoothness).powi(2);
        spec.extend_from_slice(&specular_pixel(roughness, F0(id), blue, emissive));
    }
    Ok((albedo, normal, spec))
}

/// Reoriented normal mapping: a flat detail normal leaves the source unchanged.
fn combine_normals(base: [f32; 3], detail: [f32; 3]) -> [f32; 3] {
    fn normalize(v: [f32; 3]) -> [f32; 3] {
        let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if length > 1e-8 && length.is_finite() {
            [v[0] / length, v[1] / length, v[2] / length]
        } else {
            [0.0, 0.0, 1.0]
        }
    }
    let b = normalize(base);
    let t = [b[0], b[1], b[2] + 1.0];
    if t[2] < 1e-6 {
        return b;
    }
    let u = [-detail[0], -detail[1], detail[2]];
    let scale = (t[0] * u[0] + t[1] * u[1] + t[2] * u[2]) / t[2];
    normalize([
        t[0] * scale - u[0],
        t[1] * scale - u[1],
        t[2] * scale - u[2],
    ])
}

/// Builds into an exclusive staging directory; existing pack/dist output is never overwritten.
pub fn generate(
    input: &Path,
    output: &Path,
    minecraft_jar: Option<&Path>,
    size: usize,
) -> Result<(), String> {
    if size != 128 && size != 512 {
        return Err("resolution must be 128 or 512".into());
    }
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
    let hashes = manifest::input_hashes(&data, root)?;
    let manifest_hash = manifest::sha256_file(&path)?;
    let metadata = if let Some(jar) = minecraft_jar {
        manifest::validate_against_jar(&data, jar)?
    } else {
        Default::default()
    };
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    for name in ["pack", "dist", "contact_sheet.png", "build_report.json"] {
        if output.join(name).exists() {
            return Err(format!(
                "refusing to overwrite {}",
                output.join(name).display()
            ));
        }
    }
    let stage = output.join(format!(".hyperrealism-stage-{}", std::process::id()));
    fs::create_dir(&stage).map_err(|e| format!("{}: {e}", stage.display()))?;
    let info = BuildInfo {
        hashes: &hashes,
        manifest_hash: &manifest_hash,
        client_paths_checked: minecraft_jar.is_some(),
        metadata: &metadata,
        size,
    };
    let result = build_stage(&data, root, &stage, &info);
    if let Err(err) = result {
        let _ = fs::remove_dir_all(&stage);
        return Err(err);
    }
    // Move completed outputs only after all generation has succeeded.
    let mut moved = Vec::new();
    for name in ["pack", "dist", "contact_sheet.png", "build_report.json"] {
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
struct BuildInfo<'a> {
    hashes: &'a std::collections::BTreeMap<String, String>,
    manifest_hash: &'a str,
    client_paths_checked: bool,
    metadata: &'a std::collections::BTreeMap<String, Vec<u8>>,
    size: usize,
}
fn build_stage(
    data: &manifest::Manifest,
    root: &Path,
    stage: &Path,
    info: &BuildInfo<'_>,
) -> Result<(), String> {
    let BuildInfo {
        hashes,
        manifest_hash,
        client_paths_checked,
        metadata,
        size,
    } = *info;
    let pack = stage.join("pack");
    let block = pack.join("assets/minecraft/textures/block");
    let optifine = pack.join("assets/minecraft/optifine");
    fs::create_dir_all(&block).map_err(|e| e.to_string())?;
    fs::create_dir_all(&optifine).map_err(|e| e.to_string())?;
    fs::write(pack.join("pack.mcmeta"),"{\n  \"pack\": {\n    \"description\": \"HyperRealism LabPBR CPU preview\",\n    \"min_format\": [97, 1],\n    \"max_format\": 97\n  }\n}\n").map_err(|e|e.to_string())?;
    fs::write(optifine.join("texture.properties"), "format=lab-pbr/1.3\n")
        .map_err(|e| e.to_string())?;
    let materials = manifest::by_id(data);
    // Carry attribution and honest coverage information into the distributable
    // ZIP. These fields are supplied by the manifest and are not independently
    // license-verified by the generator.
    let used: std::collections::BTreeSet<_> =
        data.textures.iter().map(|t| t.material.as_str()).collect();
    let mut credits = String::from("HyperRealism128 material credits\n\nGenerated from the following manifest-supplied materials.\nSource and license information is supplied by the manifest, not independently verified.\n\n");
    for id in used {
        let m = materials[id];
        credits.push_str(&format!(
            "Material: {}\nSource: {}\nLicense: {}\n\n",
            id,
            m.source.as_deref().unwrap_or("not specified"),
            m.license.as_deref().unwrap_or("not specified")
        ));
    }
    fs::write(pack.join("MATERIAL_CREDITS.txt"), credits).map_err(|e| e.to_string())?;
    fs::write(pack.join("README.txt"), format!("HyperRealism128 experimental preview for Minecraft Java 26.3\n\nThis preview replaces {} block textures at {}x{}, not the full game. Other textures remain vanilla or come from other enabled packs.\nDesign target: Windows 11, NVIDIA RTX 5060 Ti 8GB, Intel i5-13400 and 16GB system RAM. No performance guarantee or target-hardware benchmark is implied.\nTo install, place this ZIP in the Minecraft resourcepacks folder and enable it in Resource Packs.\nA compatible shader is needed for LabPBR maps; support for optional channels varies.\nConnected glass requires a compatible CTM mod; Iris alone does not add CTM.\nThis is an unfinished preview, not a production v1.0 release.\nSee MATERIAL_CREDITS.txt for material provenance.\n", data.textures.len(), size, size)).map_err(|e| e.to_string())?;
    let cols = 4;
    let rows = data.textures.len().div_ceil(cols);
    // Keep review thumbnails small even for the 512px pack.
    let mut sheet = vec![0; cols * SHEET_TILE * rows * SHEET_TILE * 4];
    // Keep only the most recently used prepared material; this bounds memory.
    let mut cached: Option<(&str, Prepared)> = None;
    for (index, t) in data.textures.iter().enumerate() {
        println!("{} ({})", t.name, t.recipe);
        let m = materials[t.material.as_str()];
        if cached.as_ref().map(|(id, _)| *id) != Some(m.id.as_str()) {
            drop(cached.take());
            cached = Some((m.id.as_str(), material::load(root, m)?));
        }
        let images = &cached.as_ref().expect("material just loaded").1;
        let (a, n, s) = build(
            t,
            images,
            root,
            size,
            t.pbr_strength.unwrap_or(data.pbr_strength),
        )
        .map_err(|e| format!("{}: {e}", t.name))?;
        if t.connected_glass {
            crate::connected::write(&pack, t, size, &a, &n, &s)?;
        }
        for (suffix, bytes) in [("", &a), ("_n", &n), ("_s", &s)] {
            png_io::write(
                &block.join(format!("{}{suffix}.png", t.name)),
                size,
                size,
                bytes,
            )?;
        }
        if let Some(meta) = metadata.get(&t.name) {
            // Preserve validated metadata from the user's actual client JAR.
            fs::write(block.join(format!("{}.png.mcmeta", t.name)), meta)
                .map_err(|e| e.to_string())?;
        } else if let Some(strategy) = &t.mipmap_strategy {
            // The audited 26.3 textures use static mipmap strategies;
            // unlike animation metadata this does not change frame layout.
            let meta = serde_json::json!({"texture": {"mipmap_strategy": strategy}});
            fs::write(
                block.join(format!("{}.png.mcmeta", t.name)),
                serde_json::to_vec(&meta).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }
        let ox = index % cols * SHEET_TILE;
        let oy = index / cols * SHEET_TILE;
        let factor = size / SHEET_TILE;
        for y in 0..SHEET_TILE {
            for x in 0..SHEET_TILE {
                let dst = ((oy + y) * cols * SHEET_TILE + ox + x) * 4;
                for c in 0..4 {
                    let mut sum = 0u32;
                    for sy in 0..factor {
                        for sx in 0..factor {
                            sum += a[((y * factor + sy) * size + x * factor + sx) * 4 + c] as u32;
                        }
                    }
                    sheet[dst + c] = (sum / (factor * factor) as u32) as u8;
                }
            }
        }
    }
    let verified = crate::verify::pack(&pack)?;
    if verified != data.textures.len() {
        return Err(format!(
            "pack verification: expected {} textures, got {verified}",
            data.textures.len()
        ));
    }
    let report = serde_json::json!({
        "generator_version": env!("CARGO_PKG_VERSION"),
        "backend": "cpu",
        "pbr_strength": data.pbr_strength,
        "resolution": size,
        "working_resolution": WORK,
        "textures": data.textures.iter().map(|t| serde_json::json!({"name": t.name, "material": t.material, "recipe": t.recipe, "tint": t.tint, "target_color": t.target_color, "mipmap_strategy": t.mipmap_strategy, "pbr_strength": t.pbr_strength, "connected_glass": t.connected_glass, "ore_color": t.ore_color, "ore_f0": t.ore_f0, "f0": t.f0, "porosity": t.porosity, "subsurface": t.subsurface, "f0_mask": t.f0_mask, "masked_f0": t.masked_f0, "emissive": t.emissive, "emissive_map": t.emissive_map})).collect::<Vec<_>>(),
        "materials": data.materials.iter().map(|m| serde_json::json!({"id": m.id, "source": m.source, "license": m.license, "albedo": m.albedo, "normal": m.normal, "normal_y": m.normal_y, "rotate_cw": m.rotate_cw, "roughness": m.roughness, "arm": m.arm, "height": m.height, "ao": m.ao})).collect::<Vec<_>>(),
        "manifest_sha256": manifest_hash,
        "client_paths_checked": client_paths_checked,
        "input_sha256": hashes,
    });
    let mut report_data = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    report_data.push(b'\n');
    fs::write(stage.join("build_report.json"), report_data).map_err(|e| e.to_string())?;
    png_io::write(
        &stage.join("contact_sheet.png"),
        cols * SHEET_TILE,
        rows * SHEET_TILE,
        &sheet,
    )?;
    let dist: PathBuf = stage.join("dist");
    fs::create_dir_all(&dist).map_err(|e| e.to_string())?;
    zip::write(&pack, &dist.join(ARCHIVE))?;
    Ok(())
}
