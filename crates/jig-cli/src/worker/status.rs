use serde::{Deserialize, Serialize};

/// Multiplexer-level status of a worker's window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MuxStatus {
    Running,
    Exited,
    #[default]
    NotFound,
}

impl MuxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Exited => "exited",
            Self::NotFound => "not-found",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    /// Bare worktree created via `jig create` — not daemon-managed
    Created,
    /// Worker is being created (worktree + on-create hook running)
    Initializing,
    /// Worker just spawned, no events yet
    Spawned,
    /// Tool use events flowing, actively working
    Running,
    /// Stop event fired, agent at shell prompt
    Idle,
    /// Notification event fired, agent waiting for input
    WaitingInput,
    /// No events for silence_threshold, agent may be stuck
    Stalled,
    /// PR opened, waiting for human review
    WaitingReview,
    /// PR approved, ready to merge
    Approved,
    /// PR merged successfully
    Merged,
    /// PR closed without merging. An outcome, not a failure — the work is
    /// over either way, but nobody needs to go and look at it.
    Closed,
    /// Worker failed or was killed — the mux window was lost, or it never
    /// started. Distinct from [`Self::Closed`] because the worktree of a
    /// worker that broke is worth keeping and one whose PR closed is not.
    Failed,
    /// Worker archived/cleaned up
    Archived,
}

impl WorkerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Initializing => "initializing",
            Self::Spawned => "spawned",
            Self::Running => "running",
            Self::Idle => "idle",
            Self::WaitingInput => "waiting_input",
            Self::Stalled => "stalled",
            Self::WaitingReview => "waiting_review",
            Self::Approved => "approved",
            Self::Merged => "merged",
            Self::Closed => "closed",
            Self::Failed => "failed",
            Self::Archived => "archived",
        }
    }

    pub fn needs_attention(&self) -> bool {
        matches!(self, Self::WaitingInput | Self::Stalled | Self::Failed)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Spawned)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Merged | Self::Closed | Self::Archived | Self::Failed
        )
    }

    /// Whether this is how a PR ended, as opposed to how a worker broke.
    /// Only these get their worktree pruned automatically.
    pub fn is_pr_outcome(&self) -> bool {
        matches!(self, Self::Merged | Self::Closed)
    }

    pub fn is_waiting_review(&self) -> bool {
        matches!(self, Self::WaitingReview)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_needs_attention() {
        assert!(WorkerStatus::WaitingInput.needs_attention());
        assert!(WorkerStatus::Stalled.needs_attention());
        assert!(WorkerStatus::Failed.needs_attention());
        assert!(!WorkerStatus::Running.needs_attention());
    }

    #[test]
    fn status_is_active() {
        assert!(WorkerStatus::Running.is_active());
        assert!(WorkerStatus::Spawned.is_active());
        assert!(!WorkerStatus::Idle.is_active());
        assert!(!WorkerStatus::Merged.is_active());
    }

    #[test]
    fn status_is_terminal() {
        assert!(WorkerStatus::Merged.is_terminal());
        assert!(WorkerStatus::Archived.is_terminal());
        assert!(WorkerStatus::Failed.is_terminal());
        assert!(!WorkerStatus::Running.is_terminal());
    }

    #[test]
    fn status_serialization() {
        let status = WorkerStatus::WaitingInput;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"waiting_input\"");
        let parsed: WorkerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, WorkerStatus::WaitingInput);
    }

    #[test]
    fn status_all_variants_roundtrip() {
        let variants = [
            WorkerStatus::Created,
            WorkerStatus::Initializing,
            WorkerStatus::Spawned,
            WorkerStatus::Running,
            WorkerStatus::Idle,
            WorkerStatus::WaitingInput,
            WorkerStatus::Stalled,
            WorkerStatus::WaitingReview,
            WorkerStatus::Approved,
            WorkerStatus::Merged,
            WorkerStatus::Failed,
            WorkerStatus::Archived,
        ];
        for status in &variants {
            let json = serde_json::to_string(status).unwrap();
            let parsed: WorkerStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, status);
        }
    }

    #[test]
    fn only_pr_outcomes_are_pr_outcomes() {
        assert!(WorkerStatus::Merged.is_pr_outcome());
        assert!(WorkerStatus::Closed.is_pr_outcome());

        // The ones that must keep their worktree.
        assert!(!WorkerStatus::Failed.is_pr_outcome());
        assert!(!WorkerStatus::Archived.is_pr_outcome());
        assert!(!WorkerStatus::Stalled.is_pr_outcome());
    }

    #[test]
    fn a_closed_pr_is_terminal_but_needs_no_attention() {
        assert!(WorkerStatus::Closed.is_terminal());
        assert!(
            !WorkerStatus::Closed.needs_attention(),
            "a closed PR is a resolved outcome, not something to go and look at"
        );
    }
}
