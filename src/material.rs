//! Decode and prepare one material at the common working resolution.
//! Continuous maps are periodically filtered; IDs and flags are never resized.
use crate::{
    manifest::{self, Material},
    png_io::{self, Rgba},
    resample::resize_wrap,
};
use std::path::Path;

pub const WORK: usize = 512;

pub struct Prepared {
    /// Linear-light, premultiplied RGB and linear opacity.
    pub albedo: [Vec<f32>; 4],
    /// Mean linear albedo weighted by alpha; used for optional palette adaptation.
    pub mean_linear: [f32; 3],
    pub roughness: Option<Vec<f32>>,
    pub normal: Option<[Vec<f32>; 3]>,
    pub height: Option<Vec<f32>>,
    pub ao: Option<Vec<f32>>,
}

pub fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn plane<F: Fn(&[u8]) -> f32>(image: &Rgba, f: F) -> Result<Vec<f32>, String> {
    let source: Vec<f32> = image.pixels.chunks_exact(4).map(f).collect();
    if image.width == WORK && image.height == WORK {
        return Ok(source);
    }
    resize_wrap(&source, image.width, image.height, WORK, WORK).map_err(str::to_string)
}

fn scalar(root: &Path, name: &str) -> Result<Vec<f32>, String> {
    let path = manifest::safe_file(root, name)?;
    let (width, height, source) = png_io::read_linear_scalar(&path)?;
    let mut result = if width == WORK && height == WORK {
        source
    } else {
        resize_wrap(&source, width, height, WORK, WORK).map_err(str::to_string)?
    };
    for v in &mut result {
        *v = v.clamp(0.0, 1.0);
    }
    Ok(result)
}

pub fn load(root: &Path, material: &Material) -> Result<Prepared, String> {
    // Drop each decoded source before loading the next, bounding input memory.
    let image = png_io::read_material(&manifest::safe_file(root, &material.albedo)?)?;
    let table: Vec<f32> = (0..=255)
        .map(|v| srgb_to_linear(v as f32 / 255.0))
        .collect();
    let mut rgb = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for channel in 0..3 {
        rgb[channel] = plane(&image, |p| {
            table[p[channel] as usize] * (p[3] as f32 / 255.0)
        })?;
    }
    rgb[3] = plane(&image, |p| p[3] as f32 / 255.0)?;
    let (colors, alpha) = rgb.split_at_mut(3);
    for value in &mut alpha[0] {
        *value = value.clamp(0.0, 1.0);
    }
    for color in colors {
        for (value, &alpha) in color.iter_mut().zip(&alpha[0]) {
            *value = value.clamp(0.0, alpha);
        }
    }
    let alpha_sum = rgb[3].iter().map(|&v| v as f64).sum::<f64>();
    let mean_linear = std::array::from_fn(|c| {
        if alpha_sum > 1e-8 {
            (rgb[c].iter().map(|&v| v as f64).sum::<f64>() / alpha_sum) as f32
        } else {
            0.0
        }
    });
    drop(image);
    let normal = if let Some(path) = &material.normal {
        let image = png_io::read_material(&manifest::safe_file(root, path)?)?;
        let mut v = [Vec::new(), Vec::new(), Vec::new()];
        for channel in 0..3 {
            v[channel] = plane(&image, |p| {
                let x = p[channel] as f32 / 127.5 - 1.0;
                if channel == 1 && material.normal_y == "opengl" {
                    -x
                } else {
                    x
                }
            })?;
        }
        Some(v)
    } else {
        None
    };
    let roughness = material
        .roughness
        .as_deref()
        .map(|p| scalar(root, p))
        .transpose()?;
    let height = material
        .height
        .as_deref()
        .map(|p| scalar(root, p))
        .transpose()?;
    let ao = material
        .ao
        .as_deref()
        .map(|p| scalar(root, p))
        .transpose()?;
    Ok(Prepared {
        albedo: rgb,
        mean_linear,
        roughness,
        normal,
        height,
        ao,
    })
}
