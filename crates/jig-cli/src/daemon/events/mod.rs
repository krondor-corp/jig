//! Daemon event log — append-only JSONL lifecycle events.

mod reducer;
mod schema;

pub use reducer::DaemonState;
pub use schema::{started, stopped, Event, EventKind};

use jig_core::EventLog;

/// Opened by [`crate::context::log::Logs::daemon_events`].
pub type DaemonLog = EventLog<Event>;
