pub mod herdr;
pub mod tmux;

pub use herdr::HerdrMux;
pub use tmux::TmuxMux;

use std::path::Path;

impl<M: Mux + ?Sized> Mux for Box<M> {
    fn create_window(&self, name: &str, dir: &Path) -> Result<(), MuxError> {
        (**self).create_window(name, dir)
    }
    fn window_exists(&self, name: &str) -> bool {
        (**self).window_exists(name)
    }
    fn kill_window(&self, name: &str) -> Result<(), MuxError> {
        (**self).kill_window(name)
    }
    fn kill_all(&self) -> Result<(), MuxError> {
        (**self).kill_all()
    }
    fn send_keys(&self, name: &str, keys: &[&str]) -> Result<(), MuxError> {
        (**self).send_keys(name, keys)
    }
    fn send_message(&self, name: &str, message: &str) -> Result<(), MuxError> {
        (**self).send_message(name, message)
    }
    fn is_running(&self, name: &str) -> bool {
        (**self).is_running(name)
    }
    fn attach_window(&self, name: &str) -> Result<(), MuxError> {
        (**self).attach_window(name)
    }
    fn attach(&self) -> Result<(), MuxError> {
        (**self).attach()
    }
    fn agent_state(&self, name: &str) -> Option<AgentState> {
        (**self).agent_state(name)
    }
}

/// Which multiplexer backend hosts worker terminals.
///
/// Configured globally (`mux = "herdr"` in `~/.config/jig/config.toml`) —
/// the mux is a property of the machine, not the repo. The `JIG_MUX` env
/// var overrides the configured value for one-off runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MuxKind {
    #[default]
    Tmux,
    /// Herdr workspace — daemon-owned terminals that survive disconnects
    /// and reattach over SSH, with agent lifecycle detection.
    Herdr,
}

impl std::fmt::Display for MuxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MuxKind::Tmux => "tmux",
            MuxKind::Herdr => "herdr",
        })
    }
}

impl std::str::FromStr for MuxKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tmux" => Ok(MuxKind::Tmux),
            "herdr" => Ok(MuxKind::Herdr),
            other => Err(format!(
                "unknown mux backend '{other}' (expected tmux or herdr)"
            )),
        }
    }
}

impl MuxKind {
    /// The effective backend: `JIG_MUX` env override, else `self`.
    pub fn effective(self) -> Self {
        match std::env::var("JIG_MUX") {
            Ok(v) => v.parse().unwrap_or(self),
            Err(_) => self,
        }
    }
}

/// Build the configured mux backend for a repo's worker group.
///
/// Both backends derive the group name the same way (`<prefix><repo>`), so
/// the choice only affects where the terminals live.
pub fn for_repo(kind: MuxKind, repo_name: &str) -> Box<dyn Mux> {
    for_repo_with_prefix(kind, "jig-", repo_name)
}

pub fn for_repo_with_prefix(kind: MuxKind, prefix: &str, repo_name: &str) -> Box<dyn Mux> {
    from_group_name(kind, format!("{prefix}{repo_name}"))
}

pub fn from_group_name(kind: MuxKind, name: impl Into<String>) -> Box<dyn Mux> {
    let name = name.into();
    match kind.effective() {
        MuxKind::Herdr => Box::new(HerdrMux::new(name)),
        MuxKind::Tmux => Box::new(TmuxMux::new(name)),
    }
}

pub const KNOWN_SHELLS: &[&str] = &["bash", "zsh", "fish", "sh"];

#[derive(Debug, thiserror::Error)]
pub enum MuxError {
    #[error("mux command failed: {command}: {detail}")]
    CommandFailed { command: String, detail: String },
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("mux command timed out: {command}")]
    Timeout { command: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Lifecycle state a mux backend reports for the agent in a window.
///
/// Only backends that recognize agents (herdr) report this; tmux has no
/// notion of it. Orthogonal to jig's event-derived worker status: this is
/// what the terminal looks like *right now*, not where the worker is in
/// its pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentState {
    Idle,
    Working,
    Blocked,
    Done,
    Unknown,
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AgentState::Idle => "idle",
            AgentState::Working => "working",
            AgentState::Blocked => "blocked",
            AgentState::Done => "done",
            AgentState::Unknown => "unknown",
        };
        f.write_str(s)
    }
}

/// A multiplexer backend scoped to a named group (e.g. one tmux session).
///
/// Window names are branch names — the backend manages the mapping to its
/// native hierarchy (tmux session:window, cmux workspace, etc.).
pub trait Mux: Send + Sync {
    fn create_window(&self, name: &str, dir: &Path) -> Result<(), MuxError>;
    fn window_exists(&self, name: &str) -> bool;
    fn kill_window(&self, name: &str) -> Result<(), MuxError>;
    fn kill_all(&self) -> Result<(), MuxError>;
    fn send_keys(&self, name: &str, keys: &[&str]) -> Result<(), MuxError>;
    fn send_message(&self, name: &str, message: &str) -> Result<(), MuxError>;
    fn is_running(&self, name: &str) -> bool;
    fn attach_window(&self, name: &str) -> Result<(), MuxError>;
    fn attach(&self) -> Result<(), MuxError>;

    /// The backend's own read of the agent in a window, if it has one.
    fn agent_state(&self, _name: &str) -> Option<AgentState> {
        None
    }
}
