//! Notification event types for human consumption.

use serde::{Deserialize, Serialize};

/// High-level notification events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationEvent {
    WorkStarted {
        repo: String,
        worker: String,
        issue: Option<String>,
    },
    PrOpened {
        repo: String,
        worker: String,
        pr_url: String,
    },
    FeedbackReceived {
        repo: String,
        worker: String,
        pr_url: String,
    },
    FeedbackAddressed {
        repo: String,
        worker: String,
        pr_url: String,
    },
    NeedsIntervention {
        repo: String,
        worker: String,
        reason: String,
    },
    WorkCompleted {
        repo: String,
        worker: String,
        pr_url: Option<String>,
    },
}

/// A timestamped notification.
pub type Notification = jig_core::Event<NotificationEvent>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_serializes_flat() {
        let n = Notification {
            ts: 1000,
            kind: NotificationEvent::WorkStarted {
                repo: "jig".to_string(),
                worker: "feat".to_string(),
                issue: Some("ABC-123".to_string()),
            },
        };
        let json = serde_json::to_string(&n).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "work_started");
        assert_eq!(parsed["repo"], "jig");
        assert_eq!(parsed["ts"], 1000);
        assert!(parsed.get("id").is_none());
    }

    #[test]
    fn notification_roundtrip() {
        let n = Notification::now(NotificationEvent::NeedsIntervention {
            repo: "jig".to_string(),
            worker: "fix".to_string(),
            reason: "stalled".to_string(),
        });
        let json = serde_json::to_string(&n).unwrap();
        let restored: Notification = serde_json::from_str(&json).unwrap();
        assert!(restored.ts > 0);
        assert!(matches!(
            restored.kind,
            NotificationEvent::NeedsIntervention { .. }
        ));
    }

    #[test]
    fn old_format_with_id_deserializes() {
        // Old notification JSONL lines had an `id` field; serde ignores unknown fields.
        let old_json = r#"{"ts":2000,"id":"some-uuid","type":"needs_intervention","repo":"jig","worker":"fix","reason":"stalled"}"#;
        let restored: Notification = serde_json::from_str(old_json).unwrap();
        assert_eq!(restored.ts, 2000);
        assert!(matches!(
            restored.kind,
            NotificationEvent::NeedsIntervention { .. }
        ));
    }
}
