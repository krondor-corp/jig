//! IPC — the daemon answers for itself over a unix socket.
//!
//! Wire format is newline-delimited JSON, one `Request` in and one
//! `Response` out per connection, matching the JSONL event logs elsewhere in
//! jig and needing nothing outside the standard library.
//!
//! Two halves live here:
//!
//! * **Client** — [`request`], used by `jig daemon status/stop` and `jig ps`.
//!   A refused connection means "no daemon", not an error to show the user;
//!   every caller falls back to doing the work in-process.
//! * **Server** — [`Server`], a listener thread the daemon owns. It reads
//!   [`DaemonShared`] rather than the `Daemon` itself, so answering a status
//!   request never blocks (or is blocked by) a tick.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use url::Url;

use jig_core::issues::issue::IssueRef;
use jig_core::mux::AgentState;

use super::actors::triage::TriageEntry;
use super::actors::ActorActivity;
use super::checks::PrHealth;
use super::DaemonShared;
use crate::context::{daemon_socket_path, RepoEntry};
use crate::worker::events::WorkerState;
use crate::worker::{MuxStatus, WorkerStatus};

/// An actor busy longer than this is reported as possibly stuck.
pub const STUCK_ACTOR_SECS: i64 = 10 * 60;

/// How long a client waits on a daemon that accepted the connection but has
/// not answered. Generous: the listener may be mid-tick on a loaded box.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

/// How long the server waits for a client that connected and went quiet, so
/// one wedged client cannot stall the accept loop.
const SERVER_TIMEOUT: Duration = Duration::from_secs(2);

/// Accept-loop poll interval — the gap between "quit was set" and the
/// listener thread noticing.
const ACCEPT_POLL: Duration = Duration::from_millis(500);

// ── Protocol ────────────────────────────────────────────────────────

/// A question for the daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "snake_case")]
pub enum Request {
    /// Liveness and identity only — cheap enough to call in a loop.
    Ping,
    /// Everything `jig ps` renders.
    GetStatus,
    /// Ask the daemon to stop after its current tick.
    Shutdown,
}

/// The daemon's answer.
///
/// Adjacently tagged (`{"response": "...", "body": ...}`) because internal
/// tagging cannot carry a variant whose payload is a bare string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "response", content = "body", rename_all = "snake_case")]
pub enum Response {
    Pong(DaemonInfo),
    // Boxed: `Status` is orders of magnitude larger than the other variants.
    Status(Box<DaemonStatus>),
    Ok,
    Error(String),
}

/// Who the daemon is and whether it is keeping up.
///
/// Carries the per-actor busy/finished times that the heartbeat file used to
/// hold — the only way to see an actor wedged inside a tick loop that is
/// otherwise healthy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonInfo {
    pub pid: u32,
    pub version: String,
    pub started_at: i64,
    pub ticked_at: i64,
    pub tick_interval: u64,
    /// The daemon's own tracing log.
    pub log: Option<PathBuf>,
    pub actors: Vec<ActorActivity>,
}

/// What the daemon knows right now — the frame `jig ps` renders.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub info: DaemonInfo,
    pub workers: Vec<WorkerSnapshot>,
    pub triages: Vec<TriageEntry>,
    /// Workers the spawn actor is bringing up right now.
    pub spawning: Vec<String>,
    /// Seconds until the next poll tick.
    pub poll_remaining: u64,
}

/// What a daemon says about its own health.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    /// Answering and ticking on schedule.
    Running,
    /// Answering, but the tick loop has not run for a while — the listener
    /// thread is fine and something in the tick is wedged.
    Stalled,
}

impl DaemonInfo {
    pub fn liveness(&self, now: i64) -> Liveness {
        if now - self.ticked_at > self.stall_after_secs() {
            Liveness::Stalled
        } else {
            Liveness::Running
        }
    }

    /// Several missed ticks, with slack for a slow tick on a slow machine.
    fn stall_after_secs(&self) -> i64 {
        3 * self.tick_interval as i64 + 30
    }

    /// Actors whose current request has been running suspiciously long.
    pub fn stuck_actors(&self, now: i64) -> impl Iterator<Item = &ActorActivity> {
        self.actors.iter().filter(move |a| {
            a.busy_since
                .is_some_and(|since| now - since > STUCK_ACTOR_SECS)
        })
    }
}

/// A fully serializable projection of [`WorkerState`].
///
/// This indirection is the point: `WorkerState` carries runtime-only types
/// (`Url`) and grows fields freely, so converting through a snapshot makes
/// "does this cross the wire?" a compile error at one boundary instead of a
/// serde failure at runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerSnapshot {
    // ── Event-derived ───────────────────────────────────────────
    pub status: WorkerStatus,
    pub branch: Option<String>,
    pub commit_count: u32,
    pub last_commit_at: Option<i64>,
    pub pr_url: Option<String>,
    pub nudge_counts: HashMap<String, u32>,
    pub last_nudge_at: HashMap<String, i64>,
    pub issue_ref: Option<IssueRef>,
    pub started_at: Option<i64>,
    pub last_event_at: Option<i64>,
    pub review_feedback_count: u32,
    pub pr_ci_passed: Option<bool>,
    pub pr_ci_failures: Vec<String>,
    pub pr_has_conflicts: Option<bool>,
    pub pr_review_comment_count: u32,
    pub pr_changes_requested: u32,
    pub pr_bad_commits: Vec<String>,
    pub is_draft: bool,

    // ── Runtime ─────────────────────────────────────────────────
    pub repo: Option<RepoEntry>,
    pub name: String,
    pub resolved_branch: String,
    pub mux_status: MuxStatus,
    pub mux_agent_state: Option<AgentState>,
    pub commits_ahead: usize,
    pub is_dirty: bool,
    pub parsed_pr_url: Option<String>,
    pub pr_health: PrHealth,
    pub nudge_cooldown_remaining: Option<u64>,
}

impl From<&WorkerState> for WorkerSnapshot {
    fn from(w: &WorkerState) -> Self {
        Self {
            status: w.status,
            branch: w.branch.clone(),
            commit_count: w.commit_count,
            last_commit_at: w.last_commit_at,
            pr_url: w.pr_url.clone(),
            nudge_counts: w.nudge_counts.clone(),
            last_nudge_at: w.last_nudge_at.clone(),
            issue_ref: w.issue_ref.clone(),
            started_at: w.started_at,
            last_event_at: w.last_event_at,
            review_feedback_count: w.review_feedback_count,
            pr_ci_passed: w.pr_ci_passed,
            pr_ci_failures: w.pr_ci_failures.clone(),
            pr_has_conflicts: w.pr_has_conflicts,
            pr_review_comment_count: w.pr_review_comment_count,
            pr_changes_requested: w.pr_changes_requested,
            pr_bad_commits: w.pr_bad_commits.clone(),
            is_draft: w.is_draft,

            repo: w.repo.clone(),
            name: w.name.clone(),
            resolved_branch: w.resolved_branch.to_string(),
            mux_status: w.mux_status,
            mux_agent_state: w.mux_agent_state,
            commits_ahead: w.commits_ahead,
            is_dirty: w.is_dirty,
            parsed_pr_url: w.parsed_pr_url.as_ref().map(Url::to_string),
            pr_health: w.pr_health.clone(),
            nudge_cooldown_remaining: w.nudge_cooldown_remaining,
        }
    }
}

impl WorkerSnapshot {
    /// Rebuild the `WorkerState` the `ps` renderers take.
    ///
    /// The UI layer reaches into a dozen `WorkerState` fields across as many
    /// render functions; converting back here keeps IPC an implementation
    /// detail of where the data came from rather than a second render path.
    pub fn to_display_state(&self) -> WorkerState {
        WorkerState {
            status: self.status,
            branch: self.branch.clone(),
            commit_count: self.commit_count,
            last_commit_at: self.last_commit_at,
            pr_url: self.pr_url.clone(),
            nudge_counts: self.nudge_counts.clone(),
            last_nudge_at: self.last_nudge_at.clone(),
            issue_ref: self.issue_ref.clone(),
            started_at: self.started_at,
            last_event_at: self.last_event_at,
            review_feedback_count: self.review_feedback_count,
            pr_ci_passed: self.pr_ci_passed,
            pr_ci_failures: self.pr_ci_failures.clone(),
            pr_has_conflicts: self.pr_has_conflicts,
            pr_review_comment_count: self.pr_review_comment_count,
            pr_changes_requested: self.pr_changes_requested,
            pr_bad_commits: self.pr_bad_commits.clone(),
            is_draft: self.is_draft,

            repo: self.repo.clone(),
            name: self.name.clone(),
            resolved_branch: jig_core::git::Branch::new(self.resolved_branch.clone()),
            mux_status: self.mux_status,
            mux_agent_state: self.mux_agent_state,
            commits_ahead: self.commits_ahead,
            is_dirty: self.is_dirty,
            parsed_pr_url: self
                .parsed_pr_url
                .as_deref()
                .and_then(|u| Url::parse(u).ok()),
            pr_health: self.pr_health.clone(),
            nudge_cooldown_remaining: self.nudge_cooldown_remaining,
        }
    }

    /// Whether this worker belongs to `repo_root`.
    ///
    /// The daemon is always global; a repo-scoped `jig ps` filters its frame
    /// down rather than asking for a narrower one.
    pub fn in_repo(&self, repo_root: &std::path::Path) -> bool {
        self.repo.as_ref().is_some_and(|r| r.path == repo_root)
    }
}

// ── Errors ──────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("daemon is not running")]
    NotRunning,
    #[error("daemon already running (pid {0})")]
    AlreadyRunning(u32),
    #[error("daemon socket {0}: {1}")]
    Socket(PathBuf, std::io::Error),
    #[error("failed to talk to the daemon: {0}")]
    Io(#[from] std::io::Error),
    #[error("malformed daemon response: {0}")]
    Protocol(String),
    #[error("daemon refused the request: {0}")]
    Refused(String),
}

// ── Client ──────────────────────────────────────────────────────────

/// Send one request and read one response.
///
/// `Err(IpcError::NotRunning)` means nothing is listening — callers treat
/// that as "do it in-process", not as a failure.
pub fn request(req: &Request) -> Result<Response, IpcError> {
    let path = daemon_socket_path()?;
    let stream = UnixStream::connect(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused => {
            IpcError::NotRunning
        }
        _ => IpcError::Socket(path.clone(), e),
    })?;
    stream.set_read_timeout(Some(CLIENT_TIMEOUT))?;
    stream.set_write_timeout(Some(CLIENT_TIMEOUT))?;

    write_message(&stream, req)?;
    let response: Response = read_message(&stream)?;
    match response {
        Response::Error(msg) => Err(IpcError::Refused(msg)),
        other => Ok(other),
    }
}

/// Identity of the running daemon, or `None` when none answers.
pub fn ping() -> Result<Option<DaemonInfo>, IpcError> {
    match request(&Request::Ping) {
        Ok(Response::Pong(info)) => Ok(Some(info)),
        Ok(other) => Err(IpcError::Protocol(format!("expected pong, got {other:?}"))),
        Err(IpcError::NotRunning) => Ok(None),
        Err(e) => Err(e),
    }
}

/// The daemon's current frame, or `None` when no daemon is running.
pub fn status() -> Result<Option<DaemonStatus>, IpcError> {
    match request(&Request::GetStatus) {
        Ok(Response::Status(status)) => Ok(Some(*status)),
        Ok(other) => Err(IpcError::Protocol(format!(
            "expected status, got {other:?}"
        ))),
        Err(IpcError::NotRunning) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Cheap "is anyone listening?" — used to pick IPC over an in-process tick.
pub fn is_running() -> bool {
    matches!(ping(), Ok(Some(_)))
}

// ── Framing ─────────────────────────────────────────────────────────

fn write_message<T: Serialize>(mut stream: &UnixStream, msg: &T) -> Result<(), IpcError> {
    let mut line = serde_json::to_vec(msg).map_err(|e| IpcError::Protocol(e.to_string()))?;
    line.push(b'\n');
    stream.write_all(&line)?;
    stream.flush()?;
    Ok(())
}

fn read_message<T: for<'de> Deserialize<'de>>(stream: &UnixStream) -> Result<T, IpcError> {
    let mut line = String::new();
    let read = BufReader::new(stream).read_line(&mut line)?;
    if read == 0 {
        return Err(IpcError::Protocol("connection closed early".into()));
    }
    serde_json::from_str(&line).map_err(|e| IpcError::Protocol(e.to_string()))
}

// ── Server ──────────────────────────────────────────────────────────

/// The daemon's bound socket. Unlinks itself on drop.
pub struct Server {
    listener: UnixListener,
    path: PathBuf,
}

impl Server {
    /// Bind the daemon socket, clearing one a crashed daemon left behind.
    ///
    /// A socket file whose owner is gone still exists on disk but refuses
    /// connections, so `connect()` is the liveness test: it succeeding means
    /// a real daemon is there and we must not steal its address.
    pub fn bind() -> Result<Self, IpcError> {
        let path = daemon_socket_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if path.exists() {
            match UnixStream::connect(&path) {
                Ok(_) => {
                    let pid = super::pidfile::PidFile::running_pid()
                        .ok()
                        .flatten()
                        .unwrap_or(0);
                    return Err(IpcError::AlreadyRunning(pid));
                }
                Err(_) => {
                    tracing::info!(path = %path.display(), "removing stale daemon socket");
                    std::fs::remove_file(&path)?;
                }
            }
        }

        let listener = UnixListener::bind(&path).map_err(|e| IpcError::Socket(path.clone(), e))?;
        listener.set_nonblocking(true)?;
        // The socket carries this user's worker state; keep it to the owner
        // even when the fallback directory is more permissive.
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));

        Ok(Self { listener, path })
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Serve requests until `quit` is set, on a thread of its own.
    ///
    /// A `Shutdown` request sets `quit` too, which both stops this loop and
    /// ends the daemon's tick loop.
    pub fn spawn(self, shared: DaemonShared, quit: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new()
            .name("jig-ipc".into())
            .spawn(move || self.serve(&shared, &quit))
            .expect("failed to spawn jig-ipc thread")
    }

    fn serve(&self, shared: &DaemonShared, quit: &AtomicBool) {
        tracing::info!(socket = %self.path.display(), "daemon listening");
        while !quit.load(Ordering::Relaxed) {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if let Err(e) = handle_connection(&stream, shared, quit) {
                        tracing::warn!("ipc request failed: {}", e);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(ACCEPT_POLL)
                }
                Err(e) => {
                    tracing::warn!("ipc accept failed: {}", e);
                    std::thread::sleep(ACCEPT_POLL);
                }
            }
        }
        tracing::info!("daemon listener stopped");
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Read one request, answer it, close. A panic while answering must not take
/// the listener down with it — the daemon would go mute but keep ticking.
fn handle_connection(
    stream: &UnixStream,
    shared: &DaemonShared,
    quit: &AtomicBool,
) -> Result<(), IpcError> {
    stream.set_read_timeout(Some(SERVER_TIMEOUT))?;
    stream.set_write_timeout(Some(SERVER_TIMEOUT))?;

    let req: Request = read_message(stream)?;
    let response =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| dispatch(req, shared, quit)))
            .unwrap_or_else(|_| Response::Error("daemon panicked while answering".into()));

    write_message(stream, &response)
}

fn dispatch(req: Request, shared: &DaemonShared, quit: &AtomicBool) -> Response {
    match req {
        Request::Ping => Response::Pong(shared.info()),
        Request::GetStatus => Response::Status(Box::new(shared.status())),
        Request::Shutdown => {
            tracing::info!("shutdown requested over ipc");
            quit.store(true, Ordering::Relaxed);
            Response::Ok
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(ticked_at: i64, actors: Vec<ActorActivity>) -> DaemonInfo {
        DaemonInfo {
            pid: 1,
            version: "0.0.0".into(),
            started_at: 0,
            ticked_at,
            tick_interval: 2,
            log: None,
            actors,
        }
    }

    fn actor(name: &str, busy_since: Option<i64>) -> ActorActivity {
        ActorActivity {
            name: name.into(),
            busy_since,
            last_finished: None,
        }
    }

    fn sample_worker() -> WorkerState {
        WorkerState {
            name: "feat/socket".into(),
            branch: Some("feat/socket".into()),
            resolved_branch: jig_core::git::Branch::new("feat/socket"),
            status: WorkerStatus::Running,
            commit_count: 3,
            commits_ahead: 2,
            is_dirty: true,
            mux_status: MuxStatus::Running,
            mux_agent_state: Some(AgentState::Working),
            parsed_pr_url: Some(Url::parse("https://github.com/o/r/pull/12").unwrap()),
            pr_url: Some("https://github.com/o/r/pull/12".into()),
            issue_ref: Some(IssueRef::new("KRO-213")),
            nudge_cooldown_remaining: Some(90),
            pr_health: PrHealth {
                has_pr: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn requests_round_trip() {
        for req in [Request::Ping, Request::GetStatus, Request::Shutdown] {
            let json = serde_json::to_string(&req).unwrap();
            assert_eq!(serde_json::from_str::<Request>(&json).unwrap(), req);
        }
    }

    #[test]
    fn responses_round_trip() {
        let responses = [
            Response::Pong(info(5, vec![actor("jig-sync", Some(3))])),
            Response::Status(Box::new(DaemonStatus {
                info: info(5, vec![]),
                workers: vec![WorkerSnapshot::from(&sample_worker())],
                triages: vec![TriageEntry {
                    spawned_at: 1,
                    issue_id: "KRO-1".into(),
                    repo_name: "jig".into(),
                    model: "triage-1".into(),
                }],
                spawning: vec!["feat/x".into()],
                poll_remaining: 42,
            })),
            Response::Ok,
            Response::Error("nope".into()),
        ];
        for resp in responses {
            let json = serde_json::to_string(&resp).unwrap();
            assert_eq!(serde_json::from_str::<Response>(&json).unwrap(), resp);
        }
    }

    #[test]
    fn snapshot_preserves_what_ps_renders() {
        let worker = sample_worker();
        let restored = WorkerSnapshot::from(&worker).to_display_state();

        assert_eq!(restored.name, worker.name);
        assert_eq!(restored.status, worker.status);
        assert_eq!(restored.resolved_branch, worker.resolved_branch);
        assert_eq!(restored.commits_ahead, worker.commits_ahead);
        assert_eq!(restored.is_dirty, worker.is_dirty);
        assert_eq!(restored.mux_status, worker.mux_status);
        assert_eq!(restored.mux_agent_state, worker.mux_agent_state);
        assert_eq!(restored.parsed_pr_url, worker.parsed_pr_url);
        assert_eq!(restored.issue_ref, worker.issue_ref);
        assert_eq!(restored.pr_health, worker.pr_health);
        assert_eq!(
            restored.nudge_cooldown_remaining,
            worker.nudge_cooldown_remaining
        );
    }

    #[test]
    fn snapshot_survives_the_wire() {
        let snapshot = WorkerSnapshot::from(&sample_worker());
        let json = serde_json::to_string(&snapshot).unwrap();
        assert_eq!(
            serde_json::from_str::<WorkerSnapshot>(&json).unwrap(),
            snapshot
        );
    }

    #[test]
    fn fresh_tick_is_running() {
        assert_eq!(info(1000, vec![]).liveness(1010), Liveness::Running);
    }

    #[test]
    fn missed_ticks_are_stalled() {
        // tick_interval 2 → stalled after 36s without a tick
        let info = info(1000, vec![]);
        assert_eq!(info.liveness(1036), Liveness::Running);
        assert_eq!(info.liveness(1037), Liveness::Stalled);
    }

    #[test]
    fn only_long_busy_actors_are_stuck() {
        let now = 10_000;
        let info = info(
            now,
            vec![
                actor("idle", None),
                actor("brief", Some(now - 30)),
                actor("wedged", Some(now - STUCK_ACTOR_SECS - 1)),
            ],
        );
        let stuck: Vec<_> = info.stuck_actors(now).map(|a| a.name.as_str()).collect();
        assert_eq!(stuck, vec!["wedged"]);
    }

    #[test]
    fn in_repo_matches_the_owning_repo() {
        let mut worker = sample_worker();
        let now = chrono::Utc::now();
        worker.repo = Some(RepoEntry {
            path: PathBuf::from("/src/jig"),
            added: now,
            last_used: now,
        });
        let snapshot = WorkerSnapshot::from(&worker);
        assert!(snapshot.in_repo(std::path::Path::new("/src/jig")));
        assert!(!snapshot.in_repo(std::path::Path::new("/src/other")));
    }

    #[test]
    fn a_worker_with_no_repo_belongs_to_none() {
        let snapshot = WorkerSnapshot::from(&sample_worker());
        assert!(!snapshot.in_repo(std::path::Path::new("/src/jig")));
    }
}
