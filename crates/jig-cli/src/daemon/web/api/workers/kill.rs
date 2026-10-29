//! `POST /api/v0/workers/kill`: close a worker's window.

use std::sync::Arc;

use axum::extract::State;
use axum::response::Response;

use jig_api::KillRequest;

use super::run;
use crate::daemon::commands::Command;
use crate::daemon::web::api::ApiJson;
use crate::daemon::web::guard::Authorized;
use crate::daemon::web::{Backend, Site};

pub async fn handler<B: Backend>(
    State(site): State<Arc<Site<B>>>,
    _: Authorized,
    ApiJson(request): ApiJson<KillRequest>,
) -> Response {
    run(&site, &request.worker, Command::Kill).await
}
