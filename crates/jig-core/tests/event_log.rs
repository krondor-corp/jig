//! The append-only JSONL event log.

use jig_core::{Event, EventLog, Reducible};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event")]
enum TestEvent {
    Started { ts: i64 },
    Stopped { ts: i64 },
}

#[test]
fn append_and_read_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let log: EventLog<TestEvent> = EventLog::new(tmp.path().join("test.jsonl"));

    log.append(&TestEvent::Started { ts: 1000 }).unwrap();
    log.append(&TestEvent::Stopped { ts: 2000 }).unwrap();

    let events = log.read_all().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], TestEvent::Started { ts: 1000 });
    assert_eq!(events[1], TestEvent::Stopped { ts: 2000 });
}

#[test]
fn last_event_returns_final() {
    let tmp = tempfile::tempdir().unwrap();
    let log: EventLog<TestEvent> = EventLog::new(tmp.path().join("test.jsonl"));

    log.append(&TestEvent::Started { ts: 1000 }).unwrap();
    log.append(&TestEvent::Stopped { ts: 2000 }).unwrap();

    let last = log.last_event().unwrap().unwrap();
    assert_eq!(last, TestEvent::Stopped { ts: 2000 });
}

#[test]
fn missing_file_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let log: EventLog<TestEvent> = EventLog::new(tmp.path().join("nope.jsonl"));

    assert!(!log.exists());
    assert!(log.read_all().unwrap().is_empty());
    assert!(log.last_event().unwrap().is_none());
}

#[test]
fn reset_clears_events() {
    let tmp = tempfile::tempdir().unwrap();
    let log: EventLog<TestEvent> = EventLog::new(tmp.path().join("test.jsonl"));

    log.append(&TestEvent::Started { ts: 1000 }).unwrap();
    assert_eq!(log.read_all().unwrap().len(), 1);

    log.reset().unwrap();
    assert!(log.exists());
    assert!(log.read_all().unwrap().is_empty());
}

#[test]
fn remove_deletes_file() {
    let tmp = tempfile::tempdir().unwrap();
    let subdir = tmp.path().join("sub");
    std::fs::create_dir_all(&subdir).unwrap();
    let log: EventLog<TestEvent> = EventLog::new(subdir.join("test.jsonl"));

    log.append(&TestEvent::Started { ts: 1000 }).unwrap();
    assert!(log.exists());

    log.remove().unwrap();
    assert!(!log.exists());
    assert!(!subdir.exists());
}

#[derive(Debug, Default, PartialEq)]
struct Counter {
    started: u32,
    stopped: u32,
}

impl Reducible for TestEvent {
    type State = Counter;
    fn apply(state: &mut Counter, event: &TestEvent) {
        match event {
            TestEvent::Started { .. } => state.started += 1,
            TestEvent::Stopped { .. } => state.stopped += 1,
        }
    }
}

#[test]
fn reduce_folds_events() {
    let tmp = tempfile::tempdir().unwrap();
    let log: EventLog<TestEvent> = EventLog::new(tmp.path().join("test.jsonl"));

    log.append(&TestEvent::Started { ts: 1 }).unwrap();
    log.append(&TestEvent::Stopped { ts: 2 }).unwrap();
    log.append(&TestEvent::Started { ts: 3 }).unwrap();

    let state = log.reduce().unwrap();
    assert_eq!(
        state,
        Counter {
            started: 2,
            stopped: 1
        }
    );
}

#[test]
fn event_flatten_roundtrip() {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum Kind {
        Started { pid: u32 },
    }

    let event = Event::now(Kind::Started { pid: 42 });
    let json = serde_json::to_string(&event).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed["ts"].is_i64());
    assert_eq!(parsed["type"], "started");
    assert_eq!(parsed["pid"], 42);
    assert!(parsed.get("kind").is_none());

    let restored: Event<Kind> = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.kind, Kind::Started { pid: 42 });
}

#[test]
fn reduce_empty_log_returns_default() {
    let tmp = tempfile::tempdir().unwrap();
    let log: EventLog<TestEvent> = EventLog::new(tmp.path().join("nope.jsonl"));

    let state = log.reduce().unwrap();
    assert_eq!(state, Counter::default());
}
