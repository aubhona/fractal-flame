use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

#[derive(Clone)]
pub struct Spiral {
    pub base: BaseAffineTransformation,
}

impl Spiral {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Spiral {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let theta = p.theta();
        let r = p.r();

        Point::new(
            (1.0 / r) * theta.cos() + r.sin(),
            (1.0 / r) * theta.sin() - r.cos(),
        )
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }

    fn get_name(&self) -> &'static str {
        "Spiral"
    }

    fn get_id(&self) -> &'static str {
        "spiral"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = \frac{\cos\theta}{r} + \sin r,\quad y' = \frac{\sin\theta}{r} - \cos r"
    }
}
