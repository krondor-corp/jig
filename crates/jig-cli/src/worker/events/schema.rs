//! Typed event schema for the worker event log.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use jig_core::issues::issue::IssueRef;

/// A single event in the worker event log.
///
/// Serializes as `{"ts":..., "type":"spawn", "branch":"feat/foo", ...}`.
pub type Event = jig_core::Event<EventKind>;

/// The discriminated payload of an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    Create {
        branch: String,
    },
    Initializing {
        branch: String,
        base: String,
        #[serde(default, skip_serializing_if = "is_false")]
        auto: bool,
        /// The issue being set up for, so a retry after a failed setup can
        /// rebuild the agent's task — the `Spawn` event that would otherwise
        /// carry it is only written once setup succeeds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        issue: Option<IssueRef>,
    },
    Spawn {
        branch: String,
        repo: String,
        issue: IssueRef,
    },
    Resume,
    Commit {
        sha: String,
        repo: String,
    },
    Push {
        sha: String,
        repo: String,
    },
    PrOpened {
        pr_url: String,
        pr_number: String,
    },
    Nudge {
        nudge_type: String,
        message: String,
    },
    Stop,
    Notification,
    ToolUseStart,
    ToolUseEnd,
    CiStatus,
    PrCiStatus {
        passed: bool,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        failures: Vec<String>,
    },
    PrConflict {
        has_conflict: bool,
    },
    PrReviewFeedback {
        comment_count: u32,
        changes_requested: u32,
    },
    PrCommitLint {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        bad_commits: Vec<String>,
    },
    PrMerged {
        pr_url: String,
    },
    PrClosed {
        pr_url: String,
    },
    Terminal {
        terminal: TerminalKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// A setup phase began. `log_path` is where its output goes, for the
    /// phases that have any (`on_create`).
    SetupStarted {
        phase: SetupPhase,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        log_path: Option<PathBuf>,
    },
    /// A setup phase ended. On failure `exit_code` is the hook's status —
    /// absent when it was killed, timed out (`timed_out`), or the phase
    /// isn't a process at all — and `tail` the end of its output, or the
    /// error, one line per entry.
    SetupFinished {
        phase: SetupPhase,
        ok: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
        #[serde(default, skip_serializing_if = "is_false")]
        timed_out: bool,
        duration_ms: u64,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tail: Vec<String>,
    },
    /// The daemon sent a notification of `kind` (a
    /// [`NotificationEvent::kind`](crate::notify::NotificationEvent::kind))
    /// for this worker. Recorded so a restarted daemon — whose memory of the
    /// previous tick is empty — doesn't send it again.
    Notified {
        kind: String,
    },
    /// A control verb someone ran against this worker — `jig kill`,
    /// `jig nudge`, … — whether from the CLI or over the daemon socket.
    /// A record only: it never changes the worker's status.
    Command {
        command: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
}

fn is_false(v: &bool) -> bool {
    !v
}

/// Terminal state markers — sticky, once set they can't be overridden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalKind {
    Failed,
    Merged,
    Approved,
    Archived,
}

/// The steps of bringing a worker up, in order. [`Worker::create`] stops
/// after `OnCreate`; [`Worker::spawn`] goes on to launch the agent.
///
/// [`Worker::create`]: crate::worker::Worker::create
/// [`Worker::spawn`]: crate::worker::Worker::spawn
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupPhase {
    Worktree,
    CopyFiles,
    OnCreate,
    AgentLaunch,
}

impl SetupPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Worktree => "worktree",
            Self::CopyFiles => "copy_files",
            Self::OnCreate => "on_create",
            Self::AgentLaunch => "agent_launch",
        }
    }
}

impl std::fmt::Display for SetupPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Discriminant-only enum for pattern matching (no payload).
/// Used by the reducer and derive modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Create,
    Initializing,
    Spawn,
    Resume,
    ToolUseStart,
    ToolUseEnd,
    Commit,
    Push,
    PrOpened,
    Notification,
    Stop,
    Nudge,
    CiStatus,
    PrCiStatus,
    PrConflict,
    PrReviewFeedback,
    PrCommitLint,
    PrMerged,
    PrClosed,
    Terminal,
    SetupStarted,
    SetupFinished,
    Notified,
    Command,
}

impl EventKind {
    pub fn event_type(&self) -> EventType {
        match self {
            EventKind::Create { .. } => EventType::Create,
            EventKind::Initializing { .. } => EventType::Initializing,
            EventKind::Spawn { .. } => EventType::Spawn,
            EventKind::Resume => EventType::Resume,
            EventKind::ToolUseStart => EventType::ToolUseStart,
            EventKind::ToolUseEnd => EventType::ToolUseEnd,
            EventKind::Commit { .. } => EventType::Commit,
            EventKind::Push { .. } => EventType::Push,
            EventKind::PrOpened { .. } => EventType::PrOpened,
            EventKind::Notification => EventType::Notification,
            EventKind::Stop => EventType::Stop,
            EventKind::Nudge { .. } => EventType::Nudge,
            EventKind::CiStatus => EventType::CiStatus,
            EventKind::PrCiStatus { .. } => EventType::PrCiStatus,
            EventKind::PrConflict { .. } => EventType::PrConflict,
            EventKind::PrReviewFeedback { .. } => EventType::PrReviewFeedback,
            EventKind::PrCommitLint { .. } => EventType::PrCommitLint,
            EventKind::PrMerged { .. } => EventType::PrMerged,
            EventKind::PrClosed { .. } => EventType::PrClosed,
            EventKind::Terminal { .. } => EventType::Terminal,
            EventKind::SetupStarted { .. } => EventType::SetupStarted,
            EventKind::SetupFinished { .. } => EventType::SetupFinished,
            EventKind::Notified { .. } => EventType::Notified,
            EventKind::Command { .. } => EventType::Command,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_serializes_flat() {
        let event = Event::now(EventKind::Spawn {
            branch: "feat/foo".into(),
            repo: "myrepo".into(),
            issue: IssueRef::new("JIG-5"),
        });
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["ts"].is_i64());
        assert_eq!(parsed["type"], "spawn");
        assert_eq!(parsed["branch"], "feat/foo");
        assert_eq!(parsed["issue"], "JIG-5");
        assert!(parsed.get("kind").is_none());
    }

    #[test]
    fn pr_opened_roundtrip() {
        let original = Event::now(EventKind::PrOpened {
            pr_url: "https://github.com/pr/1".into(),
            pr_number: "42".into(),
        });
        let json = serde_json::to_string(&original).unwrap();
        let restored: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.kind.event_type(), EventType::PrOpened);
        if let EventKind::PrOpened { pr_url, pr_number } = &restored.kind {
            assert_eq!(pr_url, "https://github.com/pr/1");
            assert_eq!(pr_number, "42");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn terminal_serialization() {
        let event = Event::now(EventKind::Terminal {
            terminal: TerminalKind::Failed,
            reason: Some("on-create hook failed".into()),
        });
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "terminal");
        assert_eq!(parsed["terminal"], "failed");
        assert_eq!(parsed["reason"], "on-create hook failed");
    }

    #[test]
    fn unit_variant_serialization() {
        let event = Event::now(EventKind::Stop);
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "stop");
    }

    #[test]
    fn setup_finished_serializes_flat_and_omits_defaults() {
        let event = Event::now(EventKind::SetupFinished {
            phase: SetupPhase::CopyFiles,
            ok: true,
            exit_code: None,
            timed_out: false,
            duration_ms: 12,
            tail: vec![],
        });
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "setup_finished");
        assert_eq!(parsed["phase"], "copy_files");
        assert_eq!(parsed["ok"], true);
        assert!(parsed.get("exit_code").is_none());
        assert!(parsed.get("timed_out").is_none());
        assert!(parsed.get("tail").is_none());
    }

    #[test]
    fn setup_failure_roundtrips() {
        let original = Event::now(EventKind::SetupFinished {
            phase: SetupPhase::OnCreate,
            ok: false,
            exit_code: Some(7),
            timed_out: false,
            duration_ms: 1500,
            tail: vec!["ERR_PNPM_OUTDATED_LOCKFILE".into()],
        });
        let json = serde_json::to_string(&original).unwrap();
        let restored: Event = serde_json::from_str(&json).unwrap();
        let EventKind::SetupFinished {
            phase,
            exit_code,
            tail,
            ..
        } = restored.kind
        else {
            panic!("wrong variant");
        };
        assert_eq!(phase, SetupPhase::OnCreate);
        assert_eq!(exit_code, Some(7));
        assert_eq!(tail, vec!["ERR_PNPM_OUTDATED_LOCKFILE".to_string()]);
    }

    #[test]
    fn initializing_only_has_branch_and_base() {
        let event = Event::now(EventKind::Initializing {
            branch: "feat/foo".into(),
            base: "main".into(),
            auto: false,
            issue: None,
        });
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.contains("issue"));
        assert!(!json.contains("repo"));
        assert!(!json.contains("auto")); // false is skipped
    }

    #[test]
    fn command_round_trips_and_omits_an_empty_detail() {
        let event = Event::now(EventKind::Command {
            command: "kill".into(),
            detail: None,
        });
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.contains("detail"));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "command");
        assert_eq!(parsed["command"], "kill");

        let restored: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.kind.event_type(), EventType::Command);
    }
}
