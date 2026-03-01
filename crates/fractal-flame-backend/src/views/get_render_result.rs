use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse},
};

use crate::app::use_cases::get_render_result_command::GetRenderResultCommand;
use crate::app::use_cases::get_render_result_command_handler::GetRenderResultOutcome;
use crate::di;
use crate::infra::Dependencies;

pub async fn get_render_result(
    State(deps): State<Dependencies>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    let Some(handler) = di::get_get_render_result_command_handler(&deps) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "MinIO not configured".to_string(),
        )
            .into_response();
    };

    let outcome = handler
        .handle(GetRenderResultCommand {
            job_id: job_id.clone(),
        })
        .await;

    match outcome {
        GetRenderResultOutcome::Ready(png_bytes) => (
            AppendHeaders([(header::CONTENT_TYPE, "image/png")]),
            png_bytes,
        )
            .into_response(),
        GetRenderResultOutcome::Pending => {
            (StatusCode::ACCEPTED, "Rendering in progress".to_string()).into_response()
        }
    }
}
