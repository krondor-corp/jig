//! Typed event schema for the daemon event log.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub type Event = jig_core::Event<EventKind>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    Started {
        pid: u32,
        /// This run's tracing log, so `jig daemon logs` can find it after
        /// the daemon exits. Absent in events written by older daemons.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        log: Option<PathBuf>,
    },
    Stopped {
        pid: u32,
        reason: String,
    },
}

pub fn started(log: Option<PathBuf>) -> Event {
    Event::now(EventKind::Started {
        pid: std::process::id(),
        log,
    })
}

pub fn stopped(reason: &str) -> Event {
    Event::now(EventKind::Stopped {
        pid: std::process::id(),
        reason: reason.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn started_serializes_flat() {
        let event = started(None);
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["ts"].is_i64());
        assert_eq!(parsed["type"], "started");
        assert!(parsed["pid"].is_u64());
        assert!(parsed.get("kind").is_none());
    }

    #[test]
    fn stopped_roundtrip() {
        let original = stopped("signal");
        let json = serde_json::to_string(&original).unwrap();
        let restored: Event = serde_json::from_str(&json).unwrap();

        assert!(matches!(
            restored.kind,
            EventKind::Stopped { ref reason, .. } if reason == "signal"
        ));
    }
}
