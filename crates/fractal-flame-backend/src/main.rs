mod app;
mod di;
mod infra;
mod views;

use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::Level;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;

#[tokio::main]
async fn main() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        .add_directive(tracing::Level::INFO.into());

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    let config = infra::Config::from_file(None::<&str>)
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to load config from file ({}), using defaults", e);
            infra::Config::default()
        });
    let deps = infra::Dependencies::new(config).expect("Failed to initialize dependencies");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/variations", get(views::get_variations::get_variations))
        .route(
            "/api/variations/{id}/preview",
            get(views::get_variation_preview::get_variation_preview),
        )
        .route(
            "/api/render/start",
            post(views::start_render::start_render),
        )
        .route(
            "/api/render/{job_id}/result",
            get(views::get_render_result::get_render_result),
        )
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                )
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
        .with_state(deps);

    let port = std::env::var("APP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
