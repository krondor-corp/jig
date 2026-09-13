//! Daemon event log — append-only JSONL lifecycle events.

mod reducer;
mod schema;

pub use reducer::DaemonState;
pub use schema::{started, stopped, Event, EventKind};

use crate::context::JigDirs;
use jig_core::EventLog;

pub type DaemonLog = EventLog<Event>;

pub fn global(dirs: &JigDirs) -> DaemonLog {
    EventLog::new(dirs.daemon_lifecycle_log())
}
