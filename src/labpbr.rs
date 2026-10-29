//! LabPBR 1.3 channel construction. Values are explicitly represented so that
//! filtering a material identifier cannot accidentally create a different metal.

/// F0 is an already encoded dielectric value (0..=229), or a metal ID
/// (230..=255). Callers are responsible for selecting valid IDs for their shader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F0(pub u8);

/// Construct the normal/occlusion/height map from a unit normal in DirectX
/// convention (Y down), ambient occlusion and height, all in linear space.
/// Normals are normalized before encoding; a zero vector becomes [0, 0, 1].
pub fn normal_pixel(normal: [f32; 3], ao: f32, height: f32) -> [u8; 4] {
    let [x, y, z] = normal;
    let length = (x * x + y * y + z * z).sqrt();
    let (x, y) = if length.is_finite() && length > 1e-20 {
        (x / length, y / length)
    } else {
        (0.0, 0.0)
    };
    [
        unorm(x * 0.5 + 0.5),
        unorm(y * 0.5 + 0.5),
        unorm(ao),
        unorm(height),
    ]
}

/// Construct the specular map from perceptual roughness and raw LabPBR fields.
/// Roughness is linear [0,1]; R = 1 - sqrt(roughness).
///
/// Porosity and emission are passed through as raw bytes intentionally. LabPBR
/// multiplexes other meanings into these fields, including the no-emission
/// sentinel 255; do not treat all 256 values as a continuous emissive scale.
pub fn specular_pixel(roughness: f32, f0: F0, porosity: u8, emissive: u8) -> [u8; 4] {
    let roughness = finite_unit(roughness);
    [unorm(1.0 - roughness.sqrt()), f0.0, porosity, emissive]
}

fn finite_unit(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn unorm(value: f32) -> u8 {
    (finite_unit(value) * 255.0).round() as u8
}

/// Calculate a DirectX/Y-down tangent-space normal from a periodic height map.
/// The strength parameter scales the height gradient in pixel coordinates.
pub fn normals_from_height(
    height: &[f32],
    width: usize,
    height_px: usize,
    strength: f32,
) -> Result<Vec<[f32; 3]>, &'static str> {
    if width == 0 || height_px == 0 || width.checked_mul(height_px) != Some(height.len()) {
        return Err("height dimensions do not match data");
    }
    if !strength.is_finite() || height.iter().any(|v| !v.is_finite()) {
        return Err("height and strength must be finite");
    }
    let mut result = Vec::with_capacity(height.len());
    for y in 0..height_px {
        for x in 0..width {
            let left = height[y * width + (x + width - 1) % width];
            let right = height[y * width + (x + 1) % width];
            let up = height[((y + height_px - 1) % height_px) * width + x];
            let down = height[((y + 1) % height_px) * width + x];
            // The input image's y axis points down. The negative gradient gives
            // the outward-facing normal in the requested DirectX convention.
            let nx = -(right - left) * 0.5 * strength;
            let ny = -(down - up) * 0.5 * strength;
            let inv = (nx * nx + ny * ny + 1.0).sqrt().recip();
            result.push([nx * inv, ny * inv, inv]);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specular_channels_and_endpoints() {
        assert_eq!(specular_pixel(0.25, F0(230), 37, 255), [128, 230, 37, 255]);
        assert_eq!(specular_pixel(0.0, F0(42), 0, 0)[0], 255);
        assert_eq!(specular_pixel(1.0, F0(42), 0, 0)[0], 0);
        assert_eq!(specular_pixel(4.0, F0(42), 0, 0)[0], 0);
    }

    #[test]
    fn directx_normal_and_payload() {
        assert_eq!(normal_pixel([0.0, 0.0, 1.0], 1.0, 0.0), [128, 128, 255, 0]);
        assert_eq!(
            normal_pixel([0.0, 1.0, 0.0], 0.5, 1.0),
            [128, 255, 128, 255]
        );
        assert_eq!(normal_pixel([0.0, 0.0, 0.0], 1.0, 0.0), [128, 128, 255, 0]);
    }

    #[test]
    fn height_normals_wrap_and_point_downhill() {
        let n = normals_from_height(&[0.0, 0.0, 0.0, 1.0], 1, 4, 1.0).unwrap();
        assert!(n[0][1] > 0.0); // down=0, up=1 at the wrap seam
        assert!(n[2][1] < 0.0); // down=1, up=0
        assert!(normals_from_height(&[], 0, 1, 1.0).is_err());
        assert!(normals_from_height(&[f32::NAN], 1, 1, 1.0).is_err());
    }
}
