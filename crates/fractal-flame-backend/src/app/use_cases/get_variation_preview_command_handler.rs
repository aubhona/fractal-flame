use std::sync::Arc;

use fractal_flame_core::app::image_export::fractal_image_to_png;
use fractal_flame_core::app::renderer::Renderer;
use fractal_flame_core::app::transformations::{
    base_affine_transformation::BaseAffineTransformation, diamond::Diamond, disc::Disc, ex::Ex,
    handkerchief::Handkerchief, heart::Heart, horseshoe::Horseshoe, hyperbolic::Hyperbolic,
    linear::Linear, polar::Polar, sinusoidal::Sinusoidal, spherical::Spherical, spiral::Spiral,
    swirl::Swirl,
};
use fractal_flame_core::domain::transformation::Transformation;
use fractal_flame_core::domain::{FractalImage, Rect};

use crate::app::services::minio_key_service::MinioKeyService;
use crate::infra::config::Config;
use crate::infra::minio::MinioClient;

use super::get_variation_preview_command::GetVariationPreviewCommand;

fn preview_base_affines() -> Vec<BaseAffineTransformation> {
    let color = fractal_flame_core::domain::Color {
        r: 180,
        g: 100,
        b: 220,
    };
    vec![
        BaseAffineTransformation::new(1.0, color, 0.4, 0.0, -0.3, 0.0, 0.4, 0.0),
        BaseAffineTransformation::new(1.0, color, 0.35, -0.2, 0.3, 0.2, 0.35, 0.0),
        BaseAffineTransformation::new(1.0, color, 0.3, 0.0, 0.0, 0.0, 0.3, -0.4),
    ]
}

fn create_preview_transformations(
    id: &str,
) -> Result<Vec<Box<dyn Transformation + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
    let bases = preview_base_affines();
    let mut transformations = Vec::with_capacity(bases.len());
    for base in bases {
        let t: Box<dyn Transformation + Send + Sync> = match id {
            "diamond" => Box::new(Diamond { base }),
            "disc" => Box::new(Disc::new(base)),
            "ex" => Box::new(Ex::new(base)),
            "heart" => Box::new(Heart::new(base)),
            "horseshoe" => Box::new(Horseshoe::new(base)),
            "spherical" => Box::new(Spherical::new(base)),
            "swirl" => Box::new(Swirl::new(base)),
            "linear" => Box::new(Linear::new(base)),
            "polar" => Box::new(Polar::new(base)),
            "spiral" => Box::new(Spiral::new(base)),
            "handkerchief" => Box::new(Handkerchief::new(base)),
            "hyperbolic" => Box::new(Hyperbolic::new(base)),
            "sinusoidal" => Box::new(Sinusoidal::new(base)),
            _ => return Err(format!("Unknown variation id: {}", id).into()),
        };
        transformations.push(t);
    }
    Ok(transformations)
}

pub struct GetVariationPreviewCommandHandler {
    minio: Arc<MinioClient>,
    config: Config,
}

impl GetVariationPreviewCommandHandler {
    pub fn new(minio: Arc<MinioClient>, config: Config) -> Self {
        Self { minio, config }
    }

    pub async fn handle(
        &self,
        command: GetVariationPreviewCommand,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let key =
            MinioKeyService::preview_key(&command.variation_id, command.symmetry, command.gamma);

        if let Ok(cached) = self.minio.get_object(&key).await {
            return Ok(cached);
        }

        let transformations = create_preview_transformations(&command.variation_id)?;

        let size = self.config.preview_size;
        let canvas = FractalImage::new(size, size);
        let aspect = size as f64 / size as f64;
        let world = Rect::new(-aspect, -1.0, 2.0 * aspect, 2.0);

        let renderer = Renderer::new(
            canvas,
            world,
            transformations,
            self.config.preview_samples,
            self.config.preview_iter,
            command.symmetry,
            command.gamma,
            self.config.max_threads,
        );

        renderer.render()?;
        renderer.apply_gamma_correction();

        let png_bytes = fractal_image_to_png(renderer.canvas.as_ref())?;

        if let Err(e) = self
            .minio
            .put_object(&key, png_bytes.clone(), "image/png")
            .await
        {
            tracing::warn!(key = %key, error = %e, "Failed to cache preview in MinIO");
        }

        Ok(png_bytes)
    }
}
