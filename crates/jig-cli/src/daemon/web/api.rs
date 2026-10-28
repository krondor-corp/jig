//! `/api/v0`: the JSON the web app reads, and the commands it sends.

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response as HttpResponse};
use axum::routing::{any, get, post};
use axum::{Json, Router};
use comfy_table::Color;
use serde::Deserialize;
use url::Url;

use jig_api as api;
use jig_core::git::Branch;

use super::guard::Authorized;
use super::{Backend, Site};
use crate::cli::commands::ps::render::{format_state, worker_columns, WORKER_HEADERS};
use crate::cli::ui;
use crate::daemon::commands::{Command, WorkerKey};
use crate::daemon::ipc::{DaemonStatus, Response};
use crate::worker::events::{SetupState, WorkerState};

/// The most of a `setup.log` one request returns: its end.
const MAX_LOG_BYTES: u64 = 1024 * 1024;

pub fn router<B: Backend>() -> Router<Arc<Site<B>>> {
    Router::new()
        .route(api::SESSION, get(session::<B>))
        .route(api::STATUS, get(status::<B>))
        .route(api::COMMAND, post(command::<B>))
        .route(api::SETUP_LOG, get(setup_log::<B>))
        // An unknown API path is an API error, not the app's index page.
        .route(
            "/api/{*rest}",
            any(|| async { refuse(StatusCode::NOT_FOUND, "no such API route") }),
        )
}

/// A refusal: its status, with an [`api::ApiError`] body.
pub fn refuse(status: StatusCode, error: impl Into<String>) -> HttpResponse {
    (
        status,
        Json(api::ApiError {
            error: error.into(),
        }),
    )
        .into_response()
}

async fn session<B: Backend>(State(site): State<Arc<Site<B>>>) -> Json<api::Session> {
    Json(api::Session {
        token: site.token.clone(),
    })
}

async fn status<B: Backend>(State(site): State<Arc<Site<B>>>) -> HttpResponse {
    let backend = site.backend.clone();
    match tokio::task::spawn_blocking(move || backend.status()).await {
        Ok(status) => Json(view(&status, chrono::Utc::now().timestamp())).into_response(),
        Err(e) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("status failed: {e}"),
        ),
    }
}

async fn command<B: Backend>(
    State(site): State<Arc<Site<B>>>,
    _: Authorized,
    body: Result<Json<api::CommandRequest>, JsonRejection>,
) -> HttpResponse {
    let Json(request) = match body {
        Ok(body) => body,
        Err(e) => {
            return refuse(
                StatusCode::BAD_REQUEST,
                format!("bad command: {}", e.body_text()),
            )
        }
    };
    let worker: WorkerKey = match request.worker.parse() {
        Ok(key) => key,
        Err(e) => return refuse(StatusCode::BAD_REQUEST, e),
    };
    let verb = request.command.name();
    let cmd = into_command(request.command);
    tracing::info!(worker = %worker, command = verb, "command over web");

    let backend = site.backend.clone();
    let key = worker.clone();
    let answer = tokio::task::spawn_blocking(move || backend.run_command(key, cmd)).await;
    match answer {
        Ok(Response::Ok) => Json(api::CommandOutcome {
            message: format!("{verb} {worker}: done"),
        })
        .into_response(),
        Ok(Response::Error(reason)) => {
            refuse(StatusCode::CONFLICT, format!("{verb} {worker}: {reason}"))
        }
        Ok(other) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("unexpected answer: {other:?}"),
        ),
        Err(e) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{verb} {worker}: {e}"),
        ),
    }
}

fn into_command(command: api::Command) -> Command {
    match command {
        api::Command::Kill => Command::Kill,
        api::Command::Nudge { text } => Command::Nudge { text },
        api::Command::Prune => Command::Prune,
        api::Command::RetrySetup => Command::RetrySetup,
        api::Command::Model { model } => Command::Model { model },
    }
}

#[derive(Deserialize)]
struct LogQuery {
    worker: Option<String>,
}

/// The `setup.log` of the worker named by `?worker=`. Only a path the daemon
/// itself recorded is ever read: the request names a worker, not a file.
async fn setup_log<B: Backend>(
    State(site): State<Arc<Site<B>>>,
    Query(query): Query<LogQuery>,
) -> HttpResponse {
    let Some(key) = query.worker.and_then(|k| k.parse::<WorkerKey>().ok()) else {
        return (StatusCode::BAD_REQUEST, "expected ?worker=repo/branch").into_response();
    };
    let backend = site.backend.clone();
    let read = tokio::task::spawn_blocking(move || {
        let status = backend.status();
        let log_path = status
            .workers
            .iter()
            .find(|w| {
                let w = w.to_display_state();
                w.repo_name() == key.repo && key.branch == w.name.as_str()
            })
            .and_then(|w| match &w.setup {
                SetupState::Failed { log_path, .. } | SetupState::Running { log_path, .. } => {
                    log_path.clone()
                }
                SetupState::Pending | SetupState::Ok { .. } => None,
            });
        let Some(path) = log_path else {
            return Err(format!("no setup.log for {key}"));
        };
        read_tail(&path, MAX_LOG_BYTES).map_err(|e| format!("cannot read setup.log for {key}: {e}"))
    })
    .await;
    match read {
        Ok(Ok(bytes)) => {
            ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], bytes).into_response()
        }
        Ok(Err(e)) => (StatusCode::NOT_FOUND, e).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// The last `max` bytes of `path`.
pub fn read_tail(path: &Path, max: u64) -> std::io::Result<Vec<u8>> {
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    if len > max {
        file.seek(SeekFrom::Start(len - max))?;
    }
    let mut out = Vec::new();
    file.take(max).read_to_end(&mut out)?;
    Ok(out)
}

/// The daemon's frame as the app's [`api::Status`]. Every cell is a `jig ps`
/// column, so the page and the terminal can't drift.
pub fn view(status: &DaemonStatus, now: i64) -> api::Status {
    let workers: Vec<WorkerState> = status
        .workers
        .iter()
        .map(|w| w.to_display_state())
        .collect();

    // `ps -g`'s grouping, in the order repos are first seen: the daemon
    // always watches every repo.
    let mut repos: Vec<api::Repo> = Vec::new();
    for w in &workers {
        let row = worker(w, now);
        let name = w.repo_name();
        match repos.iter_mut().find(|r| r.name == name) {
            Some(repo) => repo.workers.push(row),
            None => repos.push(api::Repo {
                name,
                workers: vec![row],
            }),
        }
    }

    api::Status {
        machine: status.info.machine.as_ref().map(|m| m.name.clone()),
        pid: status.info.pid,
        version: status.info.version.clone(),
        next_poll: ui::format_duration_short(status.poll_remaining),
        headers: WORKER_HEADERS.iter().map(|h| h.to_string()).collect(),
        repos,
        triages: status
            .triages
            .iter()
            .map(|t| api::Triage {
                issue: t.issue_id.clone(),
                model: t.model.clone(),
                elapsed: ui::format_duration_short((now - t.spawned_at).max(0) as u64),
                repo: t.repo_name.clone(),
            })
            .collect(),
        spawning: status.spawning.iter().map(Branch::to_string).collect(),
    }
}

fn worker(w: &WorkerState, now: i64) -> api::Worker {
    let failed_setup = match &w.setup {
        SetupState::Failed { tail, log_path, .. } => Some(api::FailedSetup {
            state: format_state(w, now).0,
            tail: tail.clone(),
            has_log: log_path.is_some(),
        }),
        _ => None,
    };
    api::Worker {
        key: format!("{}/{}", w.repo_name(), w.name),
        name: w.name.clone(),
        agent: w.mux_agent_state.map(|a| a.to_string()),
        pr_url: w
            .pr_url
            .as_ref()
            .filter(|u| is_web_url(u))
            .map(Url::to_string),
        cells: worker_columns(w, now)
            .into_iter()
            .map(|c| api::Cell {
                text: c.text,
                tone: tone(c.color),
                right: c.right,
            })
            .collect(),
        failed_setup,
    }
}

/// The app's [`api::Tone`] for a `ps` color.
fn tone(color: Color) -> api::Tone {
    match color {
        Color::Green | Color::DarkGreen => api::Tone::Green,
        Color::Red | Color::DarkRed => api::Tone::Red,
        Color::Yellow | Color::DarkYellow => api::Tone::Yellow,
        Color::Blue | Color::DarkBlue => api::Tone::Blue,
        Color::Cyan | Color::DarkCyan => api::Tone::Cyan,
        Color::Magenta | Color::DarkMagenta => api::Tone::Magenta,
        Color::DarkGrey | Color::Grey => api::Tone::Dim,
        _ => api::Tone::Plain,
    }
}

/// Only link what a browser should follow: never a `javascript:` URL.
fn is_web_url(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
}
