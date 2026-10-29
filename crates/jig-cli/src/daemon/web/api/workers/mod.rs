//! `/api/v0/workers/*`: a handler per verb, and the setup log.
//!
//! Every verb handler is the same three steps (check the token, parse its
//! request, map it to the daemon's [`Command`]), then [`run`].

pub mod kill;
pub mod model;
pub mod nudge;
pub mod prune;
pub mod retry_setup;
pub mod setup_log;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use jig_api::CommandOutcome;

use super::refuse;
use crate::daemon::commands::{Command, WorkerKey};
use crate::daemon::ipc::Response as IpcResponse;
use crate::daemon::web::{Backend, Site};

/// Run `cmd` on `worker` through the daemon's executor, on tokio's blocking
/// pool: a verb can take as long as `on_create` does.
pub async fn run<B: Backend>(site: &Site<B>, worker: &str, cmd: Command) -> Response {
    let key: WorkerKey = match worker.parse() {
        Ok(key) => key,
        Err(e) => return refuse(StatusCode::BAD_REQUEST, e),
    };
    let verb = cmd.name();
    tracing::info!(worker = %key, command = verb, "command over web");

    let backend = site.backend.clone();
    let target = key.clone();
    match tokio::task::spawn_blocking(move || backend.run_command(target, cmd)).await {
        Ok(IpcResponse::Ok) => Json(CommandOutcome {
            message: format!("{verb} {key}: done"),
        })
        .into_response(),
        Ok(IpcResponse::Error(reason)) => {
            refuse(StatusCode::CONFLICT, format!("{verb} {key}: {reason}"))
        }
        Ok(other) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("unexpected answer: {other:?}"),
        ),
        Err(e) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{verb} {key}: {e}"),
        ),
    }
}
