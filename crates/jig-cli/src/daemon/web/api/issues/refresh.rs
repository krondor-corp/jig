//! `POST /api/v0/issues/refresh`: have a machine fetch its issues again.

use std::sync::Arc;

use axum::extract::State;
use axum::response::Response;

use jig_api::RefreshIssuesRequest;
use jig_core::net::NodeId;

use crate::daemon::web::api::{outcome, ApiJson};
use crate::daemon::web::guard::Authorized;
use crate::daemon::web::{Backend, Site};

pub async fn handler<B: Backend>(
    State(site): State<Arc<Site<B>>>,
    _: Authorized,
    ApiJson(request): ApiJson<RefreshIssuesRequest>,
) -> Response {
    let machine = request.machine.map(NodeId::new);
    let backend = site.backend.clone();
    let answer = tokio::task::spawn_blocking(move || backend.refresh_issues(machine)).await;
    outcome("refresh issues", answer)
}
