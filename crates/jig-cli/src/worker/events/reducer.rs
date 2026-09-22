//! State reducer — folds a worker's events into [`WorkerState`].

use url::Url;

use jig_core::ReducibleKind;

use super::WorkerState;
use crate::worker::WorkerStatus;

#[cfg(test)]
use super::schema::Event;
use super::schema::{EventKind, TerminalKind};

impl ReducibleKind for EventKind {
    type State = WorkerState;

    fn apply(state: &mut WorkerState, ts: i64, kind: &EventKind) {
        if state.started_at.is_none() {
            state.started_at = Some(ts);
        }
        state.last_event_at = Some(ts);

        // Terminal events are sticky
        if let EventKind::Terminal { terminal, .. } = kind {
            state.status = match terminal {
                TerminalKind::Merged => WorkerStatus::Merged,
                TerminalKind::Approved => WorkerStatus::Approved,
                TerminalKind::Failed => WorkerStatus::Failed,
                TerminalKind::Archived => WorkerStatus::Archived,
            };
            return;
        }

        if state.status.is_terminal() {
            return;
        }

        match kind {
            EventKind::Create { .. } => {
                state.status = WorkerStatus::Created;
            }
            EventKind::Initializing { branch, .. } => {
                state.status = WorkerStatus::Initializing;
                state.branch = Some(branch.clone());
            }
            EventKind::Spawn { branch, issue, .. } => {
                state.status = WorkerStatus::Spawned;
                state.branch = Some(branch.clone());
                state.issue_ref = Some(issue.clone());
            }
            EventKind::Resume => {
                state.status = WorkerStatus::Spawned;
            }
            EventKind::ToolUseStart | EventKind::ToolUseEnd => {
                state.status = WorkerStatus::Running;
            }
            EventKind::Commit { .. } => {
                state.status = WorkerStatus::Running;
                state.commit_count += 1;
                state.last_commit_at = Some(ts);
            }
            EventKind::Push { .. } => {
                state.status = WorkerStatus::Running;
            }
            EventKind::Notification => {
                state.status = WorkerStatus::WaitingInput;
            }
            EventKind::Stop => {
                state.status = WorkerStatus::Idle;
            }
            EventKind::PrOpened { pr_url, .. } => {
                state.status = WorkerStatus::WaitingReview;
                state.pr_url = Url::parse(pr_url).ok();
            }
            EventKind::Nudge { nudge_type, .. } => {
                *state.nudge_counts.entry(nudge_type.clone()).or_insert(0) += 1;
                state.last_nudge_at.insert(nudge_type.clone(), ts);
            }
            EventKind::CiStatus => {}
            EventKind::PrCiStatus { passed, failures } => {
                state.pr_ci_passed = Some(*passed);
                state.pr_ci_failures = failures.clone();
            }
            EventKind::PrConflict { has_conflict } => {
                state.pr_has_conflicts = Some(*has_conflict);
            }
            EventKind::PrReviewFeedback {
                comment_count,
                changes_requested,
            } => {
                state.pr_review_comment_count = *comment_count;
                state.pr_changes_requested = *changes_requested;
                state.review_feedback_count = comment_count + changes_requested;
            }
            EventKind::PrCommitLint { bad_commits } => {
                state.pr_bad_commits = bad_commits.clone();
            }
            EventKind::PrMerged { pr_url } => {
                state.pr_url = Url::parse(pr_url).ok();
                state.status = WorkerStatus::Merged;
            }
            EventKind::PrClosed { .. } => {
                state.status = WorkerStatus::Failed;
            }
            EventKind::Terminal { .. } => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Config;
    use jig_core::issues::issue::IssueRef;

    fn default_config() -> Config {
        Config::default()
    }

    fn reduce(events: &[Event], config: &Config) -> WorkerState {
        let mut state = WorkerState::default();
        for event in events {
            EventKind::apply(&mut state, event.ts, &event.kind);
        }
        state.check_silence(config);
        state
    }

    #[test]
    fn empty_events_returns_created() {
        let state = reduce(&[], &default_config());
        assert_eq!(state.status, WorkerStatus::Created);
        assert_eq!(state.commit_count, 0);
    }

    #[test]
    fn commit_count_accumulates() {
        let events = vec![
            Event::now(EventKind::Spawn {
                branch: "main".into(),
                repo: "r".into(),
                issue: IssueRef::new("JIG-1"),
            }),
            Event::now(EventKind::Commit {
                sha: "abc".into(),
                repo: "r".into(),
            }),
            Event::now(EventKind::Commit {
                sha: "def".into(),
                repo: "r".into(),
            }),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.commit_count, 2);
        assert!(state.last_commit_at.is_some());
    }

    #[test]
    fn pr_url_extracted() {
        let events = vec![
            Event::now(EventKind::Spawn {
                branch: "main".into(),
                repo: "r".into(),
                issue: IssueRef::new("JIG-1"),
            }),
            Event::now(EventKind::PrOpened {
                pr_url: "https://github.com/pr/1".into(),
                pr_number: "1".into(),
            }),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::WaitingReview);
        assert_eq!(
            state.pr_url.as_ref().map(Url::to_string).as_deref(),
            Some("https://github.com/pr/1")
        );
    }

    #[test]
    fn issue_ref_extracted() {
        let events = vec![
            Event::now(EventKind::Spawn {
                branch: "main".into(),
                repo: "r".into(),
                issue: IssueRef::new("features/smart-context"),
            }),
            Event::now(EventKind::ToolUseStart),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.issue_ref.as_deref(), Some("features/smart-context"));
    }

    #[test]
    fn nudge_counts_tracked() {
        let events = vec![
            Event::now(EventKind::Spawn {
                branch: "main".into(),
                repo: "r".into(),
                issue: IssueRef::new("JIG-1"),
            }),
            Event::now(EventKind::Nudge {
                nudge_type: "stalled".into(),
                message: "m".into(),
            }),
            Event::now(EventKind::Nudge {
                nudge_type: "stalled".into(),
                message: "m".into(),
            }),
            Event::now(EventKind::Nudge {
                nudge_type: "waiting".into(),
                message: "m".into(),
            }),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.nudge_counts.get("stalled"), Some(&2));
        assert_eq!(state.nudge_counts.get("waiting"), Some(&1));
    }

    #[test]
    fn last_nudge_at_tracked() {
        let now = chrono::Utc::now().timestamp();
        let events = vec![
            Event::now(EventKind::Spawn {
                branch: "main".into(),
                repo: "r".into(),
                issue: IssueRef::new("JIG-1"),
            }),
            Event {
                ts: now - 600,
                kind: EventKind::Nudge {
                    nudge_type: "ci".into(),
                    message: "m".into(),
                },
            },
            Event {
                ts: now - 100,
                kind: EventKind::Nudge {
                    nudge_type: "ci".into(),
                    message: "m".into(),
                },
            },
            Event {
                ts: now - 500,
                kind: EventKind::Nudge {
                    nudge_type: "review".into(),
                    message: "m".into(),
                },
            },
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.last_nudge_at.get("ci"), Some(&(now - 100)));
        assert_eq!(state.last_nudge_at.get("review"), Some(&(now - 500)));
        assert_eq!(state.nudge_counts.get("ci"), Some(&2));
    }

    #[test]
    fn terminal_state_is_sticky() {
        let events = vec![
            Event::now(EventKind::Spawn {
                branch: "main".into(),
                repo: "r".into(),
                issue: IssueRef::new("JIG-1"),
            }),
            Event::now(EventKind::Terminal {
                terminal: TerminalKind::Failed,
                reason: None,
            }),
            Event::now(EventKind::ToolUseStart),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::Failed);
    }

    #[test]
    fn timestamps_tracked() {
        let events = vec![
            Event::now(EventKind::Spawn {
                branch: "main".into(),
                repo: "r".into(),
                issue: IssueRef::new("JIG-1"),
            }),
            Event::now(EventKind::ToolUseEnd),
        ];
        let state = reduce(&events, &default_config());
        assert!(state.started_at.is_some());
        assert!(state.last_event_at.is_some());
    }

    #[test]
    fn resume_preserves_commit_count_and_issue_ref() {
        let events = vec![
            Event::now(EventKind::Spawn {
                branch: "main".into(),
                repo: "r".into(),
                issue: IssueRef::new("features/smart-context"),
            }),
            Event::now(EventKind::Commit {
                sha: "abc".into(),
                repo: "r".into(),
            }),
            Event::now(EventKind::Commit {
                sha: "def".into(),
                repo: "r".into(),
            }),
            Event::now(EventKind::Resume),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::Spawned);
        assert_eq!(state.commit_count, 2);
        assert_eq!(state.issue_ref.as_deref(), Some("features/smart-context"));
    }

    #[test]
    fn resume_transitions_to_spawned() {
        let events = vec![
            Event::now(EventKind::Spawn {
                branch: "main".into(),
                repo: "r".into(),
                issue: IssueRef::new("JIG-1"),
            }),
            Event::now(EventKind::ToolUseStart),
            Event::now(EventKind::Stop),
            Event::now(EventKind::Resume),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::Spawned);
    }

    #[test]
    fn silence_triggers_stalled() {
        let old_ts = chrono::Utc::now().timestamp() - 600;
        let events = vec![Event {
            ts: old_ts,
            kind: EventKind::ToolUseEnd,
        }];
        let config = Config {
            silence_threshold_seconds: 300,
            ..Default::default()
        };
        let state = reduce(&events, &config);
        assert_eq!(state.status, WorkerStatus::Stalled);
    }

    #[test]
    fn initializing_event_sets_status() {
        let events = vec![Event::now(EventKind::Initializing {
            branch: "main".into(),
            base: "main".into(),
            auto: false,
        })];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::Initializing);
    }

    #[test]
    fn initializing_transitions_to_spawned() {
        let events = vec![
            Event::now(EventKind::Initializing {
                branch: "feat/my-feature".into(),
                base: "main".into(),
                auto: false,
            }),
            Event::now(EventKind::Spawn {
                branch: "feat/my-feature".into(),
                repo: "r".into(),
                issue: IssueRef::new("features/my-feature"),
            }),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::Spawned);
    }

    #[test]
    fn initializing_to_failed_on_terminal() {
        let events = vec![
            Event::now(EventKind::Initializing {
                branch: "main".into(),
                base: "main".into(),
                auto: false,
            }),
            Event::now(EventKind::Terminal {
                terminal: TerminalKind::Failed,
                reason: Some("on-create hook failed".into()),
            }),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::Failed);
    }

    #[test]
    fn initializing_not_marked_stalled() {
        let old_ts = chrono::Utc::now().timestamp() - 600;
        let events = vec![Event {
            ts: old_ts,
            kind: EventKind::Initializing {
                branch: "main".into(),
                base: "main".into(),
                auto: false,
            },
        }];
        let config = Config {
            silence_threshold_seconds: 300,
            ..Default::default()
        };
        let state = reduce(&events, &config);
        assert_eq!(state.status, WorkerStatus::Initializing);
    }
}
