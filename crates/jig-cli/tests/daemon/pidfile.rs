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

/// KRO-221. macOS has no `XDG_RUNTIME_DIR`, so the pid file lives under
/// `~/.config` and survives a reboot — and the recorded PID is then very
/// likely to have been handed to something else entirely. `kill(pid, 0)`
/// cannot tell the difference, so the daemon refused to start until someone
/// deleted the file by hand.
#[test]
fn a_claim_from_before_a_reboot_does_not_block_the_daemon() {
    let (_sandbox, paths, path) = sandbox();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    // pid 1 is always alive and is emphatically not our daemon. The second
    // field is the boot the claim was made in — 1970, i.e. some boot ago.
    std::fs::write(&path, "1 1").unwrap();

    let held =
        PidFile::acquire(&paths).expect("a claim from an earlier boot must not block a restart");
    assert_eq!(held.pid(), std::process::id());
}

#[test]
fn a_live_claim_from_this_boot_still_blocks_a_second_daemon() {
    let (_sandbox, paths, path) = sandbox();
    let held = PidFile::acquire(&paths).unwrap();

    // Take the boot stamp jig just wrote and hand the claim to pid 1, which
    // is alive and is not us. The boot check must not have loosened the gate.
    let written = std::fs::read_to_string(&path).unwrap();
    let boot = written
        .split_whitespace()
        .nth(1)
        .expect("a fresh claim should record its boot");
    std::fs::write(&path, format!("1 {boot}")).unwrap();

    match PidFile::acquire(&paths) {
        Err(PidFileError::AlreadyRunning(pid)) => assert_eq!(pid, 1),
        other => panic!("expected AlreadyRunning, got {other:?}"),
    }
    drop(held);
}
