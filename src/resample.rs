//! Separable periodic Lanczos-3 resampling of linear scalar images.
//! Filter continuous channels separately; never apply this to categorical metal IDs.
use std::f64::consts::PI;

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

fn weights(source: usize, target: usize, wrap: bool) -> Vec<Vec<(usize, f64)>> {
    let scale = (source as f64 / target as f64).max(1.0);
    (0..target)
        .map(|i| {
            let center = (i as f64 + 0.5) * source as f64 / target as f64 - 0.5;
            let lo = (center - 3.0 * scale).ceil() as i64;
            let hi = (center + 3.0 * scale).floor() as i64;
            let mut w: Vec<_> = (lo..=hi)
                .filter_map(|j| {
                    let d = (center - j as f64) / scale;
                    if d.abs() >= 3.0 {
                        None
                    } else {
                        let index = if wrap {
                            j.rem_euclid(source as i64) as usize
                        } else {
                            j.clamp(0, source as i64 - 1) as usize
                        };
                        Some((index, sinc(d) * sinc(d / 3.0)))
                    }
                })
                .collect();
            let sum: f64 = w.iter().map(|x| x.1).sum();
            for x in &mut w {
                x.1 /= sum;
            }
            w
        })
        .collect()
}

/// Resize one scalar channel with periodic boundaries. Output may overshoot
/// because Lanczos has negative lobes: clamp only when encoding a channel.
fn resize(
    input: &[f32],
    width: usize,
    height: usize,
    out_width: usize,
    out_height: usize,
    wrap: bool,
) -> Result<Vec<f32>, &'static str> {
    if width == 0
        || height == 0
        || out_width == 0
        || out_height == 0
        || width.checked_mul(height) != Some(input.len())
    {
        return Err("invalid input or output dimensions");
    }
    let intermediate_len = out_width.checked_mul(height).ok_or("dimensions overflow")?;
    let output_len = out_width
        .checked_mul(out_height)
        .ok_or("dimensions overflow")?;
    if width > i64::MAX as usize
        || height > i64::MAX as usize
        || input.iter().any(|x| !x.is_finite())
    {
        return Err("invalid input data");
    }
    let wx = weights(width, out_width, wrap);
    let wy = weights(height, out_height, wrap);
    let mut temp = vec![0.0f32; intermediate_len];
    for y in 0..height {
        for (x, w) in wx.iter().enumerate() {
            temp[y * out_width + x] = w
                .iter()
                .map(|&(i, v)| input[y * width + i] as f64 * v)
                .sum::<f64>() as f32;
        }
    }
    let mut output = vec![0.0f32; output_len];
    for (y, w) in wy.iter().enumerate() {
        for x in 0..out_width {
            output[y * out_width + x] = w
                .iter()
                .map(|&(i, v)| temp[i * out_width + x] as f64 * v)
                .sum::<f64>() as f32;
        }
    }
    Ok(output)
}

/// Resize one scalar channel with periodic boundaries for seamless block tiles.
pub fn resize_wrap(
    input: &[f32],
    width: usize,
    height: usize,
    out_width: usize,
    out_height: usize,
) -> Result<Vec<f32>, &'static str> {
    resize(input, width, height, out_width, out_height, true)
}

/// Resize one scalar channel with clamped edges for non-tiling entity atlases.
pub fn resize_clamp(
    input: &[f32],
    width: usize,
    height: usize,
    out_width: usize,
    out_height: usize,
) -> Result<Vec<f32>, &'static str> {
    resize(input, width, height, out_width, out_height, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constant_is_preserved() {
        let result = resize_wrap(&vec![0.7; 16 * 16], 16, 16, 3, 5).unwrap();
        assert!(result.iter().all(|&v| (v - 0.7).abs() < 1e-6));
    }
    #[test]
    fn clamped_edges_do_not_wrap_entity_atlases() {
        let edge = [1.0, 1.0, 0.0, 0.0];
        let clamped = resize_clamp(&edge, 4, 1, 2, 1).unwrap();
        let wrapped = resize_wrap(&edge, 4, 1, 2, 1).unwrap();
        assert!(clamped[0] > wrapped[0]);
        assert!(clamped[1] < wrapped[1]);
    }

    #[test]
    fn integer_scale_shift_is_periodic() {
        let data = [1.0, 0.2, 0.5, 0.1, 0.4, 0.3, 0.8, 0.9];
        let mut shifted = data;
        shifted.rotate_right(2);
        let a = resize_wrap(&data, 8, 1, 4, 1).unwrap();
        let mut b = resize_wrap(&shifted, 8, 1, 4, 1).unwrap();
        b.rotate_left(1);
        assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6));
    }
    #[test]
    fn invalid_inputs_fail() {
        assert!(resize_wrap(&[0.0], 2, 1, 1, 1).is_err());
        assert!(resize_wrap(&[f32::NAN], 1, 1, 1, 1).is_err());
        assert!(resize_wrap(&[0.0], 1, 1, 0, 1).is_err());
    }
}
