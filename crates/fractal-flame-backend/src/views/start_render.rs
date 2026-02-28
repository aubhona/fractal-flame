use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        return (StatusCode::BAD_REQUEST, "Select at least one variation".to_string()).into_response();
    }

    let job_id = Uuid::new_v4().to_string();
    let job_id_spawn = job_id.clone();
    let variation_ids = body.variation_ids.clone();
    let symmetry = body.symmetry;
    let gamma = body.gamma;
    let width = body.width;
    let height = body.height;

    tokio::spawn(async move {
        handler
            .handle(RunRenderJobCommand {
                job_id: job_id_spawn,
                variation_ids,
                symmetry,
                gamma,
                width,
                height,
            })
            .await
    });

    (StatusCode::ACCEPTED, Json(StartRenderResponse { job_id })).into_response()
}
