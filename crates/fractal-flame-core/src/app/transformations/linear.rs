use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

pub struct Linear {
    pub base: BaseAffineTransformation,
}

impl Linear {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Linear {
    fn apply(&self, point: &Point) -> Point {
        self.base.apply(point)
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }

    fn get_name(&self) -> &'static str {
        "Linear"
    }

    fn get_id(&self) -> &'static str {
        "linear"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = ax + by + c,\quad y' = dx + ey + f"
    }
}
