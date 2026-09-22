//! Daemon event log — append-only JSONL lifecycle events.

mod reducer;
mod schema;

pub use reducer::DaemonState;
pub use schema::{started, stopped, Event, EventKind};

use crate::context::AppPaths;
use jig_core::EventLog;

pub type DaemonLog = EventLog<Event>;

pub fn global(paths: &AppPaths) -> DaemonLog {
    EventLog::new(paths.daemon_lifecycle_log())
}
