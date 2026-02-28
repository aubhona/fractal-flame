use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

#[derive(Clone)]
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

    fn get_name(&self) -> &'static str {
        "Ex"
    }

    fn get_id(&self) -> &'static str {
        "ex"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = r(\sin^3(\theta+r) + \cos^3(\theta-r)),\quad y' = r(\sin^3(\theta+r) - \cos^3(\theta-r))"
    }
}
