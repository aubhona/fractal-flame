use std::sync::RwLock;

use super::color::Color;

#[derive(Debug, Default)]
pub struct PixelData {
    pub color: Color,
    pub hit_count: i32,
    pub normal: f64,
}

pub type Pixel = RwLock<PixelData>;
