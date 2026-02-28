use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

#[derive(Clone)]
pub struct Diamond {
    pub base: BaseAffineTransformation,
}

impl Transformation for Diamond {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let theta = p.theta();
        let r = p.r();

        Point::new(theta.sin() * r.cos(), r.sin() * theta.cos())
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }

    fn get_name(&self) -> &'static str {
        "Diamond"
    }

    fn get_id(&self) -> &'static str {
        "diamond"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = \sin\theta\cos r,\quad y' = \sin r\cos\theta"
    }
}
