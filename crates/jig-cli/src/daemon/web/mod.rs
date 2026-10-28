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
//! The daemon is std threads. The web view is the only async code in it: one
//! `jig-web` thread owns a current-thread tokio runtime and serves axum on
//! it. Daemon calls block, so handlers make them on tokio's blocking pool and
//! never block the runtime. Nothing else in the daemon knows tokio exists.
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use uuid::Uuid;

use super::commands::{Command, WorkerKey};
use super::ipc::{DaemonStatus, Response};
use super::DaemonShared;

/// How often the server checks whether the daemon is shutting down.
const SHUTDOWN_CHECK: Duration = Duration::from_millis(500);

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

    /// Serve until `quit` is set, on a `jig-web` thread with its own
    /// runtime.
    pub fn spawn<B: Backend>(
        self,
        backend: B,
        quit: Arc<AtomicBool>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new()
            .name("jig-web".into())
            .spawn(move || self.serve(backend, quit))
            .expect("failed to spawn jig-web thread")
    }

    fn serve<B: Backend>(self, backend: B, quit: Arc<AtomicBool>) {
        let url = self.url();
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!("web view not started: no runtime: {}", e);
                return;
            }
        };
        let app = router(Site {
            backend,
            port: self.port,
            token: self.token,
        });
        let result = runtime.block_on(async {
            let listener = tokio::net::TcpListener::from_std(self.listener)?;
            tracing::info!(url = %url, "web view listening");
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    while !quit.load(Ordering::Relaxed) {
                        tokio::time::sleep(SHUTDOWN_CHECK).await;
                    }
                })
                .await
        });
        match result {
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
