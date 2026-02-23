use std::sync::Arc;

use fractal_flame_core::app::renderer::{get_random_color, Renderer};
use fractal_flame_core::app::transformations::{
    base_affine_transformation::BaseAffineTransformation,
    diamond::Diamond,
    disc::Disc,
    ex::Ex,
    handkerchief::Handkerchief,
    heart::Heart,
    horseshoe::Horseshoe,
    hyperbolic::Hyperbolic,
    linear::Linear,
    polar::Polar,
    sinusoidal::Sinusoidal,
    spherical::Spherical,
    spiral::Spiral,
    swirl::Swirl,
};
use fractal_flame_core::domain::transformation::Transformation;
use fractal_flame_core::domain::{FractalImage, Rect};
use fractal_flame_core::infra::random;

use super::config::Config;

fn search_affine_transformation(
    config: &Config,
) -> Result<BaseAffineTransformation, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let a = random::generate_f64(-1.5, 1.5, true)?;
        let b = random::generate_f64(-1.5, 1.5, true)?;
        let c = random::generate_f64(-2.0, 2.0, true)?;
        let d = random::generate_f64(-1.5, 1.5, true)?;
        let e = random::generate_f64(-1.5, 1.5, true)?;
        let f = random::generate_f64(-2.0, 2.0, true)?;
        let color = get_random_color()?;

        let det = a * e - b * d;
        if (a * a + d * d) < 1.0
            && (b * b + e * e) < 1.0
            && (a * a + b * b + d * d + e * e) < 1.0 + det * det
        {
            let weight = random::generate_f64(
                config.transformation_min_weight,
                config.transformation_max_weight,
                true,
            )?;
            return Ok(BaseAffineTransformation::new(
                weight, color, a, b, c, d, e, f,
            ));
        }
    }
}

fn initialize_transformations(
    config: &Config,
) -> Result<Vec<Box<dyn Transformation + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
{
    let mut transformations: Vec<Box<dyn Transformation + Send + Sync>> = vec![];

    let types: Vec<fn(BaseAffineTransformation) -> Box<dyn Transformation + Send + Sync>> = vec![
        |base| Box::new(Diamond { base }),
        |base| Box::new(Disc::new(base)),
        |base| Box::new(Ex::new(base)),
        |base| Box::new(Heart::new(base)),
        |base| Box::new(Horseshoe::new(base)),
        |base| Box::new(Spherical::new(base)),
        |base| Box::new(Swirl::new(base)),
        |base| Box::new(Linear::new(base)),
        |base| Box::new(Polar::new(base)),
        |base| Box::new(Spiral::new(base)),
        |base| Box::new(Handkerchief::new(base)),
        |base| Box::new(Hyperbolic::new(base)),
        |base| Box::new(Sinusoidal::new(base)),
    ];

    for create in types {
        let base = search_affine_transformation(config)?;
        transformations.push(create(base));
    }

    Ok(transformations)
}

#[derive(Clone)]
pub struct Dependencies {
    pub config: Config,
    pub renderer: Arc<Renderer>,
    pub transformations: Arc<Vec<Box<dyn Transformation + Send + Sync>>>,
}

impl Dependencies {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let transformations = initialize_transformations(&config)?;

        let canvas = FractalImage::new(config.width, config.height);
        let aspect = config.width as f64 / config.height as f64;
        let world = Rect::new(-aspect, -1.0, 2.0 * aspect, 2.0);

        let renderer = Renderer::new(
            canvas,
            world,
            transformations,
            config.samples,
            config.iter_per_sample,
            config.symmetry,
            config.gamma,
            config.max_threads,
        );

        let transformations = renderer.transformations.clone();

        Ok(Self {
            config,
            renderer: Arc::new(renderer),
            transformations,
        })
    }
}
