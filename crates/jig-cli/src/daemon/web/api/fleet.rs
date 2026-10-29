//! `GET /api/v0/fleet`: this machine's frame and every peer's, as
//! [`jig_api::Fleet`].

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use jig_api as api;

use super::{refuse, status};
use crate::cli::ui;
use crate::daemon::fleet::PeerStatus;
use crate::daemon::ipc::DaemonStatus;
use crate::daemon::web::{Backend, Site};

pub async fn handler<B: Backend>(State(site): State<Arc<Site<B>>>) -> Response {
    let backend = site.backend.clone();
    let polled = tokio::task::spawn_blocking(move || (backend.status(), backend.peers())).await;
    match polled {
        Ok((local, peers)) => {
            Json(view(&local, peers, chrono::Utc::now().timestamp())).into_response()
        }
        Err(e) => refuse(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("fleet failed: {e}"),
        ),
    }
}

/// This machine first, then each peer in the order `Net` lists them. A
/// peer's frame is laid out here, by [`status::view`], exactly as its own
/// page would lay it out.
pub fn view(local: &DaemonStatus, peers: Result<Vec<PeerStatus>, String>, now: i64) -> api::Fleet {
    let this = status::view(local, now);
    let mut machines = vec![api::Machine {
        name: this
            .machine
            .clone()
            .unwrap_or_else(|| "this machine".into()),
        node: None,
        status: Some(this),
        error: None,
        last_seen: None,
    }];
    let peers_error = match peers {
        Ok(peers) => {
            machines.extend(peers.into_iter().map(|peer| {
                let last_seen = peer
                    .error
                    .as_ref()
                    .and(peer.seen_at)
                    .map(|at| ui::format_duration_short((now - at).max(0) as u64));
                api::Machine {
                    name: peer.node.name,
                    node: Some(peer.node.id.to_string()),
                    status: peer.frame.as_ref().map(|frame| status::view(frame, now)),
                    error: peer.error,
                    last_seen,
                }
            }));
            None
        }
        Err(e) => Some(e),
    };
    api::Fleet {
        machines,
        peers_error,
    }
}
