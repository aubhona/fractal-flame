use crate::domain::FractalImage;
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
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
        .write_image(&raw, canvas.width as u32, canvas.height as u32, ColorType::Rgba8.into())
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
