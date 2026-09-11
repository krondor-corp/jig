use std::path::PathBuf;

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

    #[error("fetch failed: {0}")]
    FetchFailed(String),

    #[error("push failed: {0}")]
    PushFailed(String),

    #[error("hook failed: {0}")]
    HookFailed(String),

    #[error(transparent)]
    Git2(#[from] git2::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, GitError>;
