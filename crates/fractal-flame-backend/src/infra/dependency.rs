use std::sync::Arc;

use fractal_flame_core::app::renderer::get_random_color;
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
use fractal_flame_core::infra::random;

use super::config::Config;
use super::minio::{MinioClient, MinioConfig};
use super::redis::RedisPool;

fn clone_transformation(
    t: &Box<dyn Transformation + Send + Sync>,
) -> Box<dyn Transformation + Send + Sync> {
    let any: &dyn std::any::Any = t.as_ref();
    if let Some(d) = any.downcast_ref::<Diamond>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Disc>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Ex>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Heart>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Horseshoe>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Spherical>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Swirl>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Linear>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Polar>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Spiral>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Handkerchief>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Hyperbolic>() {
        return Box::new(d.clone());
    }
    if let Some(d) = any.downcast_ref::<Sinusoidal>() {
        return Box::new(d.clone());
    }
    unreachable!("Unknown transformation type")
}

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
    pub transformations: Arc<Vec<Box<dyn Transformation + Send + Sync>>>,
    pub redis: Option<Arc<RedisPool>>,
    pub minio: Option<Arc<MinioClient>>,
}

/// Фильтрует трансформации по списку variation_ids и возвращает клоны.
pub fn filter_transformations_by_ids(
    transformations: &Arc<Vec<Box<dyn Transformation + Send + Sync>>>,
    ids: &[String],
) -> Result<Vec<Box<dyn Transformation + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
{
    let id_set: std::collections::HashSet<_> = ids.iter().map(|s| s.as_str()).collect();
    let mut result = Vec::new();
    for t in transformations.iter() {
        if id_set.contains(t.get_id()) {
            result.push(clone_transformation(t));
        }
    }
    if result.is_empty() {
        return Err("No variations selected".into());
    }
    Ok(result)
}

impl Dependencies {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let transformations = initialize_transformations(&config)?;

        let redis = std::env::var("REDIS_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|url| RedisPool::from_url(&url))
            .transpose()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?
            .map(Arc::new);

        if redis.is_some() {
            tracing::info!("Redis pool configured");
        }

        let minio = (|| {
            let endpoint = std::env::var("MINIO_ENDPOINT").ok().filter(|s| !s.is_empty())?;
            let access_key = std::env::var("MINIO_ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string());
            let secret_key = std::env::var("MINIO_SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string());
            let bucket = std::env::var("MINIO_BUCKET")
                .unwrap_or_else(|_| "fractal-flame".to_string());
            let region = std::env::var("MINIO_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string());

            let client = MinioClient::new(MinioConfig {
                endpoint,
                access_key,
                secret_key,
                bucket,
                region,
            })
            .ok()?;
            tracing::info!("MinIO connected");
            Some(Arc::new(client))
        })();

        Ok(Self {
            config,
            transformations: Arc::new(transformations),
            redis,
            minio,
        })
    }
}
