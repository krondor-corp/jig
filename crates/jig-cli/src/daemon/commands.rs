//! Control verbs — `kill`, `nudge`, `prune`, `retry-setup`, `model`.
//!
//! NOTE(canonical): this is the one code path that executes a control verb
//! against a worker. The daemon runs it for an IPC [`Request`]; the CLI runs
//! it in-process when no daemon answers ([`run`]); the web view and remote
//! peers will reach it through the daemon. **Do not** kill, nudge, prune or
//! re-set-up a worker on someone's behalf anywhere else — add a [`Command`].
//!
//! Every verb is addressed by [`WorkerKey`] (`repo/branch`), answers
//! `Ok` or an error with a reason, and leaves a
//! [`EventKind::Command`] in the worker's event log.

use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use jig_core::git::Branch;
use jig_core::issues::issue::IssueRef;
use jig_core::mux::{self, Mux, MuxError};
use jig_core::prompt::Prompt;

use super::actors::prune::{prune_single, PruneTarget};
use super::ipc::{self, IpcError, Request, Response};
use crate::context::{AppConfig, AppCtx, AppPaths, ContextError, RepoCtx, RepoRegistry};
use crate::prompts::{spawn_task, spawn_task_raw};
use crate::worker::events::{Event, EventKind};
use crate::worker::{Worker, WorkerError};

// ── Addressing ──────────────────────────────────────────────────────

/// Which worker a command is for: `repo/branch`, where `repo` is the name of
/// the repo's root directory — the key [`Worker::worker_key`] produces.
///
/// The repo is everything before the first `/`; branches may contain more.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct WorkerKey {
    pub repo: String,
    pub branch: Branch,
}

impl WorkerKey {
    pub fn new(repo: impl Into<String>, branch: Branch) -> Self {
        Self {
            repo: repo.into(),
            branch,
        }
    }

    /// The key for `worker` in `repo`.
    pub fn of(repo: &RepoCtx, worker: &Worker) -> Self {
        Self::new(repo.name(), worker.branch().clone())
    }
}

impl fmt::Display for WorkerKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.repo, self.branch)
    }
}

impl FromStr for WorkerKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Keys arrive over the socket and from the web view: a bad branch
        // name is an error here, not a panic in `Branch::new`.
        match s.split_once('/') {
            Some((repo, branch)) if !repo.is_empty() && !branch.is_empty() => {
                let branch = Branch::try_new(branch)
                    .ok_or_else(|| format!("'{branch}' is not a valid branch name"))?;
                Ok(Self::new(repo, branch))
            }
            _ => Err(format!("worker key '{s}' is not of the form repo/branch")),
        }
    }
}

impl TryFrom<String> for WorkerKey {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<WorkerKey> for String {
    fn from(key: WorkerKey) -> Self {
        key.to_string()
    }
}

// ── Verbs ───────────────────────────────────────────────────────────

/// A control verb, without the worker it is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Close the worker's window and forget its logs; the worktree stays.
    Kill,
    /// Type free text into the agent's session.
    Nudge { text: String },
    /// Remove the worktree and logs of a worker whose agent is not running.
    Prune,
    /// Re-run `on_create` in a worktree whose setup failed, then start the
    /// agent — see [`Worker::retry_setup`].
    RetrySetup,
    /// Switch the agent's model. v0: types `/model <model>` into the session.
    Model { model: String },
}

impl Command {
    /// The verb as typed on the command line, and as recorded in the log.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Kill => "kill",
            Self::Nudge { .. } => "nudge",
            Self::Prune => "prune",
            Self::RetrySetup => "retry-setup",
            Self::Model { .. } => "model",
        }
    }

    /// The same verb as a request for the daemon.
    pub fn request(self, worker: WorkerKey) -> Request {
        match self {
            Self::Kill => Request::Kill { worker },
            Self::Nudge { text } => Request::Nudge { worker, text },
            Self::Prune => Request::Prune { worker },
            Self::RetrySetup => Request::RetrySetup { worker },
            Self::Model { model } => Request::Model { worker, model },
        }
    }

    /// What goes in the worker's event log.
    fn event(&self) -> EventKind {
        let detail = match self {
            Self::Nudge { text } => Some(text.clone()),
            Self::Model { model } => Some(model.clone()),
            Self::Kill | Self::Prune | Self::RetrySetup => None,
        };
        EventKind::Command {
            command: self.name().to_string(),
            detail,
        }
    }

    /// Reject arguments no worker could act on, before touching anything.
    fn validate(&self) -> Result<(), CommandError> {
        match self {
            Self::Nudge { text } if text.trim().is_empty() => {
                Err(CommandError::Invalid("nudge text is empty".into()))
            }
            Self::Model { model } if model.trim().is_empty() => {
                Err(CommandError::Invalid("model name is empty".into()))
            }
            // A newline would submit `/model` early and type the rest as a
            // message to the agent.
            Self::Model { model } if model.contains(char::is_whitespace) => Err(
                CommandError::Invalid(format!("'{model}' is not a model name")),
            ),
            _ => Ok(()),
        }
    }
}

// ── Errors ──────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("no tracked repo named '{0}'")]
    UnknownRepo(String),
    #[error("{0}")]
    Invalid(String),
    #[error("worker '{0}' has no window — is its agent running?")]
    NoWindow(String),
    #[error("worker '{0}' is still running — kill it first")]
    StillRunning(String),
    #[error("worker '{0}' is busy with another command")]
    Busy(WorkerKey),
    #[error("failed to prune '{0}': {1}")]
    Prune(String, String),
    /// The daemon ran the command and it failed; the reason is its own.
    #[error("{0}")]
    Daemon(String),
    #[error(transparent)]
    Worker(#[from] WorkerError),
    #[error(transparent)]
    Mux(#[from] MuxError),
    #[error(transparent)]
    Ctx(#[from] ContextError),
    #[error(transparent)]
    Ipc(IpcError),
}

// ── Execution ───────────────────────────────────────────────────────

/// Where a command ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    Daemon,
    InProcess,
}

/// Run `cmd` on `worker` — through the daemon if one is listening, so it is
/// serialised with every other command it runs, and in this process if not.
pub fn run(
    paths: &AppPaths,
    config: &AppConfig,
    repo: &RepoCtx,
    worker: &Worker,
    cmd: Command,
) -> Result<Via, CommandError> {
    let key = WorkerKey::of(repo, worker);
    match ipc::command(paths, key, cmd.clone()) {
        Ok(true) => Ok(Via::Daemon),
        Ok(false) => {
            let mux = mux::for_repo(config.mux, &repo.name());
            apply(config, repo, worker, &cmd, &mux)?;
            Ok(Via::InProcess)
        }
        Err(IpcError::Refused(reason)) => Err(CommandError::Daemon(reason)),
        Err(e) => Err(CommandError::Ipc(e)),
    }
}

/// Run `cmd` on the worker `key` names — what the daemon does for a request.
pub fn execute(app: &AppCtx, key: &WorkerKey, cmd: &Command) -> Result<(), CommandError> {
    let (repo, worker) = resolve(app, key)?;
    let mux = mux::for_repo(app.config.mux, &repo.name());
    apply(&app.config, &repo, &worker, cmd, &mux)
}

/// Find `key`'s worker among the tracked repos.
///
/// Read fresh each time: a repo first used after the daemon started is in
/// the registry but not in the daemon's startup copy of it.
fn resolve(app: &AppCtx, key: &WorkerKey) -> Result<(RepoCtx, Worker), CommandError> {
    let registry = RepoRegistry::load(&app.paths)?;
    let mut named = false;
    for entry in registry.repos() {
        if entry.path.file_name().is_none_or(|n| *n != *key.repo) {
            continue;
        }
        named = true;
        let Ok(repo) = app.repo_at(&entry.path) else {
            continue;
        };
        match Worker::open(&repo, &key.branch) {
            Ok(worker) => return Ok((repo, worker)),
            Err(WorkerError::NotFound(_)) => continue,
            Err(e) => return Err(e.into()),
        }
    }
    if named {
        Err(WorkerError::NotFound(key.to_string()).into())
    } else {
        Err(CommandError::UnknownRepo(key.repo.clone()))
    }
}

/// Run `cmd` on a worker already in hand. The mux is a parameter so tests
/// can stand in for tmux.
pub fn apply(
    config: &AppConfig,
    repo: &RepoCtx,
    worker: &Worker,
    cmd: &Command,
    mux: &dyn Mux,
) -> Result<(), CommandError> {
    cmd.validate()?;
    let record = || {
        if let Err(e) = worker.event_log().append(&Event::now(cmd.event())) {
            tracing::warn!(worker = %worker.branch(), "failed to record {}: {}", cmd.name(), e);
        }
    };

    match cmd {
        Command::Kill => {
            // Recorded first: killing ends the log. A window that is already
            // gone is no reason to fail — the end state is the same.
            record();
            let _ = worker.kill(mux);
            worker.unregister()?;
        }
        Command::Nudge { text } => {
            require_window(worker, mux)?;
            mux.send_message(worker.branch(), text)?;
            record();
        }
        Command::Model { model } => {
            require_window(worker, mux)?;
            mux.send_message(worker.branch(), &format!("/model {model}"))?;
            record();
        }
        Command::Prune => {
            // Unlike the prune actor, which only sees finished PRs, this can
            // be pointed at live work — and pruning force-removes the tree.
            if worker.has_mux_window(mux) {
                return Err(CommandError::StillRunning(worker.branch().to_string()));
            }
            record();
            let target = PruneTarget {
                repo_path: repo.paths.repo_root.clone(),
                repo_name: repo.name(),
                worker_name: worker.branch().to_string(),
            };
            prune_single(&repo.app_paths, &target)
                .map_err(|e| CommandError::Prune(worker.branch().to_string(), e))?;
            worker.unregister()?;
        }
        Command::RetrySetup => {
            worker.retry_setup(
                repo,
                cmd.event(),
                |issue| relaunch_prompt(config, repo, issue),
                mux,
            )?;
        }
    }
    Ok(())
}

fn require_window(worker: &Worker, mux: &dyn Mux) -> Result<(), CommandError> {
    if worker.has_mux_window(mux) {
        Ok(())
    } else {
        Err(CommandError::NoWindow(worker.branch().to_string()))
    }
}

/// The task for an agent starting after a retried setup: the issue's, the
/// same prompt the spawn actor would have built, when it can be fetched.
fn relaunch_prompt(config: &AppConfig, repo: &RepoCtx, issue: Option<&IssueRef>) -> Prompt {
    let Some(id) = issue else {
        return spawn_task_raw(
            "No specific task provided. Check CLAUDE.md and the issue tracker for context.",
        );
    };
    if let Ok(provider) = repo.issue_provider(config) {
        if let Ok(Some(issue)) = provider.get(id) {
            return spawn_task(&issue, &provider);
        }
    }
    spawn_task_raw(&format!(
        "Work on issue {id}. Look it up in the issue tracker for the task."
    ))
}

// ── Daemon side ─────────────────────────────────────────────────────

/// Workers with a command running in the daemon right now.
///
/// Commands run on threads of their own so a slow one (a `retry-setup` runs
/// a whole `on_create` hook) never holds up the listener or the tick loop.
/// That also means two could reach one worker at once — a double-clicked
/// button — so a second command for a busy worker is refused, not queued.
#[derive(Clone, Default)]
pub struct InFlight(Arc<Mutex<HashSet<WorkerKey>>>);

/// A worker claimed by [`InFlight::claim`]; released on drop.
pub struct Claim {
    set: InFlight,
    key: WorkerKey,
}

impl InFlight {
    /// Claim `key`, or `None` if a command already holds it.
    pub fn claim(&self, key: &WorkerKey) -> Option<Claim> {
        let mut set = self.0.lock().unwrap_or_else(|e| e.into_inner());
        set.insert(key.clone()).then(|| Claim {
            set: self.clone(),
            key: key.clone(),
        })
    }
}

impl Drop for Claim {
    fn drop(&mut self) {
        let mut set = self.set.0.lock().unwrap_or_else(|e| e.into_inner());
        set.remove(&self.key);
    }
}

/// Answer one command request. Runs on the command's own thread.
pub fn serve(app: &AppCtx, in_flight: &InFlight, key: WorkerKey, cmd: Command) -> Response {
    let Some(_claim) = in_flight.claim(&key) else {
        return Response::Error(CommandError::Busy(key).to_string());
    };
    tracing::info!(worker = %key, command = cmd.name(), "command over ipc");
    match execute(app, &key, &cmd) {
        Ok(()) => Response::Ok,
        Err(e) => {
            tracing::warn!(worker = %key, command = cmd.name(), "command failed: {}", e);
            Response::Error(e.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_splits_at_the_first_slash() {
        let key: WorkerKey = "jig/feature/kro-249".parse().unwrap();
        assert_eq!(key.repo, "jig");
        assert_eq!(key.branch, Branch::new("feature/kro-249"));
        assert_eq!(key.to_string(), "jig/feature/kro-249");
    }

    #[test]
    fn a_key_needs_both_halves() {
        for bad in [
            "jig",
            "/branch",
            "jig/",
            "",
            "jig/../etc",
            "jig/a b",
            "jig/x..y",
        ] {
            assert!(bad.parse::<WorkerKey>().is_err(), "{bad:?}");
        }
    }

    #[test]
    fn a_key_is_a_string_on_the_wire() {
        let key = WorkerKey::new("jig", Branch::new("feat/x"));
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, r#""jig/feat/x""#);
        assert_eq!(serde_json::from_str::<WorkerKey>(&json).unwrap(), key);
        assert!(serde_json::from_str::<WorkerKey>(r#""nope""#).is_err());
    }

    #[test]
    fn empty_arguments_are_rejected() {
        assert!(Command::Nudge { text: "  ".into() }.validate().is_err());
        assert!(Command::Model { model: "".into() }.validate().is_err());
        assert!(Command::Model {
            model: "opus\nhi".into()
        }
        .validate()
        .is_err());
        assert!(Command::Model {
            model: "opus".into()
        }
        .validate()
        .is_ok());
        assert!(Command::Kill.validate().is_ok());
    }

    #[test]
    fn the_event_carries_the_free_text() {
        let EventKind::Command { command, detail } = (Command::Nudge {
            text: "rebase please".into(),
        })
        .event() else {
            panic!("expected a command event");
        };
        assert_eq!(command, "nudge");
        assert_eq!(detail.as_deref(), Some("rebase please"));
    }

    #[test]
    fn a_busy_worker_cannot_be_claimed_twice() {
        let in_flight = InFlight::default();
        let key = WorkerKey::new("jig", Branch::new("feat/x"));
        let claim = in_flight.claim(&key).expect("first claim");
        assert!(in_flight.claim(&key).is_none());
        drop(claim);
        assert!(in_flight.claim(&key).is_some());
    }
}
