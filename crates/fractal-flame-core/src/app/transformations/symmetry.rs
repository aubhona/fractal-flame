use crate::domain::transformation::Transformation;
use crate::domain::{Color, Point};

pub struct Symmetry {
    pub theta: f64,
    pub weight: f64,
    pub color: Color,
}

impl Symmetry {
    pub fn new(theta: f64) -> Self {
        Self {
            theta,
            weight: 1.0,
            color: Color::default(),
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Transformation for Symmetry {
    fn apply(&self, point: &Point) -> Point {
        let cos_theta = self.theta.cos();
        let sin_theta = self.theta.sin();

        Point::new(
            point.x * cos_theta - point.y * sin_theta,
            point.x * sin_theta + point.y * cos_theta,
        )
    }

    fn weight(&self) -> f64 {
        self.weight
    }

    fn color(&self) -> &Color {
        &self.color
    }
}
