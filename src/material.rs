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

pub fn scalar(root: &Path, name: &str) -> Result<Vec<f32>, String> {
    scalar_channel(root, name, 0)
}

/// Reconstruct a conservative periodic height approximation from a tangent-space
/// normal map. Independent row/column path integrations are averaged, detrended,
/// then robustly normalized; global PBR strength still controls final amplitude.
fn height_from_normal(normal: &[Vec<f32>; 3]) -> Result<Vec<f32>, String> {
    let n = WORK * WORK;
    if normal.iter().any(|channel| channel.len() != n) {
        return Err("normal dimensions do not match height reconstruction".into());
    }
    let mut horizontal = vec![0.0f32; n];
    let mut vertical = vec![0.0f32; n];
    let slope = |component: f32, z: f32| (-component / z.max(0.2)).clamp(-4.0, 4.0);
    for y in 0..WORK {
        let start = y * WORK;
        let mut mean = 0.0f64;
        for x in 0..WORK {
            let i = start + x;
            mean += slope(normal[0][i], normal[2][i]) as f64;
        }
        let mean = (mean / WORK as f64) as f32;
        let mut height = 0.0f32;
        for x in 0..WORK {
            let i = start + x;
            height += slope(normal[0][i], normal[2][i]) - mean;
            horizontal[i] = height;
        }
    }
    for x in 0..WORK {
        let mut mean = 0.0f64;
        for y in 0..WORK {
            let i = y * WORK + x;
            mean += slope(normal[1][i], normal[2][i]) as f64;
        }
        let mean = (mean / WORK as f64) as f32;
        let mut height = 0.0f32;
        for y in 0..WORK {
            let i = y * WORK + x;
            height += slope(normal[1][i], normal[2][i]) - mean;
            vertical[i] = height;
        }
    }
    let mut combined = Vec::with_capacity(n);
    for i in 0..n {
        combined.push((horizontal[i] + vertical[i]) * 0.5);
    }
    let mut ordered = combined.clone();
    ordered.sort_unstable_by(f32::total_cmp);
    let low = ordered[n / 100];
    let high = ordered[n * 99 / 100];
    if !low.is_finite() || !high.is_finite() || high - low < 1e-6 {
        return Ok(vec![0.5; n]);
    }
    let scale = 1.0 / (high - low);
    for value in &mut combined {
        *value = ((*value - low) * scale).clamp(0.0, 1.0);
    }
    Ok(combined)
}

fn roughness_from_height(height: &[f32]) -> Vec<f32> {
    height.iter().map(|h| 0.86 + (1.0 - h) * 0.14).collect()
}

fn apply_roughness_bias(values: &mut [f32], bias: f32) {
    for value in values {
        *value = (*value + bias).clamp(0.0, 1.0);
    }
}

fn scalar_channel(root: &Path, name: &str, channel: usize) -> Result<Vec<f32>, String> {
    let path = manifest::safe_file(root, name)?;
    let (width, height, source) = png_io::read_linear_channel(&path, channel)?;
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
    let mut height = material
        .height
        .as_deref()
        .map(|p| scalar(root, p))
        .transpose()?;
    if material.height_from_normal {
        height = Some(height_from_normal(
            normal.as_ref().expect("manifest validation requires a normal map"),
        )?);
    }
    let mut roughness = if material.roughness_from_height {
        let height = height
            .as_ref()
            .expect("manifest validation requires a height map for roughness derivation");
        Some(roughness_from_height(height))
    } else if let Some(arm) = &material.arm {
        Some(scalar_channel(root, arm, 1)?)
    } else {
        material
            .roughness
            .as_deref()
            .map(|p| scalar(root, p))
            .transpose()?
    };
    if let Some(values) = &mut roughness {
        apply_roughness_bias(values, material.roughness_bias);
    }
    let ao = if let Some(arm) = &material.arm {
        Some(scalar_channel(root, arm, 0)?)
    } else {
        material
            .ao
            .as_deref()
            .map(|p| scalar(root, p))
            .transpose()?
    };
    let mut prepared = Prepared {
        albedo: rgb,
        mean_linear,
        roughness,
        normal,
        height,
        ao,
    };
    if material.rotate_cw {
        // Rotate every registered channel together. Normal XY is already in
        // DirectX/image coordinates: clockwise maps (x,y) to (-y,x).
        fn rotate(plane: &mut Vec<f32>) {
            let mut out = vec![0.0; WORK * WORK];
            for y in 0..WORK {
                for x in 0..WORK {
                    out[y * WORK + x] = plane[(WORK - 1 - x) * WORK + y];
                }
            }
            *plane = out;
        }
        for c in &mut prepared.albedo {
            rotate(c);
        }
        for channel in [
            &mut prepared.roughness,
            &mut prepared.height,
            &mut prepared.ao,
        ] {
            if let Some(plane) = channel {
                rotate(plane);
            }
        }
        if let Some(normals) = &mut prepared.normal {
            for plane in normals.iter_mut() {
                rotate(plane);
            }
            let [x, y, _] = normals;
            for (x, y) in x.iter_mut().zip(y.iter_mut()) {
                let old_x = *x;
                *x = -*y;
                *y = old_x;
            }
        }
    }
    Ok(prepared)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_integration_produces_bounded_nonflat_periodic_height() {
        let n = WORK * WORK;
        let mut normal = [vec![0.0; n], vec![0.0; n], vec![1.0; n]];
        for y in 0..WORK {
            for x in 0..WORK {
                let i = y * WORK + x;
                let angle = std::f32::consts::TAU * x as f32 / WORK as f32;
                normal[0][i] = 0.35 * angle.cos();
                normal[2][i] = (1.0 - normal[0][i] * normal[0][i]).sqrt();
            }
        }
        let height = height_from_normal(&normal).unwrap();
        assert_eq!(height.len(), n);
        let low = height.iter().copied().fold(f32::INFINITY, f32::min);
        let high = height.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert_eq!(low, 0.0);
        assert_eq!(high, 1.0);
        assert!(height[WORK * 3 / 4] > height[0] + 0.25);
        let seam_left = height[WORK * (WORK - 1)];
        let seam_right = height[WORK * WORK - 1];
        assert!((seam_left - seam_right).abs() < 0.05);
    }

    #[test]
    fn stone_height_adds_joint_roughness_and_bias_is_clamped() {
        let roughness = roughness_from_height(&[0.0, 0.5, 1.0]);
        assert!((roughness[0] - 1.0).abs() < 1e-6);
        assert!((roughness[2] - 0.86).abs() < 1e-6);
        let mut calibrated = roughness;
        apply_roughness_bias(&mut calibrated, -0.2);
        assert!((calibrated[0] - 0.8).abs() < 1e-6);
        apply_roughness_bias(&mut calibrated, -1.0);
        assert!(calibrated.iter().all(|&v| v == 0.0));
    }
}
