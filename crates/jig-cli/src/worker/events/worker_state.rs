//! `WorkerState` — what a worker's event log reduces to, plus the runtime
//! details the daemon fills in. The reduction itself lives in `reducer.rs`.

use std::collections::HashMap;

use url::Url;

use crate::context::{Config, RepoEntry};
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
    pub fn check_silence(&mut self, config: &Config) {
        if self.status.is_terminal() {
            return;
        }
        if matches!(
            self.status,
            WorkerStatus::WaitingReview | WorkerStatus::Initializing | WorkerStatus::Created
        ) {
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
