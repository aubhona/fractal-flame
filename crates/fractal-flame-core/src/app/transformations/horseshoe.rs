use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

#[derive(Clone)]
pub struct Horseshoe {
    pub base: BaseAffineTransformation,
}

impl Horseshoe {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Horseshoe {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let r = p.r();

        Point::new((p.x - p.y) * (p.x + p.y) / r, 2.0 * p.x * p.y / r)
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }

    fn get_name(&self) -> &'static str {
        "Horseshoe"
    }

    fn get_id(&self) -> &'static str {
        "horseshoe"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = \frac{(x-y)(x+y)}{r},\quad y' = \frac{2xy}{r}"
    }
}
