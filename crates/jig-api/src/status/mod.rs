//! `/api/v0/status`: the work view, as `jig ps` would print it.
//!
//! Everything a row shows is decided by the daemon, from the same columns
//! `jig ps` prints. The app lays rows out and adds links and buttons; it
//! never re-derives a state, so the page and the terminal can't drift.

pub mod get;

pub use get::StatusRequest;

use serde::{Deserialize, Serialize};

/// The frame `jig ps` renders.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Status {
    /// This machine's fleet name, when it has one.
    pub machine: Option<String>,
    pub pid: u32,
    pub version: String,
    /// Time until the next poll, already formatted ("12s").
    pub next_poll: String,
    /// Column titles for [`Worker::cells`].
    pub headers: Vec<String>,
    /// Workers grouped by repo, in the order repos are first seen.
    pub repos: Vec<Repo>,
    pub triages: Vec<Triage>,
    /// Workers the spawn actor is bringing up right now.
    pub spawning: Vec<String>,
}

impl Status {
    pub fn workers(&self) -> impl Iterator<Item = &Worker> {
        self.repos.iter().flat_map(|r| r.workers.iter())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repo {
    pub name: String,
    pub workers: Vec<Worker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Worker {
    /// `repo/branch`: what the `workers` requests are addressed by.
    pub key: String,
    /// The full branch name. The first cell may be truncated.
    pub name: String,
    /// The agent's live state, when the mux reports one.
    pub agent: Option<String>,
    /// The PR, when there is one. Always an `http(s)` URL.
    pub pr_url: Option<String>,
    /// One per [`Status::headers`], as `jig ps` shows them.
    pub cells: Vec<Cell>,
    /// Set when `on_create` failed.
    pub failed_setup: Option<FailedSetup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub text: String,
    pub tone: Tone,
    /// Right-aligned (the numeric columns).
    pub right: bool,
}

/// A cell's color: the `jig ps` palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    #[default]
    Plain,
    Green,
    Red,
    Yellow,
    Blue,
    Cyan,
    Magenta,
    Dim,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedSetup {
    /// The state cell's text, e.g. "failed: on_create (exit 1)".
    pub state: String,
    /// The last lines of output.
    pub tail: Vec<String>,
    /// Whether [`crate::SetupLogRequest`] has the whole log.
    pub has_log: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Triage {
    pub issue: String,
    pub model: String,
    /// Already formatted ("3m").
    pub elapsed: String,
    pub repo: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tones_are_lowercase_names() {
        assert_eq!(serde_json::to_string(&Tone::Dim).unwrap(), r#""dim""#);
    }
}
