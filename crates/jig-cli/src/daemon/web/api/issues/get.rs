//! `GET /api/v0/issues?machine=<node>`: a machine's issues, as its daemon
//! last fetched them.

use std::sync::Arc;

use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use jig_api as api;
use jig_core::net::NodeId;

use crate::cli::ui;
use crate::daemon::actors::issues::IssueBoard;
use crate::daemon::web::api::refuse;
use crate::daemon::web::{Backend, Site};

pub async fn handler<B: Backend>(
    State(site): State<Arc<Site<B>>>,
    query: Result<Query<api::IssuesRequest>, QueryRejection>,
) -> Response {
    let Ok(Query(request)) = query else {
        return refuse(
            StatusCode::BAD_REQUEST,
            "expected ?machine=<node> or nothing",
        );
    };
    let machine = request.machine.map(NodeId::new);
    let backend = site.backend.clone();
    match tokio::task::spawn_blocking(move || backend.issues(machine)).await {
        Ok(Ok(board)) => Json(view(board, chrono::Utc::now().timestamp())).into_response(),
        Ok(Err(e)) => refuse(StatusCode::BAD_GATEWAY, e),
        Err(e) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("issues failed: {e}"),
        ),
    }
}

/// A daemon's [`IssueBoard`] as the app's [`api::Issues`].
pub fn view(board: IssueBoard, now: i64) -> api::Issues {
    api::Issues {
        repos: board
            .repos
            .into_iter()
            .map(|repo| api::RepoIssues {
                repo: repo.repo,
                issues: repo
                    .issues
                    .into_iter()
                    .map(|i| api::Issue {
                        id: i.id,
                        title: i.title,
                        status: i.status.to_string().to_lowercase(),
                        priority: i.priority.to_string().to_lowercase(),
                        labels: i.labels,
                        branch: i.branch.to_string(),
                        auto: i.auto,
                        parent: i.parent,
                        spawned: i.spawned,
                    })
                    .collect(),
                error: repo.error,
            })
            .collect(),
        fetched: board
            .fetched_at
            .map(|at| ui::format_duration_short((now - at).max(0) as u64)),
    }
}
