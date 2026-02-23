use crate::domain::{Color, Point};

pub trait Transformation {
    fn apply(&self, point: &Point) -> Point;
    fn weight(&self) -> f64;
    fn color(&self) -> &Color;

    fn get_name(&self) -> &'static str;
    fn get_id(&self) -> &'static str;
    fn get_formula(&self) -> &'static str;
}
