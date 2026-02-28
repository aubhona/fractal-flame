use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

#[derive(Clone)]
pub struct Hyperbolic {
    pub base: BaseAffineTransformation,
}

impl Hyperbolic {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Hyperbolic {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let theta = p.theta();
        let r = p.r();

        Point::new(theta.sin() / r, r * theta.cos())
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }

    fn get_name(&self) -> &'static str {
        "Hyperbolic"
    }

    fn get_id(&self) -> &'static str {
        "hyperbolic"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = \frac{\sin\theta}{r},\quad y' = r\cos\theta"
    }
}
