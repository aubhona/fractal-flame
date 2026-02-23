use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
};
use serde::Deserialize;

use crate::app::use_cases::get_variation_preview_command::GetVariationPreviewCommand;
use crate::app::use_cases::get_variation_preview_command_handler::GetVariationPreviewCommandHandler;
use crate::di;
use crate::infra::Dependencies;

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    #[serde(default = "default_symmetry")]
    pub symmetry: usize,
    #[serde(default = "default_gamma")]
    pub gamma: f64,
}

fn default_symmetry() -> usize {
    4
}

fn default_gamma() -> f64 {
    2.2
}

pub async fn get_variation_preview(
    State(deps): State<Dependencies>,
    Path(id): Path<String>,
    Query(params): Query<PreviewQuery>,
) -> impl IntoResponse {
    let handler: GetVariationPreviewCommandHandler =
        di::get_get_variation_preview_command_handler(&deps);
    let command = GetVariationPreviewCommand {
        variation_id: id,
        symmetry: params.symmetry,
        gamma: params.gamma,
    };
    match handler.handle(command) {
        Ok(png) => (
            AppendHeaders([(header::CONTENT_TYPE, "image/png")]),
            png,
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to generate preview: {}", e),
        )
            .into_response(),
    }
}
