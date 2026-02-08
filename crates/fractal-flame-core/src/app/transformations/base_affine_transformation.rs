use crate::domain::{Color, Point, Transformation};

pub struct BaseAffineTransformation {
    pub weight: f64,
    pub color: Color,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl BaseAffineTransformation {
    pub fn new(weight: f64, color: Color, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self {
            weight,
            color,
            a,
            b,
            c,
            d,
            e,
            f,
        }
    }
}

impl Transformation for BaseAffineTransformation {
    fn apply(&self, p: &Point) -> Point {
        Point {
            x: self.a * p.x + self.b * p.y + self.c,
            y: self.d * p.x + self.e * p.y + self.f,
        }
    }

    fn weight(&self) -> f64 {
        self.weight
    }

    fn color(&self) -> &Color {
        &self.color
    }
}
