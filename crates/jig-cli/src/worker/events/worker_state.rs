//! `WorkerState` — what a worker's event log reduces to, plus the runtime
//! details the daemon fills in. The reduction itself lives in `reducer.rs`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use url::Url;

use super::SetupPhase;

use crate::context::{AppConfig, RepoEntry};
use crate::daemon::checks::PrHealth;
use crate::worker::{MuxStatus, WorkerStatus};
use jig_core::git::Branch;
use jig_core::issues::issue::IssueRef;

/// Full worker state — event-log reduction + runtime enrichment.
///
/// Core fields are set by [`Reducible::apply`]. Runtime fields (`repo`,
/// `mux_status`, `commits_ahead`, etc.) are filled in by [`Worker::tick()`].
#[derive(Debug, Clone)]
pub struct WorkerState {
    // ── Event-derived ───────────────────────────────────────────
    pub status: WorkerStatus,
    pub branch: Option<String>,
    pub commit_count: u32,
    pub last_commit_at: Option<i64>,
    /// Parsed once, here, rather than kept as text and re-parsed by every
    /// consumer. The event log still records it as a string.
    pub pr_url: Option<Url>,
    pub nudge_counts: HashMap<String, u32>,
    pub last_nudge_at: HashMap<String, i64>,
    pub issue_ref: Option<IssueRef>,
    pub started_at: Option<i64>,
    pub last_event_at: Option<i64>,
    pub review_feedback_count: u32,
    pub pr_ci_passed: Option<bool>,
    pub pr_ci_failures: Vec<String>,
    pub pr_has_conflicts: Option<bool>,
    pub pr_review_comment_count: u32,
    pub pr_changes_requested: u32,
    pub pr_bad_commits: Vec<String>,
    pub is_draft: bool,
    /// Why the worker reached its terminal status, if its log said — for a
    /// failed setup, the hook's status, output tail and log path.
    pub terminal_reason: Option<String>,
    /// How far bringing the worker up got — what `ps`, notifications and the
    /// fleet view say about setup, all read from here.
    pub setup: SetupState,
    /// Notification kinds already sent for this worker, from `Notified`
    /// events.
    pub notified: HashSet<String>,

    // ── Runtime (set by tick) ───────────────────────────────────
    pub repo: Option<RepoEntry>,
    pub name: String,
    pub resolved_branch: Branch,
    pub mux_status: MuxStatus,
    pub mux_agent_state: Option<jig_core::mux::AgentState>,
    pub commits_ahead: usize,
    pub is_dirty: bool,
    pub pr_health: PrHealth,
    pub nudge_cooldown_remaining: Option<u64>,
}

impl WorkerState {
    pub fn repo_name(&self) -> String {
        self.repo
            .as_ref()
            .and_then(|r| r.path.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    pub fn nudge_count(&self) -> u32 {
        self.nudge_counts.values().sum()
    }

    /// Whether `ps` lists this worker.
    ///
    /// A live worker is listed while its window is there. A failed one stays
    /// listed until it is pruned — that's when someone needs to look — and so
    /// does one still setting up, which has no window yet.
    pub fn is_displayed(&self) -> bool {
        self.status == WorkerStatus::Failed
            || matches!(self.setup, SetupState::Running { .. })
            || (!self.status.is_terminal() && self.mux_status != MuxStatus::NotFound)
    }
}

/// Where a worker's setup is. Folded from `SetupStarted` / `SetupFinished`
/// events; see [`SetupPhase`] for the steps.
///
/// NOTE(canonical): this is the one structured record of setup. `ps`,
/// notifications and the fleet view render from it — **do not** parse
/// `terminal_reason` or `setup.log` for setup status.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SetupState {
    /// No phase has started — or the log predates setup events.
    #[default]
    Pending,
    /// `phase` began at `since` (unix seconds). `elapsed_ms` is what the
    /// phases before it took; `log_path` is where this one's output goes.
    Running {
        phase: SetupPhase,
        since: i64,
        #[serde(default)]
        elapsed_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        log_path: Option<PathBuf>,
    },
    /// Every phase so far succeeded, taking `duration_ms` between them.
    Ok { duration_ms: u64 },
    /// `phase` failed. `tail` is the end of its output, or the error.
    Failed {
        phase: SetupPhase,
        exit: SetupExit,
        #[serde(default)]
        tail: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        log_path: Option<PathBuf>,
    },
}

/// How a failed setup phase ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupExit {
    /// The hook exited with this status.
    Code(i32),
    /// The hook outran its deadline and was killed.
    TimedOut,
    /// Killed by a signal, or the phase isn't a process at all (git, copying
    /// files, starting the agent) — the tail says which.
    Error,
}

impl SetupExit {
    pub fn from_event(exit_code: Option<i32>, timed_out: bool) -> Self {
        match (exit_code, timed_out) {
            (_, true) => Self::TimedOut,
            (Some(code), false) => Self::Code(code),
            (None, false) => Self::Error,
        }
    }
}

impl std::fmt::Display for SetupExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Code(code) => write!(f, "exit {code}"),
            Self::TimedOut => f.write_str("timed out"),
            Self::Error => f.write_str("error"),
        }
    }
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            status: WorkerStatus::Created,
            branch: None,
            commit_count: 0,
            last_commit_at: None,
            pr_url: None,
            nudge_counts: HashMap::new(),
            last_nudge_at: HashMap::new(),
            issue_ref: None,
            started_at: None,
            last_event_at: None,
            review_feedback_count: 0,
            pr_ci_passed: None,
            pr_ci_failures: Vec::new(),
            pr_has_conflicts: None,
            pr_review_comment_count: 0,
            pr_changes_requested: 0,
            pr_bad_commits: Vec::new(),
            is_draft: false,
            terminal_reason: None,
            setup: SetupState::default(),
            notified: HashSet::new(),

            repo: None,
            name: String::new(),
            resolved_branch: Branch::new("unknown"),
            mux_status: MuxStatus::default(),
            mux_agent_state: None,
            commits_ahead: 0,
            is_dirty: false,
            pr_health: PrHealth::default(),
            nudge_cooldown_remaining: None,
        }
    }
}

impl WorkerState {
    pub fn check_silence(&mut self, config: &AppConfig) {
        // Silence only suggests a hung agent while it is supposed to be
        // working. Idle, waiting and review states are quiet by design.
        if !self.status.is_active() {
            return;
        }
        if let Some(last_ts) = self.last_event_at {
            let now = chrono::Utc::now().timestamp();
            let age = now - last_ts;
            if age > config.silence_threshold_seconds as i64 {
                self.status = WorkerStatus::Stalled;
            }
        }
    }
}
