use std::sync::Arc;

use crate::app::services::minio_key_service::MinioKeyService;
use crate::infra::minio::MinioClient;

use super::get_render_result_command::GetRenderResultCommand;

#[derive(Debug)]
pub enum GetRenderResultOutcome {
    Ready(Vec<u8>),
    Pending,
}

pub struct GetRenderResultCommandHandler {
    minio: Arc<MinioClient>,
}

impl GetRenderResultCommandHandler {
    pub fn new(minio: Arc<MinioClient>) -> Self {
        Self { minio }
    }

    pub async fn handle(&self, command: GetRenderResultCommand) -> GetRenderResultOutcome {
        let key = MinioKeyService::render_result_key(&command.job_id);

        match self.minio.get_object(&key).await {
            Ok(png_bytes) => GetRenderResultOutcome::Ready(png_bytes),
            Err(_) => GetRenderResultOutcome::Pending,
        }
    }
}
