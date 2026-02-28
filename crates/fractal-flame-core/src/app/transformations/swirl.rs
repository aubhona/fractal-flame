use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

#[derive(Clone)]
pub struct Swirl {
    pub base: BaseAffineTransformation,
}

impl Swirl {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Swirl {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let r2 = p.x.powi(2) + p.y.powi(2);

        Point::new(
            p.x * r2.sin() - p.y * r2.cos(),
            p.x * r2.cos() + p.y * r2.sin(),
        )
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }

    fn get_name(&self) -> &'static str {
        "Swirl"
    }

    fn get_id(&self) -> &'static str {
        "swirl"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = x\sin(r^2) - y\cos(r^2),\quad y' = x\cos(r^2) + y\sin(r^2)"
    }
}
