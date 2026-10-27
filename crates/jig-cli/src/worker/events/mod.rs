//! Event log system for worker lifecycle tracking.
//!
//! Append-only JSONL files per worker, stored in the global state dir.

mod reducer;
mod schema;
mod worker_state;

pub use schema::{Event, EventKind, EventType, SetupPhase, TerminalKind};
pub use worker_state::{SetupExit, SetupState, WorkerState};

/// A worker's event log. Opened by [`crate::context::log::Logs::worker_events`].
pub type EventLog = jig_core::EventLog<Event>;
