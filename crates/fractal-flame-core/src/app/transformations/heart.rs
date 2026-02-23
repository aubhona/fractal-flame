use super::base_affine_transformation::BaseAffineTransformation;
use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

pub struct Heart {
    pub base: BaseAffineTransformation,
}

impl Heart {
    pub fn new(base: BaseAffineTransformation) -> Self {
        Self { base }
    }
}

impl Transformation for Heart {
    fn apply(&self, point: &Point) -> Point {
        let p = self.base.apply(point);

        let theta = p.theta();
        let r = p.r();

        Point::new(r * (theta * r).sin(), -r * (theta * r).cos())
    }

    fn weight(&self) -> f64 {
        self.base.weight()
    }

    fn color(&self) -> &Color {
        self.base.color()
    }

    fn get_name(&self) -> &'static str {
        "Heart"
    }

    fn get_id(&self) -> &'static str {
        "heart"
    }

    fn get_formula(&self) -> &'static str {
        r"x' = r\sin(\theta r),\quad y' = -r\cos(\theta r)"
    }
}
