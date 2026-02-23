use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    State(_deps): State<Dependencies>,
    Json(body): Json<StartRenderRequest>,
) -> impl IntoResponse {
    let job_id = Uuid::new_v4().to_string();
    let job_id_for_spawn = job_id.clone();

    tokio::spawn(async move {
        tracing::info!(
            job_id = %job_id_for_spawn,
            variation_ids = ?body.variation_ids,
            symmetry = body.symmetry,
            gamma = body.gamma,
            width = body.width,
            height = body.height,
            "Render job started (placeholder)"
        );
        // TODO: Redis state, MinIO, actual rendering
    });

    (StatusCode::ACCEPTED, Json(StartRenderResponse { job_id }))
}
