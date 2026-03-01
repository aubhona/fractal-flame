use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::app::use_cases::render_progress_command::RenderProgressCommand;
use crate::di;
use crate::infra::Dependencies;

#[derive(Serialize)]
struct ProgressPayload {
    status: String,
    progress: u64,
    total: u64,
    intermediate_version: u64,
}

pub async fn render_progress(
    State(deps): State<Dependencies>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    let Some(handler) = di::get_render_progress_command_handler(&deps) else {
        return (StatusCode::SERVICE_UNAVAILABLE, "Redis not configured")
            .into_response();
    };

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(16);
    let command = RenderProgressCommand { job_id };

    tokio::spawn(async move {
        loop {
            let info = handler.get_progress(&command).await;

            let payload = ProgressPayload {
                status: info.status.clone(),
                progress: info.progress,
                total: info.total,
                intermediate_version: info.intermediate_version,
            };
            let data = serde_json::to_string(&payload).unwrap_or_default();

            match info.status.as_str() {
                "completed" => {
                    let _ = tx.send(Ok(Event::default().event("completed").data(data))).await;
                    break;
                }
                "failed" => {
                    let _ = tx.send(Ok(Event::default().event("failed").data(data))).await;
                    break;
                }
                _ => {
                    if tx.send(Ok(Event::default().event("progress").data(data))).await.is_err() {
                        break;
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let stream: ReceiverStream<Result<Event, Infallible>> = ReceiverStream::new(rx);
    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response()
}
