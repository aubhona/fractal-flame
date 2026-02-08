use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

pub struct Sinusoidal {
    pub base: BaseAffineTransformation,
}

impl Sinusoidal {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Sinusoidal {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        Point::new(p.x.sin(), p.y.cos())
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }
}
