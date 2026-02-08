#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn r(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn theta(&self) -> f64 {
        (self.x / self.y).atan()
    }

    pub fn phi(&self) -> f64 {
        (self.y / self.x).atan()
    }
}
