use crate::domain::{Color, Point};

pub trait Transformation {
    fn apply(&self, point: &Point) -> Point;
    fn weight(&self) -> f64;
    fn color(&self) -> &Color;
}
