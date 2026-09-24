//! Emitting notifications: queue writes and the configured hook.

use jig_cli::context::NotifyConfig;
use jig_cli::notify::{NotificationEvent, NotificationQueue, Notifier};

/// The event these tests emit.
fn make_event() -> NotificationEvent {
    NotificationEvent::NeedsIntervention {
        repo: "myrepo".to_string(),
        worker: "jwt-auth".to_string(),
        reason: "stalled".to_string(),
    }
}

#[test]
fn should_trigger_with_matching_filter() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("n.jsonl"));
    let config = NotifyConfig {
        events: vec!["needs_intervention".to_string()],
        ..Default::default()
    };
    let notifier = Notifier::new(config, queue);

    assert!(notifier.should_trigger(&make_event()));
}

#[test]
fn should_not_trigger_with_non_matching_filter() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("n.jsonl"));
    let config = NotifyConfig {
        events: vec!["pr_opened".to_string()],
        ..Default::default()
    };
    let notifier = Notifier::new(config, queue);

    assert!(!notifier.should_trigger(&make_event()));
}

#[test]
fn empty_filter_triggers_all() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("n.jsonl"));
    let config = NotifyConfig::default();
    let notifier = Notifier::new(config, queue);

    assert!(notifier.should_trigger(&make_event()));
}

#[test]
fn emit_writes_to_queue() {
    let tmp = tempfile::tempdir().unwrap();
    let queue_path = tmp.path().join("n.jsonl");
    let queue = NotificationQueue::new(queue_path.clone());
    let config = NotifyConfig::default();
    let notifier = Notifier::new(config, queue);

    notifier.emit(make_event()).unwrap();

    let read_queue = NotificationQueue::new(queue_path);
    let notifications = read_queue.tail(10).unwrap();
    assert_eq!(notifications.len(), 1);
}

#[test]
fn emit_strict_returns_err_on_hook_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("n.jsonl"));
    let config = NotifyConfig {
        exec: Some("exit 1".to_string()),
        ..Default::default()
    };
    let notifier = Notifier::new(config, queue);

    let err = notifier.emit_strict(make_event()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("notification hook exited with"), "got: {msg}");
}

#[test]
fn emit_strict_captures_stderr() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("n.jsonl"));
    let config = NotifyConfig {
        exec: Some("echo 'bad config' >&2; exit 1".to_string()),
        ..Default::default()
    };
    let notifier = Notifier::new(config, queue);

    let err = notifier.emit_strict(make_event()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("bad config"), "got: {msg}");
}

#[test]
fn emit_strict_succeeds_on_hook_success() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("n.jsonl"));
    let config = NotifyConfig {
        exec: Some("cat > /dev/null".to_string()),
        ..Default::default()
    };
    let notifier = Notifier::new(config, queue);

    notifier.emit_strict(make_event()).unwrap();
    // Also verify the event was queued
    let read_queue = NotificationQueue::new(tmp.path().join("n.jsonl"));
    assert_eq!(read_queue.tail(10).unwrap().len(), 1);
}
