//! `GET /api/v0/workers/setup-log?worker=<repo/branch>`: the end of a
//! worker's `setup.log`.

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use jig_api::{SetupLog, SetupLogRequest};

use crate::daemon::commands::WorkerKey;
use crate::daemon::web::api::refuse;
use crate::daemon::web::{Backend, Site};
use crate::worker::events::SetupState;

/// The most of a `setup.log` one request returns: its end.
const MAX_LOG_BYTES: u64 = 1024 * 1024;

/// Only a path the daemon itself recorded is ever read: the request names a
/// worker, not a file.
pub async fn handler<B: Backend>(
    State(site): State<Arc<Site<B>>>,
    query: Result<Query<SetupLogRequest>, QueryRejection>,
) -> Response {
    let Some(key) = query
        .ok()
        .and_then(|Query(q)| q.worker.parse::<WorkerKey>().ok())
    else {
        return refuse(StatusCode::BAD_REQUEST, "expected ?worker=repo/branch");
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
        Ok(Ok(bytes)) => Json(SetupLog {
            text: String::from_utf8_lossy(&bytes).into_owned(),
        })
        .into_response(),
        Ok(Err(e)) => refuse(StatusCode::NOT_FOUND, e),
        Err(e) => refuse(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
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
