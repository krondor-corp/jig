//! Session log retention.

use std::fs;

use crate::common::Sandbox;
use jig_cli::context::log;

/// Write `n` logs with sortable names, every other one empty.
fn seed(dir: &std::path::Path, n: usize) {
    fs::create_dir_all(dir).unwrap();
    for i in 0..n {
        let path = dir.join(format!("2026010{}T{:04}00Z.log", i / 10, i));
        fs::write(&path, if i % 2 == 0 { "" } else { "something\n" }).unwrap();
    }
}

#[test]
fn pruning_keeps_the_newest_logs_and_drops_the_empty_ones() {
    let sandbox = Sandbox::new();
    let dir = sandbox.paths().logs_dir();
    seed(&dir, 40);

    let removed = log::prune(&sandbox.paths(), 5);

    let left: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(left.len(), 5, "should keep exactly 5, kept {left:?}");
    assert_eq!(removed, 35);

    // The 5 kept are the newest with content — names sort by timestamp.
    let mut sorted = left.clone();
    sorted.sort();
    assert_eq!(sorted.last().unwrap(), "20260103T003900Z.log");
    assert!(
        left.iter()
            .all(|n| fs::metadata(dir.join(n)).unwrap().len() > 0),
        "kept an empty log: {left:?}"
    );
}

#[test]
fn the_live_daemons_log_is_never_removed() {
    let sandbox = Sandbox::new();
    let dir = sandbox.paths().logs_dir();
    seed(&dir, 40);

    // The oldest and emptiest file there is — deleting it out from under a
    // running daemon would strand everything it logged afterwards.
    let live = dir.join("20260000T000000Z.log");
    fs::write(&live, "").unwrap();

    log::prune_keeping(&sandbox.paths(), 5, Some(&live));

    assert!(live.exists(), "the log being written to must survive");
}

#[test]
fn pruning_a_directory_that_is_not_there_is_not_an_error() {
    let sandbox = Sandbox::new();
    // Nothing has created the logs directory yet.
    assert_eq!(log::prune(&sandbox.paths(), 5), 0);
}

#[test]
fn pruning_leaves_non_log_files_alone() {
    let sandbox = Sandbox::new();
    let dir = sandbox.paths().logs_dir();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("notes.txt"), "").unwrap();

    log::prune(&sandbox.paths(), 0);

    assert!(dir.join("notes.txt").exists());
}
