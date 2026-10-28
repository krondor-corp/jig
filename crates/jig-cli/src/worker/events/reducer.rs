//! State reducer — folds a worker's events into [`WorkerState`].

use url::Url;

use jig_core::ReducibleKind;

use super::{SetupExit, SetupState, WorkerState};
use crate::worker::WorkerStatus;

#[cfg(test)]
use super::schema::Event;
#[cfg(test)]
use super::schema::SetupPhase;
use super::schema::{EventKind, TerminalKind};

impl ReducibleKind for EventKind {
    type State = WorkerState;

    fn apply(state: &mut WorkerState, ts: i64, kind: &EventKind) {
        // The daemon's own record, not the worker doing anything: it must
        // not reset the silence clock.
        if let EventKind::Notified { kind } = kind {
            state.notified.insert(kind.clone());
            return;
        }

        if state.started_at.is_none() {
            state.started_at = Some(ts);
        }
        state.last_event_at = Some(ts);

        // Setup and notification records are bookkeeping, not lifecycle:
        // they apply whatever the status, and the failure that ends a setup
        // is written just before the `Terminal` it causes.
        match kind {
            EventKind::SetupStarted { phase, log_path } => {
                let elapsed_ms = match state.setup {
                    SetupState::Ok { duration_ms } => duration_ms,
                    SetupState::Running { elapsed_ms, .. } => elapsed_ms,
                    SetupState::Pending | SetupState::Failed { .. } => 0,
                };
                state.setup = SetupState::Running {
                    phase: *phase,
                    since: ts,
                    elapsed_ms,
                    log_path: log_path.clone(),
                };
                return;
            }
            EventKind::SetupFinished {
                phase,
                ok,
                exit_code,
                timed_out,
                duration_ms,
                tail,
            } => {
                let (elapsed_ms, log_path) = match &state.setup {
                    SetupState::Running {
                        elapsed_ms,
                        log_path,
                        ..
                    } => (*elapsed_ms, log_path.clone()),
                    _ => (0, None),
                };
                state.setup = if *ok {
                    SetupState::Ok {
                        duration_ms: elapsed_ms + duration_ms,
                    }
                } else {
                    SetupState::Failed {
                        phase: *phase,
                        exit: SetupExit::from_event(*exit_code, *timed_out),
                        tail: tail.clone(),
                        log_path,
                    }
                };
                return;
            }
            _ => {}
        }

        // Terminal events are sticky
        if let EventKind::Terminal { terminal, reason } = kind {
            state.terminal_reason = reason.clone();
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
                // An agent that stops with a PR open is waiting on the
                // reviewer, not idle.
                state.status = if state.pr_url.is_some() {
                    WorkerStatus::WaitingReview
                } else {
                    WorkerStatus::Idle
                };
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
                state.status = WorkerStatus::Closed;
            }
            EventKind::Terminal { .. }
            | EventKind::SetupStarted { .. }
            | EventKind::SetupFinished { .. }
            | EventKind::Notified { .. } => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AppConfig;
    use jig_core::issues::issue::IssueRef;

    fn default_config() -> AppConfig {
        AppConfig::default()
    }

    fn reduce(events: &[Event], config: &AppConfig) -> WorkerState {
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
    fn terminal_reason_is_kept() {
        let events = vec![
            Event::now(EventKind::Initializing {
                branch: "main".into(),
                base: "origin/main".into(),
                auto: true,
            }),
            Event::now(EventKind::Terminal {
                terminal: TerminalKind::Failed,
                reason: Some("on_create hook exited with status 1".into()),
            }),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::Failed);
        assert_eq!(
            state.terminal_reason.as_deref(),
            Some("on_create hook exited with status 1")
        );
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
        let config = AppConfig {
            silence_threshold_seconds: 300,
            ..Default::default()
        };
        let state = reduce(&events, &config);
        assert_eq!(state.status, WorkerStatus::Stalled);
    }

    fn silence_config() -> AppConfig {
        AppConfig {
            silence_threshold_seconds: 300,
            ..Default::default()
        }
    }

    fn at(ts: i64, kind: EventKind) -> Event {
        Event { ts, kind }
    }

    /// KRO-244. The agent keeps working after the PR poll writes `PrOpened`,
    /// then stops. It used to land on `Idle`, which silence turned `Stalled`.
    #[test]
    fn stopping_with_a_pr_open_waits_for_review_through_silence() {
        let old_ts = chrono::Utc::now().timestamp() - 600;
        let events = vec![
            at(
                old_ts,
                EventKind::Spawn {
                    branch: "main".into(),
                    repo: "r".into(),
                    issue: IssueRef::new("JIG-1"),
                },
            ),
            at(
                old_ts,
                EventKind::PrOpened {
                    pr_url: "https://github.com/o/r/pull/1".into(),
                    pr_number: "1".into(),
                },
            ),
            at(old_ts, EventKind::ToolUseStart),
            at(old_ts, EventKind::ToolUseEnd),
            at(old_ts, EventKind::Stop),
        ];
        let state = reduce(&events, &silence_config());
        assert_eq!(state.status, WorkerStatus::WaitingReview);
    }

    #[test]
    fn tool_use_with_a_pr_open_is_running() {
        let events = vec![
            Event::now(EventKind::PrOpened {
                pr_url: "https://github.com/o/r/pull/1".into(),
                pr_number: "1".into(),
            }),
            Event::now(EventKind::ToolUseStart),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::Running);
    }

    #[test]
    fn idle_is_not_marked_stalled() {
        let old_ts = chrono::Utc::now().timestamp() - 600;
        let events = vec![
            at(old_ts, EventKind::ToolUseEnd),
            at(old_ts, EventKind::Stop),
        ];
        let state = reduce(&events, &silence_config());
        assert_eq!(state.status, WorkerStatus::Idle);
    }

    #[test]
    fn waiting_input_is_not_marked_stalled() {
        let old_ts = chrono::Utc::now().timestamp() - 600;
        let events = vec![at(old_ts, EventKind::Notification)];
        let state = reduce(&events, &silence_config());
        assert_eq!(state.status, WorkerStatus::WaitingInput);
    }

    #[test]
    fn approved_is_not_marked_stalled() {
        let old_ts = chrono::Utc::now().timestamp() - 600;
        let events = vec![at(
            old_ts,
            EventKind::Terminal {
                terminal: TerminalKind::Approved,
                reason: None,
            },
        )];
        let state = reduce(&events, &silence_config());
        assert_eq!(state.status, WorkerStatus::Approved);
    }

    #[test]
    fn silent_spawn_is_marked_stalled() {
        let old_ts = chrono::Utc::now().timestamp() - 600;
        let events = vec![at(
            old_ts,
            EventKind::Spawn {
                branch: "main".into(),
                repo: "r".into(),
                issue: IssueRef::new("JIG-1"),
            },
        )];
        let state = reduce(&events, &silence_config());
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
        let config = AppConfig {
            silence_threshold_seconds: 300,
            ..Default::default()
        };
        let state = reduce(&events, &config);
        assert_eq!(state.status, WorkerStatus::Initializing);
    }

    /// KRO-219. A closed PR used to reduce to `Failed`, the same status a
    /// worker gets when its mux window is lost — which meant the daemon could
    /// not tell "this PR is over" from "this worker broke", and pruned both.
    #[test]
    fn a_closed_pr_is_closed_not_failed() {
        let state = reduce(
            &[Event::now(EventKind::PrClosed {
                pr_url: "https://github.com/o/r/pull/1".to_string(),
            })],
            &default_config(),
        );

        assert_eq!(state.status, WorkerStatus::Closed);
        assert!(state.status.is_pr_outcome(), "a closed PR is an outcome");
    }

    #[test]
    fn a_broken_worker_is_failed_and_not_a_pr_outcome() {
        let state = reduce(
            &[Event::now(EventKind::Terminal {
                terminal: TerminalKind::Failed,
                reason: Some("mux window lost, 3 resume attempts failed".to_string()),
            })],
            &default_config(),
        );

        assert_eq!(state.status, WorkerStatus::Failed);
        assert!(
            !state.status.is_pr_outcome(),
            "a broken worker must not be pruned as though its PR ended"
        );
    }

    #[test]
    fn a_merged_pr_is_a_pr_outcome() {
        let state = reduce(
            &[Event::now(EventKind::PrMerged {
                pr_url: "https://github.com/o/r/pull/1".to_string(),
            })],
            &default_config(),
        );
        assert_eq!(state.status, WorkerStatus::Merged);
        assert!(state.status.is_pr_outcome());
    }

    /// The count has to survive a reduce, or the daemon's change detector
    /// compares a fresh count against a zero it just produced and reports
    /// new feedback on every poll.
    ///
    /// `PrReviewFeedback` was in the schema and handled here from the start,
    /// but nothing ever wrote one — so this branch was dead and
    /// `review_feedback_count` reduced to 0 forever. One draft PR with 14
    /// review comments sent a `feedback_received` alert every 60 seconds.
    #[test]
    fn review_feedback_survives_a_reduce() {
        let mut state = WorkerState::default();
        EventKind::apply(
            &mut state,
            0,
            &EventKind::PrReviewFeedback {
                comment_count: 14,
                changes_requested: 1,
            },
        );

        assert_eq!(state.review_feedback_count, 15);
        assert_eq!(state.pr_review_comment_count, 14);
        assert_eq!(state.pr_changes_requested, 1);
    }

    fn started(ts: i64, phase: SetupPhase) -> Event {
        at(
            ts,
            EventKind::SetupStarted {
                phase,
                log_path: (phase == SetupPhase::OnCreate).then(|| "/x/setup.log".into()),
            },
        )
    }

    fn finished(ts: i64, phase: SetupPhase, duration_ms: u64) -> Event {
        at(
            ts,
            EventKind::SetupFinished {
                phase,
                ok: true,
                exit_code: None,
                timed_out: false,
                duration_ms,
                tail: vec![],
            },
        )
    }

    fn initializing(ts: i64) -> Event {
        at(
            ts,
            EventKind::Initializing {
                branch: "feat".into(),
                base: "main".into(),
                auto: true,
            },
        )
    }

    #[test]
    fn setup_is_pending_before_any_phase() {
        let state = reduce(&[initializing(0)], &default_config());
        assert_eq!(state.setup, SetupState::Pending);
    }

    #[test]
    fn a_running_phase_says_which_and_since_when() {
        let events = vec![
            initializing(100),
            started(100, SetupPhase::Worktree),
            finished(101, SetupPhase::Worktree, 800),
            started(101, SetupPhase::OnCreate),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(
            state.setup,
            SetupState::Running {
                phase: SetupPhase::OnCreate,
                since: 101,
                elapsed_ms: 800,
                log_path: Some("/x/setup.log".into()),
            }
        );
        assert_eq!(state.status, WorkerStatus::Initializing);
    }

    #[test]
    fn a_finished_setup_adds_up_its_phases() {
        let events = vec![
            initializing(0),
            started(0, SetupPhase::Worktree),
            finished(0, SetupPhase::Worktree, 100),
            started(0, SetupPhase::CopyFiles),
            finished(0, SetupPhase::CopyFiles, 20),
            started(0, SetupPhase::OnCreate),
            finished(3, SetupPhase::OnCreate, 3000),
            Event::now(EventKind::Spawn {
                branch: "feat".into(),
                repo: "r".into(),
                issue: IssueRef::new("JIG-1"),
            }),
            started(3, SetupPhase::AgentLaunch),
            finished(3, SetupPhase::AgentLaunch, 5),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.setup, SetupState::Ok { duration_ms: 3125 });
    }

    /// The acceptance case: a failing `on_create` is data — phase, exit and
    /// tail — and the log path it was written to.
    #[test]
    fn a_failing_on_create_is_a_failed_setup_with_phase_exit_and_tail() {
        let events = vec![
            initializing(0),
            started(0, SetupPhase::Worktree),
            finished(0, SetupPhase::Worktree, 100),
            started(0, SetupPhase::OnCreate),
            at(
                2,
                EventKind::SetupFinished {
                    phase: SetupPhase::OnCreate,
                    ok: false,
                    exit_code: Some(1),
                    timed_out: false,
                    duration_ms: 2000,
                    tail: vec!["ERR_PNPM_OUTDATED_LOCKFILE".into()],
                },
            ),
            at(
                2,
                EventKind::Terminal {
                    terminal: TerminalKind::Failed,
                    reason: Some("on_create hook exited with status 1".into()),
                },
            ),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::Failed);
        assert_eq!(
            state.setup,
            SetupState::Failed {
                phase: SetupPhase::OnCreate,
                exit: SetupExit::Code(1),
                tail: vec!["ERR_PNPM_OUTDATED_LOCKFILE".into()],
                log_path: Some("/x/setup.log".into()),
            }
        );
    }

    #[test]
    fn a_timed_out_hook_is_timed_out_not_a_code() {
        let events = vec![
            started(0, SetupPhase::OnCreate),
            at(
                0,
                EventKind::SetupFinished {
                    phase: SetupPhase::OnCreate,
                    ok: false,
                    exit_code: None,
                    timed_out: true,
                    duration_ms: 600_000,
                    tail: vec![],
                },
            ),
        ];
        let state = reduce(&events, &default_config());
        assert!(matches!(
            state.setup,
            SetupState::Failed {
                exit: SetupExit::TimedOut,
                ..
            }
        ));
    }

    /// `Notified` lands after the `Terminal` that caused it, so it must not
    /// be swallowed by terminal stickiness.
    #[test]
    fn notified_is_recorded_after_a_terminal_event() {
        let events = vec![
            Event::now(EventKind::Terminal {
                terminal: TerminalKind::Failed,
                reason: None,
            }),
            Event::now(EventKind::Notified {
                kind: "needs_intervention".into(),
            }),
        ];
        let state = reduce(&events, &default_config());
        assert_eq!(state.status, WorkerStatus::Failed);
        assert!(state.notified.contains("needs_intervention"));
    }

    #[test]
    fn notified_does_not_count_as_activity() {
        let old_ts = chrono::Utc::now().timestamp() - 600;
        let events = vec![
            at(old_ts, EventKind::ToolUseEnd),
            Event::now(EventKind::Notified {
                kind: "pr_opened".into(),
            }),
        ];
        let state = reduce(&events, &silence_config());
        assert_eq!(state.last_event_at, Some(old_ts));
        assert_eq!(state.status, WorkerStatus::Stalled);
    }
}
