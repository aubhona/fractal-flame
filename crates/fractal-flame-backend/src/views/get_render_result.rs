use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
};

use crate::app::services::minio_key_service::MinioKeyService;
use crate::infra::Dependencies;

pub async fn get_render_result(
    State(deps): State<Dependencies>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    let Some(ref minio) = deps.minio else {
        return (StatusCode::SERVICE_UNAVAILABLE, "MinIO not configured".to_string()).into_response();
    };

    let key = MinioKeyService::render_result_key(&job_id);

    match minio.get_object(&key).await {
        Ok(png_bytes) => (
            AppendHeaders([(header::CONTENT_TYPE, "image/png")]),
            png_bytes,
        )
            .into_response(),
        Err(_) => (
            StatusCode::ACCEPTED,
            "Rendering in progress".to_string(),
        )
            .into_response(),
    }
}
