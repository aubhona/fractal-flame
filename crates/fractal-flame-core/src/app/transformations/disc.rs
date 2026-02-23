use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

pub struct Disc {
    pub base: BaseAffineTransformation,
}

impl Disc {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Disc {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let theta = p.theta();
        let r = p.r();
        let pi = std::f64::consts::PI;

        Point::new(theta / pi * (pi * r).sin(), theta / pi * (pi * r).cos())
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }

    fn get_name(&self) -> &'static str {
        "Disc"
    }

    fn get_id(&self) -> &'static str {
        "disc"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = \frac{\theta}{\pi}\sin(\pi r),\quad y' = \frac{\theta}{\pi}\cos(\pi r)"
    }
}
