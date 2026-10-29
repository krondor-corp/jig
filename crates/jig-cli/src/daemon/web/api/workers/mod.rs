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
use axum::response::Response;

use jig_core::net::NodeId;

use super::{outcome, refuse};
use crate::daemon::commands::{Command, WorkerKey};
use crate::daemon::web::{Backend, Site};

/// Run `cmd` on `worker` through the daemon's executor, on tokio's blocking
/// pool: a verb can take as long as `on_create` does. `machine` is the
/// owning peer's node id, or `None` for this machine.
pub async fn run<B: Backend>(
    site: &Site<B>,
    machine: Option<&str>,
    worker: &str,
    cmd: Command,
) -> Response {
    let key: WorkerKey = match worker.parse() {
        Ok(key) => key,
        Err(e) => return refuse(StatusCode::BAD_REQUEST, e),
    };
    let verb = cmd.name();
    let machine = machine.map(NodeId::new);
    tracing::info!(worker = %key, command = verb, machine = ?machine, "command over web");

    let backend = site.backend.clone();
    let target = key.clone();
    let answer =
        tokio::task::spawn_blocking(move || backend.run_command(machine, target, cmd)).await;
    outcome(&format!("{verb} {key}"), answer)
}
