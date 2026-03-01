use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse},
};

use crate::app::use_cases::get_intermediate_result_command::GetIntermediateResultCommand;
use crate::app::use_cases::get_intermediate_result_command_handler::GetIntermediateResultOutcome;
use crate::di;
use crate::infra::Dependencies;

pub async fn get_intermediate_result(
    State(deps): State<Dependencies>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    let Some(handler) = di::get_get_intermediate_result_command_handler(&deps) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "MinIO not configured".to_string(),
        )
            .into_response();
    };

    match handler
        .handle(GetIntermediateResultCommand { job_id })
        .await
    {
        GetIntermediateResultOutcome::Ready(png_bytes) => (
            AppendHeaders([(header::CONTENT_TYPE, "image/png")]),
            png_bytes,
        )
            .into_response(),
        GetIntermediateResultOutcome::NotFound => (
            StatusCode::NOT_FOUND,
            "No intermediate image yet".to_string(),
        )
            .into_response(),
    }
}
