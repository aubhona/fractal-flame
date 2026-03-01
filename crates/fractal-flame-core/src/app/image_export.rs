use crate::domain::FractalImage;
use image::{
    ColorType, ImageEncoder,
    codecs::png::{CompressionType, FilterType, PngEncoder},
};
use std::io::Cursor;

/// Converts FractalImage to PNG bytes. Expects gamma correction already applied.
pub fn fractal_image_to_png(canvas: &FractalImage) -> Result<Vec<u8>, ImageExportError> {
    let mut raw = Vec::with_capacity(canvas.width * canvas.height * 4);

    for y in 0..canvas.height {
        for x in 0..canvas.width {
            if let Some(pixel) = canvas.pixel_at(x, y) {
                let data = pixel
                    .read()
                    .map_err(|_| ImageExportError::PixelReadFailed)?;
                raw.push(data.color.r);
                raw.push(data.color.g);
                raw.push(data.color.b);
                raw.push(255);
            }
        }
    }

    let mut buf = Cursor::new(Vec::new());
    let encoder = PngEncoder::new(&mut buf);
    encoder
        .write_image(
            &raw,
            canvas.width as u32,
            canvas.height as u32,
            ColorType::Rgba8.into(),
        )
        .map_err(ImageExportError::EncodeFailed)?;

    Ok(buf.into_inner())
}

/// Snapshot of the canvas mid-render: reads pixels non-destructively, applies gamma on the fly.
/// Optimised for speed: single lock-acquisition pass + fast PNG compression.
pub fn fractal_image_to_intermediate_png(
    canvas: &FractalImage,
    gamma: f64,
) -> Result<Vec<u8>, ImageExportError> {
    let total = canvas.width * canvas.height;

    let mut pixel_buf: Vec<(u8, u8, u8, i32)> = Vec::with_capacity(total);
    let mut max_normal = 0.0f64;

    for y in 0..canvas.height {
        for x in 0..canvas.width {
            if let Some(pixel) = canvas.pixel_at(x, y) {
                let data = pixel
                    .read()
                    .map_err(|_| ImageExportError::PixelReadFailed)?;
                let hc = data.hit_count;
                pixel_buf.push((data.color.r, data.color.g, data.color.b, hc));
                if hc > 0 {
                    let normal = (hc as f64).log10();
                    if normal > max_normal {
                        max_normal = normal;
                    }
                }
            }
        }
    }

    let inv_gamma = 1.0 / gamma;
    let mut raw = Vec::with_capacity(total * 4);
    for &(r, g, b, hc) in &pixel_buf {
        if hc > 0 && max_normal > 0.0 {
            let normal = (hc as f64).log10() / max_normal;
            let gf = normal.powf(inv_gamma);
            raw.push(((r as f64) * gf) as u8);
            raw.push(((g as f64) * gf) as u8);
            raw.push(((b as f64) * gf) as u8);
        } else {
            raw.push(0);
            raw.push(0);
            raw.push(0);
        }
        raw.push(255);
    }

    let mut buf = Cursor::new(Vec::new());
    let encoder =
        PngEncoder::new_with_quality(&mut buf, CompressionType::Fast, FilterType::NoFilter);
    encoder
        .write_image(
            &raw,
            canvas.width as u32,
            canvas.height as u32,
            ColorType::Rgba8.into(),
        )
        .map_err(ImageExportError::EncodeFailed)?;

    Ok(buf.into_inner())
}

#[derive(Debug, thiserror::Error)]
pub enum ImageExportError {
    #[error("Failed to read pixel data")]
    PixelReadFailed,
    #[error("Failed to encode PNG: {0}")]
    EncodeFailed(#[from] image::ImageError),
}
