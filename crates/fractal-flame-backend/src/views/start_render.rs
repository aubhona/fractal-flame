use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::app::use_cases::run_render_job_command::RunRenderJobCommand;
use crate::di;
use crate::infra::Dependencies;

#[derive(Debug, Deserialize)]
pub struct StartRenderRequest {
    pub variation_ids: Vec<String>,
    pub symmetry: usize,
    pub gamma: f64,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Serialize)]
pub struct StartRenderResponse {
    pub job_id: String,
}

pub async fn start_render(
    State(deps): State<Dependencies>,
    Json(body): Json<StartRenderRequest>,
) -> impl IntoResponse {
    let Some(handler) = di::get_run_render_job_command_handler(&deps) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "MinIO not configured".to_string(),
        )
            .into_response();
    };

    if body.variation_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "Select at least one variation".to_string(),
        )
            .into_response();
    }

    let command = RunRenderJobCommand {
        variation_ids: body.variation_ids.clone(),
        symmetry: body.symmetry,
        gamma: body.gamma,
        width: body.width,
        height: body.height,
    };

    let job_id = handler.start(command);

    (StatusCode::ACCEPTED, Json(StartRenderResponse { job_id })).into_response()
}
