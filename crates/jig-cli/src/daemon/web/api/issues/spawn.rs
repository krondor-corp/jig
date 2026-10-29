//! `POST /api/v0/issues/spawn`: start a worker for an issue, on the
//! machine the request names.

use std::sync::Arc;

use axum::extract::State;
use axum::response::Response;

use jig_api::SpawnRequest;
use jig_core::net::NodeId;

use crate::daemon::web::api::{outcome, ApiJson};
use crate::daemon::web::guard::Authorized;
use crate::daemon::web::{Backend, Site};

pub async fn handler<B: Backend>(
    State(site): State<Arc<Site<B>>>,
    _: Authorized,
    ApiJson(request): ApiJson<SpawnRequest>,
) -> Response {
    let SpawnRequest {
        repo,
        issue,
        machine,
    } = request;
    let what = format!("spawn {issue} in {repo}");
    let machine = machine.map(NodeId::new);
    tracing::info!(%repo, %issue, machine = ?machine, "spawn over web");
    let backend = site.backend.clone();
    // As long as the worker's setup takes: on tokio's blocking pool.
    let answer = tokio::task::spawn_blocking(move || backend.spawn(machine, repo, issue)).await;
    outcome(&what, answer)
}
