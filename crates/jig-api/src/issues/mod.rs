//! `/api/v0/issues/*`: a machine's issues, and spawning a worker for one.
//!
//! Issues belong to a machine: each repo's `jig.toml` names an issue
//! tracker profile whose key lives in that machine's config. So every
//! request names the `machine` (a [`crate::Machine::node`]; `None` for the
//! one serving the page) and is forwarded there, like the worker verbs.
//!
//! The list is the daemon's cache of its last fetch (planned and backlog),
//! cheap to poll; [`RefreshIssuesRequest`] asks for a fresh fetch.

pub mod get;
pub mod refresh;
pub mod spawn;

pub use get::IssuesRequest;
pub use refresh::RefreshIssuesRequest;
pub use spawn::SpawnRequest;

use serde::{Deserialize, Serialize};

/// One machine's issues, repo by repo.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Issues {
    pub repos: Vec<RepoIssues>,
    /// How long ago they were fetched ("2m"). `None` before the first fetch.
    pub fetched: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoIssues {
    /// The repo's directory name: what [`SpawnRequest::repo`] takes.
    pub repo: String,
    pub issues: Vec<Issue>,
    /// Why its issues could not be listed, when they could not.
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    /// What [`SpawnRequest::issue`] takes, e.g. `KRO-285`.
    pub id: String,
    pub title: String,
    /// `planned` or `backlog`.
    pub status: String,
    /// `urgent`, `high`, `medium` or `low`.
    pub priority: String,
    pub labels: Vec<String>,
    /// The branch a spawn would create.
    pub branch: String,
    /// The daemon spawns it on its own once it is planned.
    pub auto: bool,
    /// Has sub-issues: those are spawned, not this.
    pub parent: bool,
    /// Its branch already has a worktree on this machine.
    pub spawned: bool,
}
