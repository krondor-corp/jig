use std::path::PathBuf;

use crate::exec::ExecError;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("not in a git repository")]
    NotInRepo,

    #[error("not in a worktree")]
    NotInWorktree,

    /// A linked worktree that jig didn't create — e.g. one made by Claude
    /// Code under `.claude/worktrees/`. jig leaves these alone.
    #[error("{} is not a jig worktree (not under .jig/)", .0.display())]
    NotJigWorktree(PathBuf),

    #[error("branch '{0}' not found")]
    BranchNotFound(String),

    #[error("worktree '{0}' already exists")]
    WorktreeExists(String),

    #[error("worktree '{0}' not found")]
    WorktreeNotFound(String),

    #[error("uncommitted changes")]
    UncommittedChanges,

    #[error("merge conflict with '{0}'")]
    MergeConflict(String),

    #[error("invalid path: {0}")]
    InvalidPath(PathBuf),

    /// `git` itself said no — its own message, which is more use than
    /// libgit2's error codes ever were.
    #[error("{0}")]
    Cli(String),

    #[error("fetch failed: {0}")]
    FetchFailed(String),

    #[error("push failed: {0}")]
    PushFailed(String),

    #[error("hook failed: {0}")]
    HookFailed(String),

    /// A command that could not be run, or that outran its deadline.
    /// Forwarded as-is: `exec` already names it and says how long it had.
    #[error(transparent)]
    Exec(#[from] ExecError),

    #[error(transparent)]
    Git2(#[from] git2::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, GitError>;
