//! Web view: the work view and the command buttons, served over HTTP on
//! `127.0.0.1`.
//!
//! Off unless `[fleet] web = true`. Two halves:
//!
//! * **API** (`api`) is axum routes under `/api/v0`, speaking the
//!   [`jig_api`] wire types. Handlers read the same [`DaemonStatus`] `jig ps`
//!   renders and run commands through the same executor the IPC socket does
//!   ([`DaemonShared::run_command`]). The web view is a third client of one
//!   daemon, not a second implementation of it.
//! * **App** (`spa`) is the Yew app in `crates/jig-cli/web`, built by Trunk
//!   into `web/dist/` and embedded in the binary.
//!
//! # Threads
//!
//! The daemon is std threads. The web view is the only async code in it: a
//! [`Service`] that a [`crate::runtime::Supervisor`] runs on the `jig-web`
//! thread, which owns a current-thread tokio runtime. Daemon calls block, so
//! handlers make them on tokio's blocking pool and never block the runtime.
//! Shutdown is the supervisor's: `daemon start` signals it when it quits, and
//! in-flight requests get [`DRAIN`] to finish.
//!
//! # Hardening
//!
//! A port on `127.0.0.1` is out of other machines' reach, but any web page
//! open in the browser can still send requests to it. So:
//!
//! * **`Host` check** (`guard`): a request must name `127.0.0.1:<port>` or
//!   `localhost:<port>`. A DNS-rebinding page reaches us under its own
//!   hostname, and is refused before anything else runs.
//! * **No state change on `GET`**: every `GET` only reads. Commands are
//!   `POST`s.
//! * **Session token**: a command must carry [`jig_api::TOKEN_HEADER`], a
//!   random value minted at daemon start. The app reads it from
//!   [`jig_api::SESSION`]. Another origin can't read that response (no CORS),
//!   and can't send a custom header cross-origin without a preflight, which
//!   is never granted. A present `Origin` header must be ours too.
//! * **CSP**: scripts only from us. The bundle's inline loader is allowed by
//!   its hash, not by `'unsafe-inline'`.

mod api;
mod guard;
mod spa;

use std::net::{Ipv4Addr, TcpListener};
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use tokio::sync::watch;
use uuid::Uuid;

use super::commands::{Command, WorkerKey};
use super::ipc::{DaemonStatus, Response};
use super::DaemonShared;
use crate::runtime::Service;

/// How long a stopping daemon waits for in-flight web requests (a command
/// someone clicked) before it leaves them behind.
pub const DRAIN: Duration = Duration::from_secs(10);

/// What the web view needs from the daemon: [`DaemonShared`] in the daemon,
/// a stand-in in tests. Both calls may block.
pub trait Backend: Clone + Send + Sync + 'static {
    /// The frame `jig ps` renders.
    fn status(&self) -> DaemonStatus;
    /// Run a control verb, blocking until it answers.
    fn run_command(&self, worker: WorkerKey, cmd: Command) -> Response;
}

impl Backend for DaemonShared {
    fn status(&self) -> DaemonStatus {
        DaemonShared::status(self)
    }

    fn run_command(&self, worker: WorkerKey, cmd: Command) -> Response {
        DaemonShared::run_command(self, worker, cmd)
    }
}

/// Everything a request is answered from.
struct Site<B> {
    backend: B,
    port: u16,
    token: String,
}

/// The web view's bound port.
pub struct WebServer {
    listener: TcpListener,
    port: u16,
    token: String,
}

impl WebServer {
    /// Bind `127.0.0.1:port`, never another interface. Port 0 picks one.
    ///
    /// Binds here, synchronously, so a taken port is the caller's error to
    /// report before anything is spawned.
    pub fn bind(port: u16) -> std::io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        Ok(Self {
            listener,
            port,
            token: Uuid::new_v4().simple().to_string(),
        })
    }

    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/", self.port)
    }

    /// The web view as a [`Service`]: this port, answering from `backend`.
    pub fn serve<B: Backend>(self, backend: B) -> WebView<B> {
        WebView {
            server: self,
            backend,
        }
    }
}

/// A bound web view and the daemon behind it: what [`Supervisor`] runs.
///
/// [`Supervisor`]: crate::runtime::Supervisor
pub struct WebView<B> {
    server: WebServer,
    backend: B,
}

impl<B: Backend> Service for WebView<B> {
    type State = Self;

    async fn run(view: Self, mut shutdown: watch::Receiver<()>) {
        let WebView { server, backend } = view;
        let url = server.url();
        let listener = match tokio::net::TcpListener::from_std(server.listener) {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!("web view not started: {}", e);
                return;
            }
        };
        let app = router(Site {
            backend,
            port: server.port,
            token: server.token,
        });
        tracing::info!(url = %url, "web view listening");
        let served = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown.changed().await;
            })
            .await;
        match served {
            Ok(()) => tracing::info!("web view stopped"),
            Err(e) => tracing::warn!("web view failed: {}", e),
        }
    }
}

/// The whole site: the API, then the app as the fallback, behind the guards.
fn router<B: Backend>(site: Site<B>) -> Router {
    let site = Arc::new(site);
    api::router()
        .fallback(spa::serve)
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&site),
            guard::host::<B>,
        ))
        .layer(axum::middleware::from_fn(guard::headers))
        .with_state(site)
}

#[cfg(test)]
mod tests;
