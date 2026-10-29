//! `/api/v0`: the server side of [`jig_api`], laid out like it: a module per
//! subject, a file per request.
//!
//! Each handler takes its request type from `jig_api` (the same struct the
//! web app sends) and is mounted on that request's `PATH`, so a route can't
//! move on one side only.

pub mod fleet;
pub mod issues;
pub mod session;
pub mod status;
pub mod workers;

use std::sync::Arc;

use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::{Json, Router};
use serde::de::DeserializeOwned;

use jig_api::{CommandOutcome, ErrorBody};
use tokio::task::JoinError;

use super::{Backend, Site};
use crate::daemon::ipc::Response as IpcResponse;

pub fn router<B: Backend>() -> Router<Arc<Site<B>>> {
    use jig_api::{fleet as f, issues as i, session as s, status as st, workers as w};
    Router::new()
        .route(s::get::PATH, get(session::handler::<B>))
        .route(st::get::PATH, get(status::handler::<B>))
        .route(f::get::PATH, get(fleet::handler::<B>))
        .route(i::get::PATH, get(issues::get::handler::<B>))
        .route(i::refresh::PATH, post(issues::refresh::handler::<B>))
        .route(i::spawn::PATH, post(issues::spawn::handler::<B>))
        .route(w::kill::PATH, post(workers::kill::handler::<B>))
        .route(w::nudge::PATH, post(workers::nudge::handler::<B>))
        .route(w::prune::PATH, post(workers::prune::handler::<B>))
        .route(
            w::retry_setup::PATH,
            post(workers::retry_setup::handler::<B>),
        )
        .route(w::model::PATH, post(workers::model::handler::<B>))
        .route(w::setup_log::PATH, get(workers::setup_log::handler::<B>))
        // An unknown API path is an API error, not the app's index page.
        .route(
            "/api/{*rest}",
            any(|| async { refuse(StatusCode::NOT_FOUND, "no such API route") }),
        )
}

/// A refusal: its status, with an [`ErrorBody`].
pub fn refuse(status: StatusCode, error: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorBody {
            error: error.into(),
        }),
    )
        .into_response()
}

/// What a daemon's answer to a command (`what`, e.g. "kill jig/feat")
/// becomes: a [`CommandOutcome`], or a refusal carrying the daemon's reason
/// (409) or what went wrong getting one (500).
pub fn outcome(what: &str, answer: Result<IpcResponse, JoinError>) -> Response {
    match answer {
        Ok(IpcResponse::Ok) => Json(CommandOutcome {
            message: format!("{what}: done"),
        })
        .into_response(),
        Ok(IpcResponse::Error(reason)) => refuse(StatusCode::CONFLICT, format!("{what}: {reason}")),
        Ok(other) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("unexpected answer: {other:?}"),
        ),
        Err(e) => refuse(StatusCode::INTERNAL_SERVER_ERROR, format!("{what}: {e}")),
    }
}

/// A JSON request body, refused with an [`ErrorBody`] (not axum's plain-text
/// rejection) when it doesn't parse, so the client always gets a reason.
pub struct ApiJson<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned> FromRequest<S> for ApiJson<T> {
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(request, state).await {
            Ok(Json(body)) => Ok(Self(body)),
            Err(e) => Err(refuse(
                StatusCode::BAD_REQUEST,
                format!("bad request: {}", e.body_text()),
            )),
        }
    }
}
