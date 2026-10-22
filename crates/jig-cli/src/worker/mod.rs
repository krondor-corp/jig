//! Worker — the single abstraction for a Claude Code session.
//!
//! A Worker owns its identity and a [`WorktreeRef`] pointing at its
//! git worktree on disk.  The full [`Worktree`] (wrapping a git2 repo
//! handle) is resolved on demand — we never serialize what we can derive.

pub mod events;
mod status;

pub use status::{MuxStatus, WorkerStatus};

use std::path::Path;

use uuid::Uuid;

use crate::context::log::{LogFile, Logs};
use crate::context::{worktree_path, AppPaths, RepoCtx};
use events::{Event, EventKind, TerminalKind, WorkerState};
use jig_core::agents::Agent;
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

/// Whether [`Worker::setup`] runs the repo's `on_create` hook.
///
/// Skipping it (`jig create --no-hooks`) still copies the configured files:
/// those are what make the worktree usable, not a user extension point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnCreate {
    Run,
    Skip,
}

/// Why [`Worker::setup`] couldn't make a fresh worktree ready to work in.
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    /// The hook ran and didn't succeed — exited non-zero, was killed, or
    /// outran its deadline. `excerpt` is [`LogFile::excerpt`] of `setup.log`.
    #[error("{label} {outcome}:\n{excerpt}")]
    Hook {
        label: String,
        outcome: String,
        excerpt: String,
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
/// Everything is derived at runtime via the [`Worktree`] handle.
#[derive(Debug, Clone)]
pub struct Worker {
    pub(crate) id: Uuid,
    pub(crate) branch: Branch,
    pub(crate) path: WorktreeRef,
    pub(crate) issue_ref: Option<IssueRef>,
}

impl From<&Worktree> for Worker {
    fn from(wt: &Worktree) -> Self {
        Self {
            id: Uuid::new_v4(),
            branch: wt.branch_name(),
            path: wt.as_ref(),
            issue_ref: None,
        }
    }
}

impl Worker {
    pub fn from_branch(repo_root: &Path, branch: Branch) -> Self {
        let worktree_path = repo_root.join(crate::context::JIG_DIR).join(&*branch);
        Self {
            id: Uuid::nil(),
            branch,
            path: WorktreeRef::new(worktree_path),
            issue_ref: None,
        }
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
    pub fn event_log(&self, paths: &AppPaths) -> events::EventLog {
        Logs::new(paths).worker_events(&self.repo_name(), &self.branch)
    }

    /// This worker's `on_create` output, from its last [`Worker::setup`].
    pub fn setup_log(&self, paths: &AppPaths) -> LogFile {
        Logs::new(paths).setup(&self.repo_name(), &self.branch)
    }

    pub fn status(&self, paths: &AppPaths) -> Option<WorkerStatus> {
        let log = self.event_log(paths);
        if !log.exists() {
            return None;
        }
        let state: WorkerState = log.reduce().ok()?;
        Some(state.status)
    }

    pub fn fail_reason(&self, paths: &AppPaths) -> Option<String> {
        let events = self.event_log(paths).read_all().ok()?;
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

    pub fn unregister(&self, paths: &AppPaths) -> Result<(), WorkerError> {
        // First, so removing the event log can also remove the now-empty
        // directory.
        let _ = self.setup_log(paths).remove();
        let _ = self.event_log(paths).remove();
        Ok(())
    }

    pub fn discover(repo: &Repo) -> Vec<Self> {
        let mut workers: Vec<Self> = repo
            .list_worktrees()
            .unwrap_or_default()
            .iter()
            .map(Self::from)
            .collect();
        workers.sort_by(|a, b| a.branch.cmp(&b.branch));
        workers
    }

    pub fn repo_name(&self) -> String {
        self.repo_root()
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// The main checkout this worker's worktree hangs off: the parent of the
    /// `.jig/` directory above it.
    fn repo_root(&self) -> Option<&Path> {
        let path: &Path = &self.path;
        path.ancestors()
            .find(|a| a.file_name().is_some_and(|n| n == crate::context::JIG_DIR))
            .and_then(Path::parent)
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

    /// Create the worktree, set it up, and start the repo's agent in it.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        paths: &AppPaths,
        repo: &RepoCtx,
        branch: &Branch,
        base: &Branch,
        prompt: Prompt,
        auto: bool,
        issue_ref: Option<IssueRef>,
        mux: &dyn Mux,
    ) -> Result<Self, WorkerError> {
        let git = Repo::open(&repo.paths.repo_root)?;
        let worker = Self {
            id: Uuid::new_v4(),
            branch: branch.clone(),
            path: WorktreeRef::new(worktree_path(&repo.paths.repo_root, branch)),
            issue_ref: issue_ref.clone(),
        };
        let branch_name = branch.to_string();

        let event_log = worker.event_log(paths);
        event_log.reset()?;

        let _ = event_log.append(&Event::now(EventKind::Initializing {
            branch: branch_name.clone(),
            base: base.to_string(),
            auto,
        }));

        // Recorded so `jig attach`, `jig list` and the daemon's
        // needs_intervention alert can say why.
        let failed = |e: WorkerError| {
            let _ = event_log.append(&Event::now(EventKind::Terminal {
                terminal: TerminalKind::Failed,
                reason: Some(e.to_string()),
            }));
            e
        };

        Worktree::create(&git, branch, base).map_err(|e| failed(e.into()))?;
        worker
            .setup(paths, repo, OnCreate::Run)
            .map_err(|e| failed(e.into()))?;

        let issue = issue_ref.unwrap_or_else(|| IssueRef::new(branch_name.clone()));
        let _ = event_log.append(&Event::now(EventKind::Spawn {
            branch: branch_name,
            repo: worker.repo_name(),
            issue,
        }));

        mux.create_window(&worker.branch, &worker.path)?;
        let cmd = repo.config.agent.agent().spawn(prompt)?;
        mux.send_keys(&worker.branch, &[&cmd, "Enter"])?;

        Ok(worker)
    }

    /// Make a freshly created worktree ready to work in: copy the configured
    /// files over from the main checkout, then run the `on_create` hook.
    ///
    /// NOTE(canonical): this is the one place worktree setup happens — `jig
    /// spawn`, the daemon's auto-spawn and `jig create` all come through here.
    /// **Do not** copy files or run `on_create` anywhere else.
    ///
    /// The hook's stdout and stderr go to the worker's `setup.log`; a failure
    /// quotes the end of it and says where it is.
    pub fn setup(
        &self,
        paths: &AppPaths,
        repo: &RepoCtx,
        on_create: OnCreate,
    ) -> Result<(), SetupError> {
        let wt_path: &Path = &self.path;
        let config = &repo.config.worktree;

        for file in config.copy_paths() {
            let src = repo.paths.repo_root.join(&file);
            let dst = wt_path.join(&file);
            if src.exists() {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&src, &dst)?;
            }
        }

        let log = self.setup_log(paths);
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

        let label = hook.label().to_string();
        let outcome = match hook.in_dir(wt_path).output_to(log.create()?).run() {
            Ok(output) if output.success() => return Ok(()),
            Ok(output) => match output.status.code() {
                Some(code) => format!("exited with status {code}"),
                None => "was killed by a signal".to_string(),
            },
            Err(ExecError::TimedOut { timeout, .. }) => {
                format!("timed out after {timeout} and was killed")
            }
            Err(e) => return Err(e.into()),
        };
        Err(SetupError::Hook {
            label,
            outcome,
            excerpt: log.excerpt(SETUP_EXCERPT_LINES),
        })
    }

    pub fn resume(
        paths: &AppPaths,
        wt: &Worktree,
        agent: &Agent,
        prompt: Prompt,
        mux: &dyn Mux,
    ) -> Result<Self, WorkerError> {
        let worker = Self {
            id: Uuid::new_v4(),
            branch: wt.branch_name(),
            path: wt.as_ref(),
            issue_ref: None,
        };

        let _ = worker
            .event_log(paths)
            .append(&Event::now(EventKind::Resume));

        mux.create_window(&worker.branch, &worker.path)?;
        let cmd = agent.resume(prompt)?;
        mux.send_keys(&worker.branch, &[&cmd, "Enter"])?;

        Ok(worker)
    }

    pub fn nudge(
        &self,
        paths: &AppPaths,
        prompt: Prompt,
        mux: &dyn Mux,
    ) -> Result<(), WorkerError> {
        let nudge_type_key = prompt.name().to_string();
        let message = prompt.render()?;

        mux.send_message(&self.branch, &message)?;

        let _ = self.event_log(paths).append(&Event::now(EventKind::Nudge {
            nudge_type: nudge_type_key,
            message: message.clone(),
        }));

        Ok(())
    }

    pub fn kill(&self, mux: &dyn Mux) -> Result<(), WorkerError> {
        mux.kill_window(&self.branch)?;
        Ok(())
    }

    pub fn attach(&self, paths: &AppPaths, mux: &dyn Mux) -> Result<(), WorkerError> {
        if !mux.window_exists(&self.branch) {
            if self.path.exists() {
                if let Some(status) = self.status(paths) {
                    match status {
                        WorkerStatus::Initializing => {
                            return Err(WorkerError::Initializing(self.branch.to_string()));
                        }
                        WorkerStatus::Failed => {
                            let reason =
                                self.fail_reason(paths).unwrap_or_else(|| "unknown".into());
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

    pub fn is_orphaned(&self, paths: &AppPaths, mux: &dyn Mux) -> bool {
        if self.has_mux_window(mux) {
            return false;
        }
        match self.status(paths) {
            Some(s) => {
                !s.is_terminal() && s != WorkerStatus::Initializing && s != WorkerStatus::Created
            }
            None => false,
        }
    }
}
