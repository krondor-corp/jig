//! Worker — the single abstraction for a Claude Code session.
//!
//! A Worker owns its identity and a [`WorktreeRef`] pointing at its
//! git worktree on disk.  The full [`Worktree`] (wrapping a git2 repo
//! handle) is resolved on demand — we never serialize what we can derive.

pub mod events;
mod status;

pub use status::{MuxStatus, WorkerStatus};

use std::path::Path;
use std::time::Instant;

use uuid::Uuid;

use crate::context::log::LogFile;
use crate::context::{worktree_path, RepoCtx, JIG_DIR};
use crate::prompts::resume_task;
use events::{Event, EventKind, SetupPhase, SetupState, TerminalKind, WorkerState};
use jig_core::exec::ExecError;
use jig_core::git::{Branch, Repo, Worktree, WorktreeRef};

use jig_core::issues::issue::IssueRef;
use jig_core::mux::Mux;
use jig_core::prompt::Prompt;

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("worker '{0}' not found")]
    NotFound(String),

    #[error("worker '{0}' is still initializing")]
    Initializing(String),

    #[error("worker '{branch}' failed during setup: {reason}")]
    SetupFailed { branch: String, reason: String },

    #[error("can't retry setup for worker '{branch}': {why}")]
    NotRetryable { branch: String, why: String },

    #[error(transparent)]
    Setup(#[from] SetupError),

    #[error(transparent)]
    Mux(#[from] jig_core::mux::MuxError),

    #[error(transparent)]
    Git(#[from] jig_core::git::GitError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Agent(#[from] jig_core::agents::AgentError),

    #[error(transparent)]
    Template(#[from] handlebars::RenderError),

    #[error(transparent)]
    EventLog(#[from] jig_core::events::EventLogError),
}

/// How many lines of `setup.log` a setup failure quotes.
const SETUP_EXCERPT_LINES: usize = 20;

/// Whether [`Worker::create`] runs the repo's `on_create` hook.
///
/// Skipping it (`jig create --no-hooks`) still copies the configured files:
/// those are what make the worktree usable, not a user extension point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnCreate {
    Run,
    Skip,
}

/// Why [`Worker::create`] or [`Worker::spawn`] couldn't make a fresh worktree
/// ready to work in.
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    /// The hook ran and didn't succeed — exited non-zero, was killed, or
    /// outran its deadline. `excerpt` is [`LogFile::excerpt`] of `setup.log`;
    /// `exit_code`, `timed_out` and `tail` are the same facts as data, for
    /// the `SetupFinished` event.
    #[error("{label} {outcome}:\n{excerpt}")]
    Hook {
        label: String,
        outcome: String,
        excerpt: String,
        exit_code: Option<i32>,
        timed_out: bool,
        tail: Vec<String>,
    },

    /// The hook couldn't be started at all.
    #[error(transparent)]
    Exec(#[from] ExecError),

    /// Copying files or opening `setup.log` failed — nothing anyone handles
    /// differently, so not told apart.
    #[error("unexpected error setting up the worktree: {0}")]
    Io(#[from] std::io::Error),
}

/// A Worker is a Claude Code session in an isolated git worktree.
///
/// Holds its own logs, resolved when it is opened or created. Get one with
/// [`Worker::open`] for a worktree that exists, [`Worker::create`] or
/// [`Worker::spawn`] to make one, or [`Worker::discover`] for all of a repo's.
#[derive(Debug, Clone)]
pub struct Worker {
    pub(crate) id: Uuid,
    pub(crate) branch: Branch,
    pub(crate) path: WorktreeRef,
    pub(crate) issue_ref: Option<IssueRef>,
    event_log: events::EventLog,
    setup_log: LogFile,
}

impl Worker {
    /// The worker for `branch`'s worktree in `repo`: [`WorkerError::NotFound`]
    /// if there is no such worktree. No side effects — safe to call as often
    /// as you like.
    pub fn open(repo: &RepoCtx, branch: &Branch) -> Result<Self, WorkerError> {
        let path = WorktreeRef::new(worktree_path(&repo.paths.repo_root, branch));
        if !path.exists() {
            return Err(WorkerError::NotFound(branch.to_string()));
        }
        path.open()?;
        Ok(Self::at(repo, branch.clone(), path))
    }

    /// Every worker in `repo`, sorted by branch.
    pub fn discover(repo: &RepoCtx) -> Result<Vec<Self>, WorkerError> {
        let git = Repo::open(&repo.paths.repo_root)?;
        let mut workers: Vec<Self> = git
            .list_worktrees()?
            .iter()
            .map(|wt| Self::at(repo, wt.branch_name(), wt.as_ref()))
            .collect();
        workers.sort_by(|a, b| a.branch.cmp(&b.branch));
        Ok(workers)
    }

    /// Create `branch`'s worktree from `base` and set it up. For a worktree
    /// someone will work in by hand — [`Worker::spawn`] also starts an agent.
    pub fn create(
        repo: &RepoCtx,
        branch: &Branch,
        base: &Branch,
        on_create: OnCreate,
    ) -> Result<Self, WorkerError> {
        let worker = Self::init(repo, branch, base, on_create, false, None)?;
        let _ = worker.event_log.append(&Event::now(EventKind::Create {
            branch: branch.to_string(),
        }));
        Ok(worker)
    }

    /// Create `branch`'s worktree from `base`, set it up, and start the
    /// repo's agent in it.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        repo: &RepoCtx,
        branch: &Branch,
        base: &Branch,
        prompt: Prompt,
        auto: bool,
        issue_ref: Option<IssueRef>,
        mux: &dyn Mux,
    ) -> Result<Self, WorkerError> {
        let mut worker = Self::init(repo, branch, base, OnCreate::Run, auto, issue_ref.as_ref())?;
        worker.issue_ref = issue_ref.clone();

        let issue = issue_ref.unwrap_or_else(|| IssueRef::new(branch.to_string()));
        let _ = worker.event_log.append(&Event::now(EventKind::Spawn {
            branch: branch.to_string(),
            repo: repo.name(),
            issue,
        }));

        worker.phase(SetupPhase::AgentLaunch, None, || {
            mux.create_window(&worker.branch, &worker.path)?;
            let cmd = repo.config.agent.agent().spawn(prompt)?;
            mux.send_keys(&worker.branch, &[&cmd, "Enter"])?;
            Ok(())
        })?;

        Ok(worker)
    }

    /// A worker at `path`, with its logs resolved.
    fn at(repo: &RepoCtx, branch: Branch, path: WorktreeRef) -> Self {
        let logs = repo.logs();
        let repo_name = repo.name();
        Self {
            id: Uuid::new_v4(),
            event_log: logs.worker_events(&repo_name, &branch),
            setup_log: logs.setup(&repo_name, &branch),
            branch,
            path,
            issue_ref: None,
        }
    }

    /// What [`Worker::create`] and [`Worker::spawn`] share: start a fresh
    /// event log, create the worktree and set it up, each step a recorded
    /// [`Worker::phase`] so `ps`, `jig attach`, `jig list` and the daemon's
    /// needs_intervention alert can say how far it got and why it stopped.
    fn init(
        repo: &RepoCtx,
        branch: &Branch,
        base: &Branch,
        on_create: OnCreate,
        auto: bool,
        issue: Option<&IssueRef>,
    ) -> Result<Self, WorkerError> {
        let git = Repo::open(&repo.paths.repo_root)?;
        let worker = Self::at(
            repo,
            branch.clone(),
            WorktreeRef::new(worktree_path(&repo.paths.repo_root, branch)),
        );

        worker.event_log.reset()?;
        let _ = worker
            .event_log
            .append(&Event::now(EventKind::Initializing {
                branch: branch.to_string(),
                base: base.to_string(),
                auto,
                issue: issue.cloned(),
            }));

        worker.phase(SetupPhase::Worktree, None, || {
            Worktree::create(&git, branch, base)?;
            Ok(())
        })?;
        worker.setup(repo, on_create)?;

        Ok(worker)
    }

    /// Run one setup phase, recording it as `SetupStarted` / `SetupFinished`.
    /// A failure ends the worker — a `Terminal` failure with the error as
    /// its reason follows.
    fn phase(
        &self,
        phase: SetupPhase,
        log: Option<&LogFile>,
        run: impl FnOnce() -> Result<(), WorkerError>,
    ) -> Result<(), WorkerError> {
        let _ = self.event_log.append(&Event::now(EventKind::SetupStarted {
            phase,
            log_path: log.map(|l| l.path().to_path_buf()),
        }));
        let start = Instant::now();
        let result = run();
        let duration_ms = start.elapsed().as_millis() as u64;

        let (exit_code, timed_out, tail) = match &result {
            Ok(()) => (None, false, Vec::new()),
            Err(WorkerError::Setup(SetupError::Hook {
                exit_code,
                timed_out,
                tail,
                ..
            })) => (*exit_code, *timed_out, tail.clone()),
            Err(e) => (
                None,
                false,
                e.to_string().lines().map(String::from).collect(),
            ),
        };
        let _ = self.event_log.append(&Event::now(EventKind::SetupFinished {
            phase,
            ok: result.is_ok(),
            exit_code,
            timed_out,
            duration_ms,
            tail,
        }));

        if let Err(e) = &result {
            let _ = self.event_log.append(&Event::now(EventKind::Terminal {
                terminal: TerminalKind::Failed,
                reason: Some(e.to_string()),
            }));
        }
        result
    }

    /// Re-run setup in a worktree whose setup failed, and start the agent
    /// if it succeeds — `jig retry-setup`.
    ///
    /// Only for a worker whose setup failed (a failed `SetupFinished` in its
    /// log, `on_create` or the agent launch alike). One that failed later —
    /// its window was lost — has history worth keeping and wants
    /// `jig resume` instead.
    ///
    /// A failure is sticky in the event log, so the retry starts it afresh,
    /// carrying over the base, `auto` flag and issue the first attempt
    /// recorded. `cause` is appended straight after the new `Initializing`
    /// event, so the log says what started this setup. `prompt` builds the
    /// agent's task from the issue, if one was recorded.
    pub fn retry_setup(
        &self,
        repo: &RepoCtx,
        cause: EventKind,
        prompt: impl FnOnce(Option<&IssueRef>) -> Prompt,
        mux: &dyn Mux,
    ) -> Result<(), WorkerError> {
        let branch = self.branch.to_string();
        let state: WorkerState = self.event_log.reduce()?;
        let not_retryable = |why: String| WorkerError::NotRetryable {
            branch: branch.clone(),
            why,
        };
        if state.status != WorkerStatus::Failed {
            return Err(not_retryable(format!(
                "it is {}, not failed",
                state.status.as_str()
            )));
        }
        if !matches!(state.setup, SetupState::Failed { .. }) {
            return Err(not_retryable(
                "it got past setup — use `jig resume` to relaunch it".into(),
            ));
        }
        if self.has_mux_window(mux) {
            return Err(not_retryable(
                "it still has a window — kill it first".into(),
            ));
        }

        let (base, auto) = self
            .event_log
            .read_all()?
            .iter()
            .rev()
            .find_map(|e| match &e.kind {
                EventKind::Initializing { base, auto, .. } => Some((base.clone(), *auto)),
                _ => None,
            })
            .unwrap_or_default();
        let issue = state.issue_ref;

        self.event_log.reset()?;
        let _ = self.event_log.append(&Event::now(EventKind::Initializing {
            branch: branch.clone(),
            base,
            auto,
            issue: issue.clone(),
        }));
        let _ = self.event_log.append(&Event::now(cause));

        self.setup(repo, OnCreate::Run)?;

        let _ = self.event_log.append(&Event::now(EventKind::Spawn {
            branch: branch.clone(),
            repo: repo.name(),
            issue: issue.clone().unwrap_or_else(|| IssueRef::new(&branch)),
        }));
        self.phase(SetupPhase::AgentLaunch, None, || {
            mux.create_window(&self.branch, &self.path)?;
            let cmd = repo.config.agent.agent().spawn(prompt(issue.as_ref()))?;
            mux.send_keys(&self.branch, &[&cmd, "Enter"])?;
            Ok(())
        })
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn branch(&self) -> &Branch {
        &self.branch
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn issue_ref(&self) -> Option<&IssueRef> {
        self.issue_ref.as_ref()
    }

    pub fn worker_key(&self) -> String {
        format!("{}/{}", self.repo_name(), self.branch)
    }

    pub fn worktree(&self) -> Result<Worktree, WorkerError> {
        Ok(self.path.open()?)
    }

    /// This worker's event log.
    pub fn event_log(&self) -> &events::EventLog {
        &self.event_log
    }

    /// This worker's `on_create` output, from its last setup.
    pub fn setup_log(&self) -> &LogFile {
        &self.setup_log
    }

    pub fn status(&self) -> Option<WorkerStatus> {
        if !self.event_log.exists() {
            return None;
        }
        let state: WorkerState = self.event_log.reduce().ok()?;
        Some(state.status)
    }

    pub fn fail_reason(&self) -> Option<String> {
        let events = self.event_log.read_all().ok()?;
        events.iter().rev().find_map(|e| {
            if let EventKind::Terminal {
                reason: Some(r), ..
            } = &e.kind
            {
                Some(r.clone())
            } else {
                None
            }
        })
    }

    pub fn remove(&self, force: bool) -> Result<(), WorkerError> {
        Ok(self.worktree()?.remove(force)?)
    }

    pub fn unregister(&self) -> Result<(), WorkerError> {
        // First, so removing the event log can also remove the now-empty
        // directory.
        let _ = self.setup_log.remove();
        let _ = self.event_log.remove();
        Ok(())
    }

    pub fn repo_name(&self) -> String {
        let path: &Path = &self.path;
        path.ancestors()
            .find(|a| a.file_name().is_some_and(|n| n == JIG_DIR))
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    // ── Mux operations ─────────────────────────────────────────────

    pub fn has_mux_window(&self, mux: &dyn Mux) -> bool {
        mux.window_exists(&self.branch)
    }

    pub fn is_agent_running(&self, mux: &dyn Mux) -> bool {
        mux.is_running(&self.branch)
    }

    pub fn mux_status(&self, mux: &dyn Mux) -> MuxStatus {
        if !mux.window_exists(&self.branch) {
            MuxStatus::NotFound
        } else if mux.is_running(&self.branch) {
            MuxStatus::Running
        } else {
            MuxStatus::Exited
        }
    }

    /// Make a freshly created worktree ready to work in: copy the configured
    /// files over from the main checkout, then run the `on_create` hook.
    ///
    /// NOTE(canonical): this is the one place worktree setup happens, and only
    /// [`Worker::create`] / [`Worker::spawn`] (and [`Worker::retry_setup`],
    /// re-running a failed one) call it. **Do not** copy files or run
    /// `on_create` anywhere else.
    ///
    /// Each is a recorded [`Worker::phase`]. The hook's stdout and stderr go
    /// to the worker's `setup.log`; a failure quotes the end of it and says
    /// where it is.
    fn setup(&self, repo: &RepoCtx, on_create: OnCreate) -> Result<(), WorkerError> {
        let wt_path: &Path = &self.path;
        let config = &repo.config.worktree;

        self.phase(SetupPhase::CopyFiles, None, || {
            for file in config.copy_paths() {
                let src = repo.paths.repo_root.join(&file);
                let dst = wt_path.join(&file);
                if src.exists() {
                    if let Some(parent) = dst.parent() {
                        std::fs::create_dir_all(parent).map_err(SetupError::from)?;
                    }
                    std::fs::copy(&src, &dst).map_err(SetupError::from)?;
                }
            }
            Ok(())
        })?;

        let log = &self.setup_log;
        let hook = match on_create {
            OnCreate::Run => config.on_create(),
            OnCreate::Skip => None,
        };
        let Some(hook) = hook else {
            // So a log from an earlier setup of this branch can't be
            // mistaken for this one's.
            let _ = log.remove();
            return Ok(());
        };

        self.phase(SetupPhase::OnCreate, Some(log), || {
            let label = hook.label().to_string();
            let output = log.create().map_err(SetupError::from)?;
            let (outcome, exit_code, timed_out) = match hook.in_dir(wt_path).output_to(output).run()
            {
                Ok(output) if output.success() => return Ok(()),
                Ok(output) => match output.status.code() {
                    Some(code) => (format!("exited with status {code}"), Some(code), false),
                    None => ("was killed by a signal".to_string(), None, false),
                },
                Err(ExecError::TimedOut { timeout, .. }) => (
                    format!("timed out after {timeout} and was killed"),
                    None,
                    true,
                ),
                Err(e) => return Err(SetupError::from(e).into()),
            };
            Err(SetupError::Hook {
                label,
                outcome,
                excerpt: log.excerpt(SETUP_EXCERPT_LINES),
                exit_code,
                timed_out,
                tail: log.quoted_tail(SETUP_EXCERPT_LINES),
            }
            .into())
        })
    }

    /// Relaunch the repo's agent in this worker's worktree.
    pub fn resume(&self, repo: &RepoCtx, prompt: Prompt, mux: &dyn Mux) -> Result<(), WorkerError> {
        let _ = self.event_log.append(&Event::now(EventKind::Resume));

        mux.create_window(&self.branch, &self.path)?;
        let cmd = repo.config.agent.agent().resume(prompt)?;
        mux.send_keys(&self.branch, &[&cmd, "Enter"])?;
        Ok(())
    }

    /// Relaunch the agent if its window is gone — after a crash, or when the
    /// daemon restarts a worker. `false` if the window is still there.
    pub fn recover(&self, repo: &RepoCtx, mux: &dyn Mux) -> Result<bool, WorkerError> {
        if self.has_mux_window(mux) {
            return Ok(false);
        }
        let prompt = resume_task("You were interrupted. Resume your previous task.");
        self.resume(repo, prompt, mux)?;
        Ok(true)
    }

    pub fn nudge(&self, prompt: Prompt, mux: &dyn Mux) -> Result<(), WorkerError> {
        let nudge_type_key = prompt.name().to_string();
        let message = prompt.render()?;

        mux.send_message(&self.branch, &message)?;

        let _ = self.event_log.append(&Event::now(EventKind::Nudge {
            nudge_type: nudge_type_key,
            message: message.clone(),
        }));

        Ok(())
    }

    pub fn kill(&self, mux: &dyn Mux) -> Result<(), WorkerError> {
        mux.kill_window(&self.branch)?;
        Ok(())
    }

    pub fn attach(&self, mux: &dyn Mux) -> Result<(), WorkerError> {
        if !mux.window_exists(&self.branch) {
            if self.path.exists() {
                if let Some(status) = self.status() {
                    match status {
                        WorkerStatus::Initializing => {
                            return Err(WorkerError::Initializing(self.branch.to_string()));
                        }
                        WorkerStatus::Failed => {
                            let reason = self.fail_reason().unwrap_or_else(|| "unknown".into());
                            return Err(WorkerError::SetupFailed {
                                branch: self.branch.to_string(),
                                reason,
                            });
                        }
                        _ => {}
                    }
                }
            }
            return Err(WorkerError::NotFound(self.branch.to_string()));
        }
        mux.attach_window(&self.branch)?;
        Ok(())
    }

    pub fn is_orphaned(&self, mux: &dyn Mux) -> bool {
        if self.has_mux_window(mux) {
            return false;
        }
        match self.status() {
            Some(s) => {
                !s.is_terminal() && s != WorkerStatus::Initializing && s != WorkerStatus::Created
            }
            None => false,
        }
    }
}
