//! `GET /api/v0/session`: hands the page its command token.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;

use jig_api::Session;

use crate::daemon::web::{Backend, Site};

pub async fn handler<B: Backend>(State(site): State<Arc<Site<B>>>) -> Json<Session> {
    Json(Session {
        token: site.token.clone(),
    })
}
