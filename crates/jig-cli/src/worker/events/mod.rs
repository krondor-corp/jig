//! Event log system for worker lifecycle tracking.
//!
//! Append-only JSONL files per worker, stored in the global state dir.

mod reducer;
mod schema;

pub use reducer::WorkerState;
pub use schema::{Event, EventKind, EventType, TerminalKind};

/// Worker event log — wraps the core `EventLog` with a `for_worker` constructor.
pub type EventLog = jig_core::EventLog<Event>;

/// A worker's event log: `~/.config/jig/<repo>/<branch>/events.jsonl`
pub fn event_log_for_worker(dirs: &crate::context::JigDirs, repo: &str, branch: &str) -> EventLog {
    EventLog::new(dirs.worker_events_dir(repo, branch).join("events.jsonl"))
}
