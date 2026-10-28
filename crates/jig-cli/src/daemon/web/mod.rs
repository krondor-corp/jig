//! Web view — the work view and the command buttons over HTTP on
//! `127.0.0.1`.
//!
//! Off unless `[fleet] web = true`. It reads the same [`DaemonStatus`] `jig
//! ps` renders and runs commands through the same executor the IPC socket
//! does ([`DaemonShared::run_command`]), so it is a third face on one daemon,
//! not a second implementation. Std threads only: one listener thread, one
//! thread per connection.
//!
//! # Hardening
//!
//! A port on `127.0.0.1` is out of other machines' reach but not out of
//! every web page's: any site open in the browser can send requests to it.
//! So:
//!
//! * **`Host` check** — a request must name `127.0.0.1:<port>` or
//!   `localhost:<port>`. A DNS-rebinding page reaches us under its own
//!   hostname, and is refused before anything else is read.
//! * **No state change on `GET`** — every `GET` only reads. Commands are
//!   `POST /command`.
//! * **Session token** — a command must carry `X-Jig-Token`, a random value
//!   minted at daemon start and embedded in the page. Another origin cannot
//!   read the page to learn it, and a custom header cannot be sent
//!   cross-origin without a CORS preflight, which is never granted. A
//!   present `Origin` header must be ours too.

mod http;
mod page;

use std::io::Read;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use uuid::Uuid;

use super::commands::{Command, WorkerKey};
use super::ipc::{DaemonStatus, Request, Response};
use super::DaemonShared;
use crate::worker::events::SetupState;
use http::{read_request, HttpError, HttpRequest, HttpResponse};

/// How long a connection may take to send its request, or to take a reply.
const IO_TIMEOUT: Duration = Duration::from_secs(5);

/// Connections served at once; more are told to come back later.
const MAX_CONNECTIONS: usize = 32;

/// The longest the listener waits before re-checking `quit`.
const SHUTDOWN_CHECK: Duration = Duration::from_millis(500);

/// The most of a `setup.log` one request returns — its end.
const MAX_LOG_BYTES: u64 = 1024 * 1024;

/// What the web view needs from the daemon. [`DaemonShared`] in the daemon;
/// a stand-in in tests.
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

/// The web view's bound port.
pub struct WebServer {
    listener: TcpListener,
    port: u16,
    token: String,
}

impl WebServer {
    /// Bind `127.0.0.1:port` — never another interface. Port 0 picks one.
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

    /// Serve until `quit` is set, on a thread of its own.
    pub fn spawn<B: Backend>(
        self,
        backend: B,
        quit: Arc<AtomicBool>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new()
            .name("jig-web".into())
            .spawn(move || self.serve(backend, &quit))
            .expect("failed to spawn jig-web thread")
    }

    fn serve<B: Backend>(self, backend: B, quit: &AtomicBool) {
        tracing::info!(url = %self.url(), "web view listening");
        let site = Arc::new(Site {
            backend,
            port: self.port,
            token: self.token,
        });
        let active = Arc::new(AtomicUsize::new(0));
        while !quit.load(Ordering::Relaxed) {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
                    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
                    let Some(slot) = Slot::take(&active) else {
                        let mut stream = stream;
                        let _ =
                            HttpResponse::text(503, "too many connections").write_to(&mut stream);
                        continue;
                    };
                    let site = Arc::clone(&site);
                    let spawned = std::thread::Builder::new()
                        .name("jig-web-conn".into())
                        .spawn(move || {
                            let _slot = slot;
                            serve_connection(stream, &site);
                        });
                    if let Err(e) = spawned {
                        tracing::warn!("web view could not spawn a connection thread: {}", e);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    wait_for_client(&self.listener)
                }
                Err(e) => {
                    tracing::warn!("web view accept failed: {}", e);
                    std::thread::sleep(SHUTDOWN_CHECK);
                }
            }
        }
        tracing::info!("web view stopped");
    }
}

/// Sleep on the socket until a client connects or it is time to check
/// `quit` — as the IPC listener does.
fn wait_for_client(listener: &TcpListener) {
    use std::os::fd::AsFd;

    use nix::poll::{poll, PollFd, PollFlags, PollTimeout};

    let mut fds = [PollFd::new(listener.as_fd(), PollFlags::POLLIN)];
    let timeout = PollTimeout::try_from(SHUTDOWN_CHECK).unwrap_or(PollTimeout::MAX);
    let _ = poll(&mut fds, timeout);
}

/// One of [`MAX_CONNECTIONS`]; given back on drop.
struct Slot(Arc<AtomicUsize>);

impl Slot {
    fn take(active: &Arc<AtomicUsize>) -> Option<Self> {
        let prev = active.fetch_add(1, Ordering::AcqRel);
        if prev >= MAX_CONNECTIONS {
            active.fetch_sub(1, Ordering::AcqRel);
            None
        } else {
            Some(Self(Arc::clone(active)))
        }
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

fn serve_connection<B: Backend>(mut stream: TcpStream, site: &Site<B>) {
    let response = match read_request(&mut stream) {
        Ok(req) => site.respond(&req),
        Err(HttpError::Closed) => return,
        Err(HttpError::TooLarge) => HttpResponse::text(413, "request too large"),
        Err(e) => HttpResponse::text(400, e.to_string()),
    };
    if let Err(e) = site.harden(response).write_to(&mut stream) {
        tracing::debug!("web view reply failed: {}", e);
    }
}

/// Everything a request is answered from.
struct Site<B> {
    backend: B,
    port: u16,
    token: String,
}

impl<B: Backend> Site<B> {
    fn respond(&self, req: &HttpRequest) -> HttpResponse {
        if !self.host_allowed(req.header("host")) {
            return HttpResponse::text(403, "forbidden host");
        }
        let now = chrono::Utc::now().timestamp();
        match (req.method.as_str(), req.path.as_str()) {
            ("GET", "/") => {
                HttpResponse::html(page::document(&self.backend.status(), &self.token, now))
            }
            ("GET", "/view") => HttpResponse::html(page::view(&self.backend.status(), now)),
            ("GET", "/app.js") => HttpResponse::new(
                200,
                "text/javascript; charset=utf-8",
                include_str!("app.js"),
            ),
            ("GET", "/app.css") => {
                HttpResponse::new(200, "text/css; charset=utf-8", include_str!("app.css"))
            }
            ("GET", "/setup-log") => self.setup_log(req),
            ("POST", "/command") => self.command(req),
            (_, "/" | "/view" | "/app.js" | "/app.css" | "/setup-log" | "/command") => {
                HttpResponse::text(405, "method not allowed")
            }
            _ => HttpResponse::text(404, "not found"),
        }
    }

    /// Headers on every response: never cached, never framed, and no
    /// script or style but our own.
    fn harden(&self, response: HttpResponse) -> HttpResponse {
        response
            .with_header("Cache-Control", "no-store")
            .with_header("X-Content-Type-Options", "nosniff")
            .with_header("X-Frame-Options", "DENY")
            .with_header("Referrer-Policy", "no-referrer")
            .with_header(
                "Content-Security-Policy",
                "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; \
                 frame-ancestors 'none'; base-uri 'none'; form-action 'none'",
            )
    }

    /// Whether `host` names this server. Anything else is a page that
    /// reached us through a hostname it controls.
    fn host_allowed(&self, host: Option<&str>) -> bool {
        let Some(host) = host else { return false };
        let port = self.port;
        host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
    }

    /// Whether `origin`, when sent, is one of ours.
    fn origin_allowed(&self, origin: Option<&str>) -> bool {
        match origin {
            None => true,
            Some(origin) => origin
                .strip_prefix("http://")
                .is_some_and(|host| self.host_allowed(Some(host))),
        }
    }

    fn command(&self, req: &HttpRequest) -> HttpResponse {
        let refuse =
            |status, error: &str| HttpResponse::json(status, &json!({"ok": false, "error": error}));

        let token_ok = req
            .header("x-jig-token")
            .is_some_and(|t| constant_time_eq(t.as_bytes(), self.token.as_bytes()));
        if !token_ok {
            return refuse(403, "missing or wrong token — reload the page");
        }
        if !self.origin_allowed(req.header("origin")) {
            return refuse(403, "forbidden origin");
        }

        // The body is an IPC `Request`, verbs only — one format for both.
        let request: Request = match serde_json::from_slice(&req.body) {
            Ok(r) => r,
            Err(e) => return refuse(400, &format!("bad command: {e}")),
        };
        let Ok((worker, cmd)) = request.into_command() else {
            return refuse(400, "not a command");
        };
        let verb = cmd.name();
        tracing::info!(worker = %worker, command = verb, "command over web");
        match self.backend.run_command(worker.clone(), cmd) {
            Response::Ok => HttpResponse::json(
                200,
                &json!({"ok": true, "message": format!("{verb} {worker}: done")}),
            ),
            Response::Error(reason) => refuse(409, &format!("{verb} {worker}: {reason}")),
            other => refuse(500, &format!("unexpected answer: {other:?}")),
        }
    }

    /// The `setup.log` of the worker named by `?worker=`. Only a path the
    /// daemon itself recorded is ever read — the request names a worker,
    /// not a file.
    fn setup_log(&self, req: &HttpRequest) -> HttpResponse {
        let Some(key) = req
            .query_param("worker")
            .and_then(|k| k.parse::<WorkerKey>().ok())
        else {
            return HttpResponse::text(400, "expected ?worker=repo/branch");
        };
        let status = self.backend.status();
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
            return HttpResponse::text(404, format!("no setup.log for {key}"));
        };
        match read_tail(&path, MAX_LOG_BYTES) {
            Ok(bytes) => HttpResponse::text(200, bytes),
            Err(e) => HttpResponse::text(404, format!("cannot read setup.log for {key}: {e}")),
        }
    }
}

/// The last `max` bytes of `path`.
fn read_tail(path: &std::path::Path, max: u64) -> std::io::Result<Vec<u8>> {
    use std::io::{Seek, SeekFrom};

    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    if len > max {
        file.seek(SeekFrom::Start(len - max))?;
    }
    let mut out = Vec::new();
    file.take(max).read_to_end(&mut out)?;
    Ok(out)
}

/// Compare without leaking how much of the token matched.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests;
