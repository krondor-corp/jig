//! `GET /api/v0/status`: the daemon's frame as [`jig_api::Status`].

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use comfy_table::Color;
use url::Url;

use jig_api as api;
use jig_core::git::Branch;

use super::refuse;
use crate::cli::commands::ps::render::{format_state, worker_columns, WORKER_HEADERS};
use crate::cli::ui;
use crate::daemon::ipc::DaemonStatus;
use crate::daemon::web::{Backend, Site};
use crate::worker::events::{SetupState, WorkerState};

pub async fn handler<B: Backend>(State(site): State<Arc<Site<B>>>) -> Response {
    let backend = site.backend.clone();
    match tokio::task::spawn_blocking(move || backend.status()).await {
        Ok(status) => Json(view(&status, chrono::Utc::now().timestamp())).into_response(),
        Err(e) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("status failed: {e}"),
        ),
    }
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
