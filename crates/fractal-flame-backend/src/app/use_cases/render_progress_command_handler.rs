use std::sync::Arc;

use crate::app::services::redis_key_service::RedisKeyService;
use crate::infra::redis::RedisPool;

use super::render_progress_command::RenderProgressCommand;

#[derive(Clone, Debug)]
pub struct JobProgress {
    pub status: String,
    pub progress: u64,
    pub total: u64,
    pub intermediate_version: u64,
}

pub struct RenderProgressCommandHandler {
    redis: Arc<RedisPool>,
}

impl RenderProgressCommandHandler {
    pub fn new(redis: Arc<RedisPool>) -> Self {
        Self { redis }
    }

    pub async fn get_progress(&self, command: &RenderProgressCommand) -> JobProgress {
        let job_id = &command.job_id;

        let status = self
            .redis
            .get(&RedisKeyService::job_status(job_id))
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "pending".to_string());

        let progress: u64 = self
            .redis
            .get(&RedisKeyService::job_progress(job_id))
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let total: u64 = self
            .redis
            .get(&RedisKeyService::job_total(job_id))
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let intermediate_version: u64 = self
            .redis
            .get(&RedisKeyService::job_intermediate_version(job_id))
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        JobProgress {
            status,
            progress,
            total,
            intermediate_version,
        }
    }
}
