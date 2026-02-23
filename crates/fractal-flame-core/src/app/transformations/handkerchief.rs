use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

pub struct Handkerchief {
    pub base: BaseAffineTransformation,
}

impl Handkerchief {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Handkerchief {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let theta = p.theta();
        let r = p.r();

        Point::new(r * (theta + r).sin(), r * (theta - r).cos())
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }

    fn get_name(&self) -> &'static str {
        "Handkerchief"
    }

    fn get_id(&self) -> &'static str {
        "handkerchief"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = r\sin(\theta + r),\quad y' = r\cos(\theta - r)"
    }
}
