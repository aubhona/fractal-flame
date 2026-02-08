use super::pixel::{Pixel, PixelData};

#[derive(Debug, Default)]
pub struct FractalImage {
    pub data: Vec<Pixel>,
    pub width: usize,
    pub height: usize,
}

impl FractalImage {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width
            .checked_mul(height)
            .expect("Image dimensions too large");
        Self {
            data: (0..size)
                .map(|_| Pixel::new(PixelData::default()))
                .collect(),
            width,
            height,
        }
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> Option<&Pixel> {
        if x < self.width && y < self.height {
            Some(&self.data[y * self.width + x])
        } else {
            None
        }
    }
}
