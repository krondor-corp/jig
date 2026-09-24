//! Daemon lifecycle over the unix socket: single instance, IPC status, stop,
//! and recovery from a socket a crashed daemon left behind.
//!
//! Every test gets its own `XDG_CONFIG_HOME` *and* `XDG_RUNTIME_DIR`, so the
//! daemons they start cannot see each other or the developer's own.

mod common;

use std::time::Duration;

use common::{pid_in, Sandbox};
use predicates::prelude::*;

#[test]
fn status_without_a_daemon_says_not_running() {
    let sandbox = Sandbox::new();
    sandbox
        .jig()
        .args(["daemon", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("daemon not running"));
}

#[test]
fn stop_without_a_daemon_says_not_running() {
    let sandbox = Sandbox::new();
    sandbox
        .jig()
        .args(["daemon", "stop"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("daemon not running"));
}

#[test]
fn start_once_runs_a_tick_and_cleans_up() {
    let sandbox = Sandbox::new();
    sandbox
        .jig()
        .args(["daemon", "start", "--once"])
        .assert()
        .success()
        .stderr(predicate::str::contains("daemon started"));

    assert!(
        !sandbox.paths().socket().exists(),
        "a clean exit must unlink its socket"
    );
    assert!(
        !sandbox.paths().pid_file().exists(),
        "a clean exit must release its pid file"
    );
}

#[test]
fn a_second_daemon_refuses_to_start() {
    let sandbox = Sandbox::new();
    let mut daemon = sandbox.start_daemon();
    let pid = pid_in(&sandbox.paths().pid_file());

    sandbox
        .jig()
        .args(["daemon", "start", "--once"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "daemon already running (pid {pid})"
        )));

    sandbox.jig().args(["daemon", "stop"]).assert().success();
    daemon.wait_for_exit();
}

#[test]
fn status_answers_over_the_socket() {
    let sandbox = Sandbox::new();
    let mut daemon = sandbox.start_daemon();
    let pid = pid_in(&sandbox.paths().pid_file());

    sandbox
        .jig()
        .args(["daemon", "status"])
        .assert()
        .success()
        .stderr(predicate::str::contains("daemon running"))
        .stderr(predicate::str::contains(format!("pid {pid}")))
        .stderr(predicate::str::contains("jig-monitor"));

    sandbox.jig().args(["daemon", "stop"]).assert().success();
    daemon.wait_for_exit();
}

#[test]
fn stop_shuts_the_daemon_down_and_clears_its_files() {
    let sandbox = Sandbox::new();
    let mut daemon = sandbox.start_daemon();

    sandbox
        .jig()
        .args(["daemon", "stop"])
        .assert()
        .success()
        .stderr(predicate::str::contains("daemon stopping"));

    daemon.wait_for_exit();
    assert!(
        !sandbox.paths().socket().exists(),
        "stop must unlink the socket"
    );
    assert!(
        !sandbox.paths().pid_file().exists(),
        "stop must release the pid file"
    );

    sandbox
        .jig()
        .args(["daemon", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("daemon not running"));
}

#[test]
fn a_socket_left_by_a_crashed_daemon_does_not_block_a_restart() {
    let sandbox = Sandbox::new();
    let dir = sandbox.paths().runtime_dir().to_path_buf();
    std::fs::create_dir_all(&dir).unwrap();

    // What SIGKILL leaves: a socket file nobody is listening on, and a pid
    // file naming a process that is gone.
    std::os::unix::net::UnixListener::bind(sandbox.paths().socket()).unwrap();
    std::fs::write(sandbox.paths().pid_file(), "4294967290").unwrap();

    sandbox
        .jig()
        .args(["daemon", "start", "--once"])
        .assert()
        .success()
        .stderr(predicate::str::contains("daemon started"));
}

#[test]
fn ps_without_a_daemon_still_works() {
    let sandbox = Sandbox::new();
    sandbox
        .jig()
        .args(["ps", "-g"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No spawned sessions"));
}

/// A `ps --watch` client writes a log of its own, newer than the daemon's.
/// After the daemon exits, `jig daemon logs` must still resolve to the
/// daemon's log — which it finds through the daemon's `Started` event, not
/// by picking the newest file.
#[test]
fn a_watching_client_does_not_shadow_the_daemon_log() {
    let sandbox = Sandbox::new();
    let mut daemon = sandbox.start_daemon();

    let real_log = String::from_utf8(
        sandbox
            .jig()
            .args(["daemon", "logs", "--path"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    assert!(real_log.ends_with(".log"), "got {real_log:?}");

    // Run `ps -gw` against the daemon, then kill it the way a user's ctrl-c
    // would not (no cleanup), which is the harshest case for stray files.
    let mut client = sandbox.spawn_jig(&["ps", "-gw"]);
    std::thread::sleep(Duration::from_millis(1500));
    let _ = client.kill();
    let _ = client.wait();

    sandbox.jig().args(["daemon", "stop"]).assert().success();
    daemon.wait_for_exit();

    let resolved = String::from_utf8(
        sandbox
            .jig()
            .args(["daemon", "logs", "--path"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    assert_eq!(
        resolved, real_log,
        "after the daemon exits, `jig daemon logs` must still resolve to its \
         log, not to one a watching client left behind"
    );
}

#[test]
fn ps_reads_from_a_running_daemon() {
    let sandbox = Sandbox::new();
    let mut daemon = sandbox.start_daemon();

    // No repos are registered, so the frame is empty either way — what is
    // under test is that `ps` takes the IPC path without erroring.
    sandbox
        .jig()
        .args(["ps", "-g"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No spawned sessions"));

    sandbox.jig().args(["daemon", "stop"]).assert().success();
    daemon.wait_for_exit();
}
