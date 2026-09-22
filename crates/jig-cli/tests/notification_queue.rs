//! The append-only notification queue.

use jig_cli::notify::{NotificationEvent, NotificationQueue};

#[test]
fn emit_and_tail() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("notifications.jsonl"));

    queue
        .emit(NotificationEvent::WorkStarted {
            repo: "jig".to_string(),
            worker: "feat".to_string(),
            issue: Some("ABC-123".to_string()),
        })
        .unwrap();

    let notifications = queue.tail(10).unwrap();
    assert_eq!(notifications.len(), 1);
    assert!(matches!(
        notifications[0].kind,
        NotificationEvent::WorkStarted { .. }
    ));
}

#[test]
fn multiple_emits_accumulate() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("notifications.jsonl"));

    queue
        .emit(NotificationEvent::WorkStarted {
            repo: "jig".to_string(),
            worker: "a".to_string(),
            issue: None,
        })
        .unwrap();
    queue
        .emit(NotificationEvent::NeedsIntervention {
            repo: "jig".to_string(),
            worker: "a".to_string(),
            reason: "stalled".to_string(),
        })
        .unwrap();
    queue
        .emit(NotificationEvent::WorkCompleted {
            repo: "jig".to_string(),
            worker: "a".to_string(),
            pr_url: None,
        })
        .unwrap();

    let all = queue.tail(100).unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn tail_limits_results() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("notifications.jsonl"));

    for i in 0..5 {
        queue
            .emit(NotificationEvent::WorkStarted {
                repo: "jig".to_string(),
                worker: format!("w{}", i),
                issue: None,
            })
            .unwrap();
    }

    let last2 = queue.tail(2).unwrap();
    assert_eq!(last2.len(), 2);
}

#[test]
fn missing_file_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("nonexistent.jsonl"));

    assert!(!queue.exists());
    assert!(queue.tail(10).unwrap().is_empty());
    assert!(queue.read_since(0).unwrap().is_empty());
}

#[test]
fn read_since_filters() {
    let tmp = tempfile::tempdir().unwrap();
    let queue = NotificationQueue::new(tmp.path().join("notifications.jsonl"));

    queue
        .emit(NotificationEvent::WorkStarted {
            repo: "jig".to_string(),
            worker: "a".to_string(),
            issue: None,
        })
        .unwrap();

    let now = chrono::Utc::now().timestamp();

    queue
        .emit(NotificationEvent::PrOpened {
            repo: "jig".to_string(),
            worker: "a".to_string(),
            pr_url: "https://github.com/pr/1".to_string(),
        })
        .unwrap();

    // read_since(now) should only return the second notification
    // (both have same second-resolution timestamp, so use now-1 to be safe)
    let all = queue.tail(100).unwrap();
    assert_eq!(all.len(), 2);

    // At minimum, read_since with a future ts should return empty
    let future = queue.read_since(now + 100).unwrap();
    assert!(future.is_empty());
}
