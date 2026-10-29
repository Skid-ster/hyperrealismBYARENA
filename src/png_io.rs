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

/// Read a PNG or JPEG identified by its file signature, not its extension.
/// JPEG alpha is always opaque. CMYK JPEGs are rejected rather than silently
/// applying an unknown color profile or incorrect ink conversion.
pub fn read_material(path: &Path) -> Result<Rgba, String> {
    use std::io::Read;
    let mut header = [0u8; 8];
    let mut file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let n = file
        .read(&mut header)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    if n >= 8 && header == *b"\x89PNG\r\n\x1a\n" {
        return read(path);
    }
    if n < 2 || header[0..2] != [0xff, 0xd8] {
        return Err(format!("{}: expected PNG or JPEG", path.display()));
    }
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut decoder = jpeg_decoder::Decoder::new(file);
    decoder
        .read_info()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let info = decoder
        .info()
        .ok_or_else(|| format!("{}: missing JPEG metadata", path.display()))?;
    if info.width == 0 || info.height == 0 || info.width > 8192 || info.height > 8192 {
        return Err(format!(
            "{}: image dimensions outside 1..8192",
            path.display()
        ));
    }
    if info.pixel_format == jpeg_decoder::PixelFormat::CMYK32 {
        return Err(format!("{}: CMYK JPEG is not supported", path.display()));
    }
    let data = decoder
        .decode()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut pixels = Vec::with_capacity(info.width as usize * info.height as usize * 4);
    match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => {
            for v in data {
                pixels.extend_from_slice(&[v, v, v, 255]);
            }
        }
        jpeg_decoder::PixelFormat::RGB24 => {
            for p in data.chunks_exact(3) {
                pixels.extend_from_slice(&[p[0], p[1], p[2], 255]);
            }
        }
        jpeg_decoder::PixelFormat::CMYK32 => unreachable!(),
    }
    Ok(Rgba {
        width: info.width as usize,
        height: info.height as usize,
        pixels,
    })
}

/// Decode the red/gray channel as linear data, preserving 16-bit PNG precision.
/// JPEG and 8-bit PNG samples are interpreted as unorm, with no gamma transform.
pub fn read_linear_scalar(path: &Path) -> Result<(usize, usize, Vec<f32>), String> {
    read_linear_channel(path, 0)
}

/// Decode a linear RGB channel from PNG/JPEG. Grayscale is replicated.
/// PNG retains 16-bit precision; samples are never gamma-converted.
pub fn read_linear_channel(
    path: &Path,
    channel: usize,
) -> Result<(usize, usize, Vec<f32>), String> {
    if channel > 2 {
        return Err("linear channel must be 0, 1, or 2".into());
    }
    use std::io::Read;
    let mut file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut header = [0u8; 8];
    let count = file
        .read(&mut header)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    if count < 8 || header != *b"\x89PNG\r\n\x1a\n" {
        let image = read_material(path)?;
        return Ok((
            image.width,
            image.height,
            image
                .pixels
                .chunks_exact(4)
                .map(|p| p[channel] as f32 / 255.0)
                .collect(),
        ));
    }
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(png::Transformations::EXPAND);
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
    let channels = match info.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => return Err(format!("{}: unsupported PNG color type", path.display())),
    };
    let channel = if matches!(
        info.color_type,
        png::ColorType::Grayscale | png::ColorType::GrayscaleAlpha
    ) {
        0
    } else {
        channel
    };
    let (bytes_per_sample, div) = match info.bit_depth {
        png::BitDepth::Eight => (1, 255.0),
        png::BitDepth::Sixteen => (2, 65535.0),
        _ => return Err(format!("{}: unsupported PNG bit depth", path.display())),
    };
    let values = bytes[..info.buffer_size()]
        .chunks_exact(channels * bytes_per_sample)
        .map(|p| {
            if bytes_per_sample == 1 {
                p[channel] as f32 / div
            } else {
                let offset = channel * 2;
                u16::from_be_bytes([p[offset], p[offset + 1]]) as f32 / div
            }
        })
        .collect();
    Ok((info.width as usize, info.height as usize, values))
}
