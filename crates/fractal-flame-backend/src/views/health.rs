use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::infra::Dependencies;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct ReadinessResponse {
    status: &'static str,
    redis: ComponentStatus,
    minio: ComponentStatus,
}

#[derive(Serialize)]
struct ComponentStatus {
    available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

pub async fn readiness(State(deps): State<Dependencies>) -> impl IntoResponse {
    let redis_status = match &deps.redis {
        Some(r) => match r.ping().await {
            Ok(()) => ComponentStatus {
                available: true,
                error: None,
            },
            Err(e) => ComponentStatus {
                available: false,
                error: Some(e.to_string()),
            },
        },
        None => ComponentStatus {
            available: false,
            error: Some("not configured".to_string()),
        },
    };

    let minio_status = match &deps.minio {
        Some(m) => match m.ping().await {
            Ok(()) => ComponentStatus {
                available: true,
                error: None,
            },
            Err(e) => ComponentStatus {
                available: false,
                error: Some(e.to_string()),
            },
        },
        None => ComponentStatus {
            available: false,
            error: Some("not configured".to_string()),
        },
    };

    let all_ok = redis_status.available && minio_status.available;
    let status_code = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(ReadinessResponse {
            status: if all_ok { "ready" } else { "not_ready" },
            redis: redis_status,
            minio: minio_status,
        }),
    )
}
