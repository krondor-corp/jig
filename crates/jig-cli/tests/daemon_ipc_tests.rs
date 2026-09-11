#![allow(deprecated)] // Command::cargo_bin is deprecated but used across tests

//! Daemon lifecycle over the unix socket: single instance, IPC status, stop,
//! and recovery from a socket a crashed daemon left behind.
//!
//! Every test gets its own `XDG_CONFIG_HOME` *and* `XDG_RUNTIME_DIR`, so the
//! daemons they start cannot see each other or the developer's own.

use std::path::{Path, PathBuf};
use std::process::{Child, Command as StdCommand, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// An isolated config + runtime root for one test's daemons.
struct Sandbox {
    config: TempDir,
    runtime: TempDir,
}

impl Sandbox {
    fn new() -> Self {
        Self {
            config: TempDir::new().expect("config dir"),
            runtime: TempDir::new().expect("runtime dir"),
        }
    }

    fn jig(&self) -> Command {
        let mut cmd = Command::cargo_bin("jig").expect("jig binary");
        cmd.env("XDG_CONFIG_HOME", self.config.path());
        cmd.env("XDG_RUNTIME_DIR", self.runtime.path());
        cmd.current_dir(self.config.path());
        cmd
    }

    fn socket(&self) -> PathBuf {
        self.runtime.path().join("jig").join("daemon.sock")
    }

    fn pid_file(&self) -> PathBuf {
        self.runtime.path().join("jig").join("daemon.pid")
    }

    /// Start a background daemon and wait until it answers.
    fn start_daemon(&self) -> Daemon {
        let child = StdCommand::new(assert_cmd::cargo::cargo_bin("jig"))
            .args(["daemon", "start"])
            .env("XDG_CONFIG_HOME", self.config.path())
            .env("XDG_RUNTIME_DIR", self.runtime.path())
            .current_dir(self.config.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn daemon");

        wait_for(|| self.socket().exists(), "daemon socket to appear");
        // The socket file exists a moment before the listener answers.
        wait_for(
            || {
                self.jig()
                    .args(["daemon", "status"])
                    .output()
                    .unwrap()
                    .status
                    .success()
            },
            "daemon to answer status",
        );
        Daemon(child)
    }
}

/// A background daemon, killed if a test fails before stopping it.
struct Daemon(Child);

impl Daemon {
    fn wait_for_exit(&mut self) {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            if matches!(self.0.try_wait(), Ok(Some(_))) {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("daemon did not exit within 10s");
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn wait_for(mut ready: impl FnMut() -> bool, what: &str) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(30) {
        if ready() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("timed out waiting for {what}");
}

fn pid_in(path: &Path) -> u32 {
    std::fs::read_to_string(path)
        .expect("pid file")
        .trim()
        .parse()
        .expect("pid file holds a number")
}

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
        !sandbox.socket().exists(),
        "a clean exit must unlink its socket"
    );
    assert!(
        !sandbox.pid_file().exists(),
        "a clean exit must release its pid file"
    );
}

#[test]
fn a_second_daemon_refuses_to_start() {
    let sandbox = Sandbox::new();
    let mut daemon = sandbox.start_daemon();
    let pid = pid_in(&sandbox.pid_file());

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
    let pid = pid_in(&sandbox.pid_file());

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
    assert!(!sandbox.socket().exists(), "stop must unlink the socket");
    assert!(
        !sandbox.pid_file().exists(),
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
    let dir = sandbox.runtime.path().join("jig");
    std::fs::create_dir_all(&dir).unwrap();

    // What SIGKILL leaves: a socket file nobody is listening on, and a pid
    // file naming a process that is gone.
    std::os::unix::net::UnixListener::bind(sandbox.socket()).unwrap();
    std::fs::write(sandbox.pid_file(), "4294967290").unwrap();

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
