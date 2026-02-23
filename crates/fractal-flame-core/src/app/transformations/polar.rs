use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

pub struct Polar {
    pub base: BaseAffineTransformation,
}

impl Polar {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Polar {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let theta = p.theta();
        let r = p.r();
        let pi = std::f64::consts::PI;

        Point::new(theta / pi, r - 1.0)
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }

    fn get_name(&self) -> &'static str {
        "Polar"
    }

    fn get_id(&self) -> &'static str {
        "polar"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = \frac{\theta}{\pi},\quad y' = r - 1"
    }
}
