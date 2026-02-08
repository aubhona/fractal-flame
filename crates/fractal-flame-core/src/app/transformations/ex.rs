use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

pub struct Ex {
    pub base: BaseAffineTransformation,
}

impl Ex {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Ex {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let theta = p.theta();
        let r = p.r();
        let p0 = (theta + r).sin();
        let p1 = (theta - r).cos();

        Point::new(r * (p0.powi(3) + p1.powi(3)), r * (p0.powi(3) - p1.powi(3)))
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }
}
