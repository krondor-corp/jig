//! Git worktree — a [`Repo`] that has been validated as a linked worktree.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::branch::Branch;
use super::diff::{Diff, Stats as DiffStats};
use super::error::{GitError, Result};
use super::Repo;

/// Lightweight serializable reference to a worktree on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorktreeRef(PathBuf);

impl WorktreeRef {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    // TODO (cleanup): more foolproof check,
    //  such as seeing if we have a link to our root
    //  .git or something
    pub fn exists(&self) -> bool {
        self.0.exists()
    }

    pub fn open(&self) -> Result<Worktree> {
        Worktree::open(&self.0)
    }
}

impl std::ops::Deref for WorktreeRef {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

impl std::fmt::Display for WorktreeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display().fmt(f)
    }
}

/// How often [`Hook`] checks whether the command has exited.
const HOOK_POLL: Duration = Duration::from_millis(100);

/// A repo-configured command run on a new worktree, under a deadline.
///
/// The deadline is the point: the spawn actor handles one request at a time
/// and drops the rest while one is in flight, so an `on_create` that never
/// returns — a `pnpm install` waiting on a dead network, say — stops
/// auto-spawn for good, with nothing in the logs to say why.
pub struct Hook {
    pub command: Command,
    pub timeout: Duration,
}

impl Hook {
    /// Run to completion in `dir`, or kill it once `timeout` has passed.
    ///
    /// stdout is discarded (it always was); stderr is drained on a thread so
    /// a chatty hook cannot fill the pipe and deadlock against our polling.
    fn run(mut self, dir: &Path) -> Result<()> {
        let mut child = self
            .command
            .current_dir(dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut pipe = child.stderr.take();
        let draining = std::thread::spawn(move || {
            let mut buf = String::new();
            if let Some(pipe) = pipe.as_mut() {
                let _ = pipe.read_to_string(&mut buf);
            }
            buf
        });

        let deadline = Instant::now() + self.timeout;
        let status = loop {
            match child.try_wait()? {
                Some(status) => break status,
                None if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(GitError::HookTimedOut(self.timeout));
                }
                None => std::thread::sleep(HOOK_POLL),
            }
        };

        let stderr = draining.join().unwrap_or_default();
        if !status.success() {
            // A hook that fails silently used to produce a bare
            // "hook failed:" with nothing after the colon.
            return Err(GitError::HookFailed(match stderr.trim() {
                "" => match status.code() {
                    Some(code) => format!("exited with status {code}, no output"),
                    None => "killed by a signal, no output".to_string(),
                },
                message => message.to_string(),
            }));
        }
        Ok(())
    }
}

/// A validated jig-managed worktree. Guarantees the underlying repo is a
/// linked worktree (not the main clone) living under the repo's `.jig/`.
///
/// Other linked worktrees (Claude Code's `.claude/worktrees/`, hand-made
/// ones) fail to open with [`GitError::NotJigWorktree`], so everything built
/// on `Worktree` — discovery, the daemon, hooks — skips them.
pub struct Worktree {
    repo: Repo,
    /// Path relative to `.jig/` — the worker's identity.
    name: Branch,
}

impl Worktree {
    /// Discover the worktree containing the current working directory.
    pub fn current() -> Result<Self> {
        let repo = Repo::discover()?;
        Self::validate(repo)
    }

    /// Open a worktree at `path`. Errors if it's not a linked worktree.
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repo::open(path)?;
        Self::validate(repo)
    }

    fn validate(repo: Repo) -> Result<Self> {
        if !repo.inner().is_worktree() {
            return Err(GitError::NotInWorktree);
        }
        let path = repo.root()?;
        let name = match path.strip_prefix(repo.worktrees_path()) {
            Ok(rel) => Branch::new(rel.to_string_lossy().as_ref()),
            Err(_) => return Err(GitError::NotJigWorktree(path)),
        };
        Ok(Self { repo, name })
    }

    /// Create a git worktree on disk: ensures `.jig` is git-excluded,
    /// creates the worktree, copies files, and runs the on-create hook.
    pub fn create(
        repo: &Repo,
        branch: &Branch,
        base: &Branch,
        copy_files: &[PathBuf],
        on_create: Option<Hook>,
    ) -> Result<Self> {
        crate::git::ensure_excluded(&repo.common_dir(), super::WORKTREES_DIR)?;
        let path = repo.create_worktree(branch, base)?;
        let wt = Self::open(&path)?;

        let repo_root = repo.clone_path();
        for file in copy_files {
            let src = repo_root.join(file);
            let dst = wt.path().join(file);
            if src.exists() {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&src, &dst)?;
            }
        }

        if let Some(hook) = on_create {
            hook.run(&wt.path())?;
        }

        Ok(wt)
    }

    pub fn as_ref(&self) -> WorktreeRef {
        WorktreeRef::new(self.path())
    }

    // ── Derived state ──

    pub fn path(&self) -> PathBuf {
        self.repo.root().expect("worktrees always have a workdir")
    }

    /// The worker name: this worktree's path relative to `.jig/`. Not
    /// necessarily the checked-out git branch — see [`Worktree::branch`].
    pub fn branch_name(&self) -> Branch {
        self.name.clone()
    }

    pub fn repo_root(&self) -> PathBuf {
        self.repo.clone_path()
    }

    pub fn head_sha(&self) -> Result<String> {
        Ok(self.repo.head_oid()?.to_string())
    }

    pub fn branch(&self) -> Result<Branch> {
        self.repo.current_branch()
    }

    /// The checked-out branch, or `None` when HEAD is detached (or unborn)
    /// and there is no branch to name.
    ///
    /// This is what a forge knows the worker by. [`Worktree::branch_name`]
    /// is not: the two start out identical and drift apart the moment
    /// anyone renames the branch.
    pub fn checked_out_branch(&self) -> Option<Branch> {
        if self.repo.inner().head_detached().unwrap_or(true) {
            return None;
        }
        self.branch().ok()
    }

    pub fn base_branch(&self) -> Result<Branch> {
        self.repo.base_branch()
    }

    pub fn repo_name(&self) -> String {
        self.repo_root()
            .file_name()
            .expect("repo root must have a directory name")
            .to_string_lossy()
            .to_string()
    }

    // ── Operations ──

    /// Remove this worktree. Prunes empty parent directories up to (but
    /// not including) the directory named `stop_at`.
    pub fn remove(&self, force: bool) -> Result<()> {
        let worktrees_path = self.repo.worktrees_path();
        self.repo.remove(force)?;
        self.cleanup_empty_parents(&worktrees_path)?;
        Ok(())
    }

    // ── Git queries ──

    pub fn has_uncommitted_changes(&self) -> Result<bool> {
        self.repo.has_uncommitted_changes()
    }

    pub fn commits_ahead(&self) -> Result<Vec<String>> {
        self.repo.commits_ahead(&self.base_branch()?)
    }

    pub fn diff(&self) -> Result<Diff<'_>> {
        self.repo.diff(&self.base_branch()?)
    }

    pub fn diff_stats(&self) -> Result<DiffStats> {
        self.diff()?.stats()
    }

    pub fn diff_stat(&self) -> Result<String> {
        self.diff()?.stat_string()
    }

    fn cleanup_empty_parents(&self, stop_at: &Path) -> Result<()> {
        let path = self.path();
        let mut parent = path.parent();
        while let Some(p) = parent {
            if p == stop_at {
                break;
            }
            if p.read_dir()?.next().is_some() {
                break;
            }
            std::fs::remove_dir(p)?;
            parent = p.parent();
        }
        Ok(())
    }
}
