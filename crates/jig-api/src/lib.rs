//! The web view's API: the JSON the daemon serves and the web app reads.
//!
//! The daemon (`jig_cli::daemon::web`) serializes these; the Yew app
//! (`crates/jig-cli/web`) deserializes them. Both compile this crate, so a
//! field can't change on one side only. Route paths are the constants
//! below. Use them on both sides, never a literal.
//!
//! Everything a row shows is decided by the daemon, from the same columns
//! `jig ps` prints. The app lays rows out and adds links and buttons; it
//! never re-derives a state, so the page and the terminal can't drift.

use serde::{Deserialize, Serialize};

/// `GET` → [`Session`].
pub const SESSION: &str = "/api/v0/session";
/// `GET` → [`Status`].
pub const STATUS: &str = "/api/v0/status";
/// `POST` [`CommandRequest`] → [`CommandOutcome`], or [`ApiError`].
pub const COMMAND: &str = "/api/v0/command";
/// `GET ?worker=<repo/branch>` → the end of that worker's `setup.log`, as
/// plain text.
pub const SETUP_LOG: &str = "/api/v0/setup-log";

/// The header a command carries its [`Session::token`] in.
pub const TOKEN_HEADER: &str = "x-jig-token";

/// What the app needs before it can send commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Minted when the daemon starts; required on every command.
    pub token: String,
}

/// Everything the daemon knows right now: the frame `jig ps` renders.
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
    /// `repo/branch`: what commands and the setup log are addressed by.
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
    /// The state cell's text, e.g. "on_create failed (exit 1)".
    pub state: String,
    /// The last lines of output.
    pub tail: Vec<String>,
    /// Whether [`SETUP_LOG`] has the whole log for this worker.
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

/// A control verb: the same five the CLI has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verb", rename_all = "kebab-case")]
pub enum Command {
    Kill,
    Nudge { text: String },
    Prune,
    RetrySetup,
    Model { model: String },
}

impl Command {
    /// The CLI verb, e.g. `retry-setup`.
    pub fn name(&self) -> &'static str {
        match self {
            Command::Kill => "kill",
            Command::Nudge { .. } => "nudge",
            Command::Prune => "prune",
            Command::RetrySetup => "retry-setup",
            Command::Model { .. } => "model",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRequest {
    /// `repo/branch`, as in [`Worker::key`].
    pub worker: String,
    pub command: Command,
}

/// A command the daemon ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandOutcome {
    pub message: String,
}

/// Why a request was refused: the body of every non-2xx JSON response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_request_reads_from_json() {
        let req: CommandRequest = serde_json::from_str(
            r#"{"worker":"jig/feat/x","command":{"verb":"nudge","text":"rebase"}}"#,
        )
        .unwrap();
        assert_eq!(req.worker, "jig/feat/x");
        assert_eq!(
            req.command,
            Command::Nudge {
                text: "rebase".into()
            }
        );

        let req: CommandRequest =
            serde_json::from_str(r#"{"worker":"jig/x","command":{"verb":"retry-setup"}}"#).unwrap();
        assert_eq!(req.command.name(), "retry-setup");
    }

    #[test]
    fn every_verb_round_trips() {
        for command in [
            Command::Kill,
            Command::Nudge { text: "hi".into() },
            Command::Prune,
            Command::RetrySetup,
            Command::Model {
                model: "opus".into(),
            },
        ] {
            let json = serde_json::to_string(&command).unwrap();
            assert!(json.contains(command.name()), "{json}");
            assert_eq!(serde_json::from_str::<Command>(&json).unwrap(), command);
        }
    }

    #[test]
    fn tones_are_lowercase_names() {
        assert_eq!(serde_json::to_string(&Tone::Dim).unwrap(), r#""dim""#);
    }
}
