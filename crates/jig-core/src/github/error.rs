use std::io;

use crate::exec::ExecError;

pub type Result<T> = std::result::Result<T, GitHubError>;

#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error("gh CLI failed: {0}")]
    Cli(String),
    #[error("failed to parse GitHub response: {msg}")]
    Parse { msg: String, body: String },
    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    /// A `gh` that could not be run, or that outran its deadline. Forwarded
    /// as-is: `exec` already knows which command and how long it was given.
    #[error(transparent)]
    Exec(#[from] ExecError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
