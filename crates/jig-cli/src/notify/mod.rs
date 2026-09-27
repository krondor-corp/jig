//! Notification system for human-facing alerts.
//!
//! Append-only JSONL queue at `~/.config/jig/state/notifications.jsonl`.

use jig_core::exec::ExecError;

mod events;
mod hook;
mod queue;

#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// A hook that could not be run, or that outran its deadline.
    #[error(transparent)]
    Exec(#[from] ExecError),
    #[error("{0}")]
    Hook(String),
}

pub use events::{Notification, NotificationEvent};
pub use hook::Notifier;
pub use queue::NotificationQueue;
