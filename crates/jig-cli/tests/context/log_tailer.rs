//! Following a log file as it grows — what `ps --watch` and
//! `jig daemon logs -f` do.

use std::io::Write;

use jig_cli::context::log::{tail_lines, LogTailer};

#[test]
fn tailer_returns_only_appended_complete_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("d.log");
    std::fs::write(&path, "old\n").unwrap();

    let mut tailer = LogTailer::from_end(path.clone());
    assert!(tailer.poll(10).is_empty());

    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    write!(f, "one\ntw").unwrap();
    assert_eq!(tailer.poll(10), vec!["one"]);

    writeln!(f, "o").unwrap();
    assert_eq!(tailer.poll(10), vec!["two"]);
}

#[test]
fn tailer_restarts_after_truncation() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("d.log");
    std::fs::write(&path, "a long first line\n").unwrap();
    let mut tailer = LogTailer::from_end(path.clone());

    std::fs::write(&path, "new\n").unwrap();
    assert_eq!(tailer.poll(10), vec!["new"]);
}

#[test]
fn tailer_waits_for_missing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("later.log");
    let mut tailer = LogTailer::from_end(path.clone());
    assert!(tailer.poll(10).is_empty());

    std::fs::write(&path, "hi\n").unwrap();
    assert_eq!(tailer.poll(10), vec!["hi"]);
}

#[test]
fn tail_lines_keeps_last_n() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("d.log");
    std::fs::write(&path, "1\n2\n3\n4\n").unwrap();
    assert_eq!(tail_lines(&path, 2).unwrap(), vec!["3", "4"]);
    assert_eq!(tail_lines(&path, 10).unwrap().len(), 4);
    assert!(tail_lines(&path, 0).unwrap().is_empty());
}
