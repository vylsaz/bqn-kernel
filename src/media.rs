use base64::{engine::general_purpose::STANDARD, write::EncoderWriter, Engine};
use hound::{SampleFormat, WavSpec, WavWriter};

pub fn base64_png(width: u32, height: u32, channels: u32, data: &[u8]) -> Result<String, String> {
    let color = match channels {
        1 => png::ColorType::Grayscale,
        2 => png::ColorType::GrayscaleAlpha,
        3 => png::ColorType::Rgb,
        4 => png::ColorType::Rgba,
        _ => unreachable!(),
    };
    let mut out = Vec::new();
    {
        let w = EncoderWriter::new(&mut out, &STANDARD);
        let mut enc = png::Encoder::new(w, width, height);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_color(color);
        enc.set_compression(png::Compression::Fast);
        let mut writer = enc.write_header().map_err(|e| format!("PNG: {e}"))?;
        writer
            .write_image_data(data)
            .map_err(|e| format!("PNG: {e}"))?
    }
    String::from_utf8(out).map_err(|e| format!("Base64: {e}"))
}

pub fn base64_wav(channels: u16, sample_rate: u32, samples: &[f64]) -> Result<String, String> {
    let bits_per_sample = 32;
    let sample_format = SampleFormat::Float;
    let mut bytes = std::io::Cursor::new(Vec::new());
    {
        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample,
            sample_format,
        };
        let mut writer = WavWriter::new(&mut bytes, spec).unwrap();
        for &s in samples {
            writer
                .write_sample(s as f32)
                .map_err(|e| format!("WAV: {e}"))?;
        }
        writer.finalize().map_err(|e| format!("WAV: {e}"))?;
    }
    Ok(STANDARD.encode(bytes.into_inner()))
}
