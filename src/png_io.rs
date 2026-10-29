use std::{fs::File, io::BufWriter, path::Path};

pub struct Rgba {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

pub fn read(path: &Path) -> Result<Rgba, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let dimensions = reader.info();
    if dimensions.width == 0
        || dimensions.height == 0
        || dimensions.width > 8192
        || dimensions.height > 8192
    {
        return Err(format!(
            "{}: image dimensions outside 1..8192",
            path.display()
        ));
    }
    let mut bytes = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut bytes)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    if info.width == 0 || info.height == 0 || info.width > 8192 || info.height > 8192 {
        return Err(format!(
            "{}: image dimensions outside 1..8192",
            path.display()
        ));
    }
    let input = &bytes[..info.buffer_size()];
    let mut pixels = Vec::with_capacity(info.width as usize * info.height as usize * 4);
    match info.color_type {
        png::ColorType::Rgba => pixels.extend_from_slice(input),
        png::ColorType::Rgb => {
            for p in input.chunks_exact(3) {
                pixels.extend_from_slice(&[p[0], p[1], p[2], 255]);
            }
        }
        png::ColorType::Grayscale => {
            for &p in input {
                pixels.extend_from_slice(&[p, p, p, 255]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for p in input.chunks_exact(2) {
                pixels.extend_from_slice(&[p[0], p[0], p[0], p[1]]);
            }
        }
        _ => return Err(format!("{}: unsupported PNG format", path.display())),
    }
    Ok(Rgba {
        width: info.width as usize,
        height: info.height as usize,
        pixels,
    })
}
pub fn write(path: &Path, width: usize, height: usize, pixels: &[u8]) -> Result<(), String> {
    if width.checked_mul(height).and_then(|n| n.checked_mul(4)) != Some(pixels.len()) {
        return Err("incorrect PNG dimensions".into());
    }
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(pixels).map_err(|e| e.to_string())
}
