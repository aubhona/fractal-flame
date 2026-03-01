use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use fractal_flame_core::app::image_export::{
    fractal_image_to_intermediate_png, fractal_image_to_png,
};
use fractal_flame_core::app::renderer::Renderer;
use fractal_flame_core::domain::{FractalImage, Rect};
use uuid::Uuid;

use crate::app::services::minio_key_service::MinioKeyService;
use crate::app::services::redis_key_service::RedisKeyService;
use crate::infra::config::Config;
use crate::infra::dependency::generate_transformations_for_ids;
use crate::infra::minio::MinioClient;
use crate::infra::redis::RedisPool;

use super::run_render_job_command::RunRenderJobCommand;

#[derive(Clone)]
pub struct RunRenderJobCommandHandler {
    pub config: Config,
    pub redis: Option<Arc<RedisPool>>,
    pub minio: Arc<MinioClient>,
}

impl RunRenderJobCommandHandler {
    pub fn new(config: Config, redis: Option<Arc<RedisPool>>, minio: Arc<MinioClient>) -> Self {
        Self {
            config,
            redis,
            minio,
        }
    }

    pub fn start(&self, command: RunRenderJobCommand) -> String {
        let job_id = Uuid::new_v4().to_string();
        let handler = self.clone();
        let job_id_clone = job_id.clone();
        tokio::spawn(async move {
            handler.handle_inner(job_id_clone, command).await;
        });
        job_id
    }

    async fn set_redis(&self, key: &str, value: &str) {
        if let Some(ref r) = self.redis {
            let _ = r.set(key, value, Some(self.config.job_ttl_secs)).await;
        }
    }

    async fn handle_inner(&self, job_id: String, command: RunRenderJobCommand) {
        let RunRenderJobCommand {
            variation_ids,
            symmetry,
            gamma,
            width,
            height,
        } = command;

        let transformations = match generate_transformations_for_ids(&self.config, &variation_ids) {
            Ok(t) => t,
            Err(e) => {
                tracing::error!(job_id = %job_id, error = %e, "Failed to generate transformations");
                self.set_redis(&RedisKeyService::job_status(&job_id), "failed")
                    .await;
                return;
            }
        };

        let total_samples = self.config.samples;
        self.set_redis(&RedisKeyService::job_status(&job_id), "rendering")
            .await;
        self.set_redis(
            &RedisKeyService::job_total(&job_id),
            &total_samples.to_string(),
        )
        .await;
        self.set_redis(&RedisKeyService::job_progress(&job_id), "0")
            .await;
        self.set_redis(&RedisKeyService::job_intermediate_version(&job_id), "0")
            .await;

        let canvas = FractalImage::new(width, height);
        let aspect = width as f64 / height as f64;
        let world = Rect::new(-aspect, -1.0, 2.0 * aspect, 2.0);

        let mut renderer = Renderer::new(
            canvas,
            world,
            transformations,
            self.config.samples,
            self.config.iter_per_sample,
            symmetry,
            gamma,
            self.config.max_threads,
        );

        let progress = Arc::new(AtomicUsize::new(0));
        renderer.progress = Some(progress.clone());

        let canvas_shared = renderer.canvas.clone();
        let render_done = Arc::new(AtomicBool::new(false));

        let progress_sync_interval = Duration::from_millis(self.config.progress_sync_interval_ms);
        let intermediate_image_interval =
            Duration::from_millis(self.config.intermediate_image_interval_ms);
        let job_ttl = self.config.job_ttl_secs;

        let progress_sync_handle = {
            let progress = progress.clone();
            let render_done = render_done.clone();
            let redis = self.redis.clone();
            let job_id = job_id.clone();

            tokio::spawn(async move {
                while !render_done.load(Ordering::Relaxed) {
                    tokio::time::sleep(progress_sync_interval).await;
                    let current = progress.load(Ordering::Relaxed);
                    if let Some(ref r) = redis {
                        let _ = r
                            .set(
                                &RedisKeyService::job_progress(&job_id),
                                &current.to_string(),
                                Some(job_ttl),
                            )
                            .await;
                    }
                }
            })
        };

        let image_monitor_handle = {
            let progress = progress.clone();
            let render_done = render_done.clone();
            let canvas_for_monitor = canvas_shared.clone();
            let redis = self.redis.clone();
            let minio = self.minio.clone();
            let job_id = job_id.clone();
            let gamma_for_monitor = gamma;

            tokio::spawn(async move {
                let mut intermediate_version: u64 = 0;
                let mut last_progress: usize = 0;

                while !render_done.load(Ordering::Relaxed) {
                    tokio::time::sleep(intermediate_image_interval).await;

                    if render_done.load(Ordering::Relaxed) {
                        break;
                    }

                    let current = progress.load(Ordering::Relaxed);
                    if current == last_progress {
                        continue;
                    }
                    last_progress = current;

                    let canvas_snap = canvas_for_monitor.clone();
                    let snap_gamma = gamma_for_monitor;
                    let png_result = tokio::task::spawn_blocking(move || {
                        fractal_image_to_intermediate_png(&canvas_snap, snap_gamma)
                    })
                    .await;

                    if let Ok(Ok(png_bytes)) = png_result {
                        let key = MinioKeyService::intermediate_key(&job_id);
                        if minio.put_object(&key, png_bytes, "image/png").await.is_ok() {
                            intermediate_version += 1;
                            if let Some(ref r) = redis {
                                let _ = r
                                    .set(
                                        &RedisKeyService::job_intermediate_version(&job_id),
                                        &intermediate_version.to_string(),
                                        Some(job_ttl),
                                    )
                                    .await;
                            }
                        }
                    }
                }
            })
        };

        let minio = self.minio.clone();
        let redis = self.redis.clone();

        let result = tokio::task::spawn_blocking(move || {
            renderer.render().map_err(|e| e.to_string())?;
            renderer.apply_gamma_correction();
            fractal_image_to_png(renderer.canvas.as_ref()).map_err(|e| e.to_string())
        })
        .await;

        render_done.store(true, Ordering::Relaxed);
        let _ = progress_sync_handle.await;
        let _ = image_monitor_handle.await;

        match result {
            Ok(Ok(png_bytes)) => {
                let key = MinioKeyService::render_result_key(&job_id);
                if let Err(e) = minio.put_object(&key, png_bytes, "image/png").await {
                    tracing::error!(job_id = %job_id, error = %e, "Failed to upload result to MinIO");
                    if let Some(ref r) = redis {
                        let _ = r
                            .set(
                                &RedisKeyService::job_status(&job_id),
                                "failed",
                                Some(job_ttl),
                            )
                            .await;
                    }
                    return;
                }
                tracing::info!(job_id = %job_id, "Render job completed, result uploaded to MinIO");
                if let Some(ref r) = redis {
                    let _ = r
                        .set(
                            &RedisKeyService::job_progress(&job_id),
                            &total_samples.to_string(),
                            Some(job_ttl),
                        )
                        .await;
                    let _ = r
                        .set(
                            &RedisKeyService::job_status(&job_id),
                            "completed",
                            Some(job_ttl),
                        )
                        .await;
                }
            }
            Ok(Err(e)) => {
                tracing::error!(job_id = %job_id, error = %e, "Render job failed");
                if let Some(ref r) = redis {
                    let _ = r
                        .set(
                            &RedisKeyService::job_status(&job_id),
                            "failed",
                            Some(job_ttl),
                        )
                        .await;
                }
            }
            Err(e) => {
                tracing::error!(job_id = %job_id, error = %e, "Render job task panicked");
                if let Some(ref r) = redis {
                    let _ = r
                        .set(
                            &RedisKeyService::job_status(&job_id),
                            "failed",
                            Some(job_ttl),
                        )
                        .await;
                }
            }
        }
    }
}
