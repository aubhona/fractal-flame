use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

pub struct Spherical {
    pub base: BaseAffineTransformation,
}

impl Spherical {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Spherical {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let r2 = p.x.powi(2) + p.y.powi(2);

        Point::new(p.x / r2, p.y / r2)
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }
}
