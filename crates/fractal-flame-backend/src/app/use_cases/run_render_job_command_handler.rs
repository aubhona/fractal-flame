use std::sync::Arc;

use fractal_flame_core::app::image_export::fractal_image_to_png;
use fractal_flame_core::app::renderer::Renderer;
use fractal_flame_core::domain::{FractalImage, Rect};

use crate::infra::config::Config;
use crate::infra::dependency::filter_transformations_by_ids;
use crate::infra::minio::MinioClient;
use crate::app::services::minio_key_service::MinioKeyService;
use crate::infra::redis::RedisPool;

use super::run_render_job_command::RunRenderJobCommand;

#[derive(Clone)]
pub struct RunRenderJobCommandHandler {
    pub transformations: Arc<Vec<Box<dyn fractal_flame_core::domain::transformation::Transformation + Send + Sync>>>,
    pub config: Config,
    pub redis: Option<Arc<RedisPool>>,
    pub minio: Arc<MinioClient>,
}

impl RunRenderJobCommandHandler {
    pub fn new(
        transformations: Arc<Vec<Box<dyn fractal_flame_core::domain::transformation::Transformation + Send + Sync>>>,
        config: Config,
        redis: Option<Arc<RedisPool>>,
        minio: Arc<MinioClient>,
    ) -> Self {
        Self {
            transformations,
            config,
            redis,
            minio,
        }
    }

    pub async fn handle(&self, command: RunRenderJobCommand) {
        let RunRenderJobCommand {
            job_id,
            variation_ids,
            symmetry,
            gamma,
            width,
            height,
        } = command;

        let transformations = match filter_transformations_by_ids(&self.transformations, &variation_ids)
        {
            Ok(t) => t,
            Err(e) => {
                tracing::error!(job_id = %job_id, error = %e, "Failed to filter transformations");
                if let Some(ref r) = self.redis {
                    let _ = r
                        .set(&format!("job:{}:status", job_id), "failed", Some(3600))
                        .await;
                }
                return;
            }
        };

        if let Some(ref r) = self.redis {
            let key = format!("job:{}:status", job_id);
            let _ = r.set(&key, "pending", Some(3600)).await;
        }

        let config = self.config.clone();
        let redis = self.redis.clone();
        let minio = self.minio.clone();

        let result = tokio::task::spawn_blocking(move || {
            let canvas = FractalImage::new(width, height);
            let aspect = width as f64 / height as f64;
            let world = Rect::new(-aspect, -1.0, 2.0 * aspect, 2.0);

            let renderer = Renderer::new(
                canvas,
                world,
                transformations,
                config.samples,
                config.iter_per_sample,
                symmetry,
                gamma,
                config.max_threads,
            );

            renderer.render().map_err(|e| e.to_string())?;
            renderer.apply_gamma_correction();

            let png_bytes =
                fractal_image_to_png(renderer.canvas.as_ref()).map_err(|e| e.to_string())?;
            Ok::<Vec<u8>, String>(png_bytes)
        })
        .await;

        match result {
            Ok(Ok(png_bytes)) => {
                let key = MinioKeyService::render_result_key(&job_id);
                if let Err(e) = minio.put_object(&key, png_bytes, "image/png").await {
                    tracing::error!(job_id = %job_id, error = %e, "Failed to upload result to MinIO");
                    if let Some(ref r) = redis {
                        let _ = r
                            .set(&format!("job:{}:status", job_id), "failed", Some(3600))
                            .await;
                    }
                    return;
                }
                tracing::info!(job_id = %job_id, "Render job completed, result uploaded to MinIO");
                if let Some(ref r) = redis {
                    let _ = r
                        .set(&format!("job:{}:status", job_id), "completed", Some(3600))
                        .await;
                }
            }
            Ok(Err(e)) => {
                tracing::error!(job_id = %job_id, error = %e, "Render job failed");
                if let Some(ref r) = redis {
                    let _ = r
                        .set(&format!("job:{}:status", job_id), "failed", Some(3600))
                        .await;
                }
            }
            Err(e) => {
                tracing::error!(job_id = %job_id, error = %e, "Render job task panicked");
                if let Some(ref r) = redis {
                    let _ = r
                        .set(&format!("job:{}:status", job_id), "failed", Some(3600))
                        .await;
                }
            }
        }
    }
}
