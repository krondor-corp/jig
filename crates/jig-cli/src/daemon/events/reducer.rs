//! State reducer — builds DaemonState from events.

use jig_core::ReducibleKind;

use super::schema::EventKind;
#[cfg(test)]
use super::schema::{started, stopped, Event};

#[derive(Debug, Clone, Default)]
pub struct DaemonState {
    pub started_at: Option<i64>,
    pub stopped_at: Option<i64>,
    pub pid: Option<u32>,
    pub stop_reason: Option<String>,
    /// The latest run's tracing log, when it recorded one.
    pub log: Option<std::path::PathBuf>,
}

impl ReducibleKind for EventKind {
    type State = DaemonState;

    fn apply(state: &mut DaemonState, ts: i64, kind: &EventKind) {
        match kind {
            EventKind::Started { pid, log } => {
                state.started_at = Some(ts);
                state.stopped_at = None;
                state.pid = Some(*pid);
                state.stop_reason = None;
                state.log = log.clone();
            }
            EventKind::Stopped { pid: _, reason } => {
                state.stopped_at = Some(ts);
                state.stop_reason = Some(reason.clone());
            }
        }
    }
}

impl DaemonState {
    pub fn previous_run_crashed(&self) -> bool {
        self.started_at.is_some() && self.stopped_at.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reduce(events: &[Event]) -> DaemonState {
        let mut state = DaemonState::default();
        for event in events {
            EventKind::apply(&mut state, event.ts, &event.kind);
        }
        state
    }

    #[test]
    fn empty_events_defaults() {
        let state = reduce(&[]);
        assert!(state.started_at.is_none());
        assert!(!state.previous_run_crashed());
    }

    #[test]
    fn started_without_stopped_is_crash() {
        let events = vec![started(None)];
        let state = reduce(&events);
        assert!(state.started_at.is_some());
        assert!(state.previous_run_crashed());
    }

    #[test]
    fn started_then_stopped_is_clean() {
        let events = vec![started(None), stopped("normal")];
        let state = reduce(&events);
        assert!(state.started_at.is_some());
        assert!(state.stopped_at.is_some());
        assert!(!state.previous_run_crashed());
        assert_eq!(state.stop_reason.as_deref(), Some("normal"));
    }

    #[test]
    fn multiple_runs_last_wins() {
        let events = vec![started(None), stopped("normal"), started(None)];
        let state = reduce(&events);
        assert!(state.previous_run_crashed());
        assert!(state.stopped_at.is_none());
    }

    #[test]
    fn latest_start_names_the_log() {
        let first = std::path::PathBuf::from("/logs/1.log");
        let second = std::path::PathBuf::from("/logs/2.log");
        let state = reduce(&[
            started(Some(first)),
            stopped("normal"),
            started(Some(second.clone())),
            stopped("normal"),
        ]);
        assert_eq!(state.log, Some(second), "a stop must not clear the log");
    }

    #[test]
    fn a_start_from_an_older_daemon_parses_without_a_log() {
        let event: Event = serde_json::from_str(r#"{"ts":1,"type":"started","pid":7}"#).unwrap();
        assert!(matches!(event.kind, EventKind::Started { log: None, .. }));
    }
}
