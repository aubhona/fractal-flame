use std::sync::Arc;

use crate::app::services::minio_key_service::MinioKeyService;
use crate::infra::minio::MinioClient;

use super::get_intermediate_result_command::GetIntermediateResultCommand;

#[derive(Debug)]
pub enum GetIntermediateResultOutcome {
    Ready(Vec<u8>),
    NotFound,
}

pub struct GetIntermediateResultCommandHandler {
    minio: Arc<MinioClient>,
}

impl GetIntermediateResultCommandHandler {
    pub fn new(minio: Arc<MinioClient>) -> Self {
        Self { minio }
    }

    pub async fn handle(
        &self,
        command: GetIntermediateResultCommand,
    ) -> GetIntermediateResultOutcome {
        let key = MinioKeyService::intermediate_key(&command.job_id);
        match self.minio.get_object(&key).await {
            Ok(png_bytes) => GetIntermediateResultOutcome::Ready(png_bytes),
            Err(_) => GetIntermediateResultOutcome::NotFound,
        }
    }
}
