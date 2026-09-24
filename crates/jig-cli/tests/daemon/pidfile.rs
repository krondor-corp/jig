//! The PID file that keeps one daemon per user.

use std::path::PathBuf;

use crate::common::Sandbox;
use jig_cli::context::AppPaths;
use jig_cli::daemon::pidfile::{PidFile, PidFileError};

/// A fresh sandbox, its paths, and where its pid file lives.
fn sandbox() -> (Sandbox, AppPaths, PathBuf) {
    let sandbox = Sandbox::new();
    let paths = sandbox.paths();
    let path = paths.pid_file();
    (sandbox, paths, path)
}

#[test]
fn acquire_writes_our_pid() {
    let (_sandbox, paths, _path) = sandbox();
    let held = PidFile::acquire(&paths).unwrap();
    assert_eq!(held.pid(), std::process::id());
    assert_eq!(
        PidFile::running_pid(&paths).unwrap(),
        Some(std::process::id()),
        "a held slot should report its pid as running"
    );
}

#[test]
fn drop_releases_the_slot() {
    let (_sandbox, paths, path) = sandbox();
    drop(PidFile::acquire(&paths).unwrap());
    assert!(!path.exists(), "drop should remove our own pid file");
    assert_eq!(PidFile::running_pid(&paths).unwrap(), None);
}

#[test]
fn a_live_foreign_pid_blocks_acquire() {
    let (_sandbox, paths, path) = sandbox();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    // PID 1 always exists and is never us.
    std::fs::write(&path, "1").unwrap();

    match PidFile::acquire(&paths) {
        Err(PidFileError::AlreadyRunning(1)) => {}
        other => panic!("expected AlreadyRunning(1), got {other:?}"),
    }
}

#[test]
fn a_stale_pid_is_taken_over() {
    let (_sandbox, paths, path) = sandbox();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    // Above the pid_max ceiling on every platform we run on, so it is
    // guaranteed to belong to nobody.
    std::fs::write(&path, "4294967290").unwrap();

    let held = PidFile::acquire(&paths).expect("a dead pid must not block a restart");
    assert_eq!(held.pid(), std::process::id());
}

#[test]
fn a_garbage_pid_file_is_taken_over() {
    let (_sandbox, paths, path) = sandbox();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "not a pid").unwrap();

    let held = PidFile::acquire(&paths).expect("an unparseable pid file must not wedge the daemon");
    assert_eq!(held.pid(), std::process::id());
}
