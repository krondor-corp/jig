//! Shared setup for jig's integration tests.
//!
//! A [`Sandbox`] wraps [`jig_core::test_support::Fixture`] — its own
//! `XDG_CONFIG_HOME`, `XDG_RUNTIME_DIR` and any number of repos — and passes
//! those roots to every `jig` it runs, so tests — and the daemons they
//! start — run in parallel without seeing each other or the developer's real
//! `~/.config/jig` and daemon socket. Nothing here changes the test
//! process's own environment or working directory; that is what keeps
//! `cargo test` safe to parallelize.
//!
//! Lives at `tests/common/mod.rs` (not `tests/common.rs`) so cargo doesn't
//! build it as a test binary of its own. Pull it in with `mod common;`.

// Each test binary uses a different subset of these helpers.
#![allow(dead_code)]
// `Command::cargo_bin` is deprecated but used across the test suite.
#![allow(deprecated)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command as StdCommand, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::Command;
use jig_core::test_support::Fixture;
use tempfile::TempDir;

/// A [`Fixture`] (config root, runtime root, any number of repos) plus the
/// means to run the `jig` binary against it.
pub struct Sandbox {
    fixture: Fixture,
    /// Where commands run when the sandbox has no repos.
    scratch: TempDir,
}

impl Sandbox {
    /// No repos; commands run in an empty directory that isn't a git repo.
    pub fn new() -> Self {
        Self {
            fixture: Fixture::new(),
            scratch: TempDir::new().expect("scratch dir"),
        }
    }

    /// `n` repos (see [`jig_core::test_support::init_repo`]); commands run
    /// in the first.
    pub fn with_repos(n: usize) -> Self {
        let mut sandbox = Self::new();
        for _ in 0..n {
            sandbox.add_repo();
        }
        let legacy_config = sandbox.jig_dir().join("config");
        std::fs::create_dir_all(legacy_config.parent().unwrap()).unwrap();
        std::fs::write(&legacy_config, "_default=main\n").unwrap();
        sandbox
    }

    /// One repo, which commands run in.
    pub fn with_repo() -> Self {
        Self::with_repos(1)
    }

    /// Add another repo and return its path.
    pub fn add_repo(&mut self) -> PathBuf {
        self.fixture.add_repo().to_path_buf()
    }

    /// The `i`th repo, in the order they were added.
    pub fn repo(&self, i: usize) -> &Path {
        self.fixture.repo(i)
    }

    /// Where [`Self::jig`] runs: the first repo, else the scratch dir.
    pub fn work_dir(&self) -> &Path {
        self.fixture.repos().next().unwrap_or(self.scratch.path())
    }

    /// `$XDG_CONFIG_HOME/jig` — config, registry and `state/`.
    pub fn jig_dir(&self) -> PathBuf {
        self.fixture.config_home().join("jig")
    }

    /// `$XDG_RUNTIME_DIR/jig` — the daemon's socket and PID file.
    pub fn runtime_jig_dir(&self) -> PathBuf {
        self.fixture.runtime_home().join("jig")
    }

    pub fn socket(&self) -> PathBuf {
        self.runtime_jig_dir().join("daemon.sock")
    }

    pub fn pid_file(&self) -> PathBuf {
        self.runtime_jig_dir().join("daemon.pid")
    }

    /// Replace the global `config.toml`.
    pub fn write_global_config(&self, toml: &str) {
        let path = self.jig_dir().join("config.toml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, toml).unwrap();
    }

    /// Run `git` in the working directory.
    pub fn git(&self, args: &[&str]) {
        let status = StdCommand::new("git")
            .args(args)
            .current_dir(self.work_dir())
            .output()
            .expect("run git")
            .status;
        assert!(status.success(), "git {args:?} failed");
    }

    /// An empty commit in the working-directory repo.
    pub fn commit(&self, message: &str) {
        self.git(&["commit", "--allow-empty", "-m", message, "-q"]);
    }

    /// `jig`, pointed at this sandbox, running in [`Self::work_dir`].
    pub fn jig(&self) -> Command {
        self.jig_in(self.work_dir())
    }

    /// `jig`, pointed at this sandbox, running in `dir` (e.g. another repo).
    pub fn jig_in(&self, dir: &Path) -> Command {
        let mut cmd = Command::cargo_bin("jig").expect("jig binary");
        cmd.env("XDG_CONFIG_HOME", self.fixture.config_home())
            .env("XDG_RUNTIME_DIR", self.fixture.runtime_home())
            .current_dir(dir);
        cmd
    }

    /// `jig <args>` as a background process, output discarded.
    pub fn spawn_jig(&self, args: &[&str]) -> Child {
        StdCommand::new(assert_cmd::cargo::cargo_bin("jig"))
            .args(args)
            .env("XDG_CONFIG_HOME", self.fixture.config_home())
            .env("XDG_RUNTIME_DIR", self.fixture.runtime_home())
            .current_dir(self.work_dir())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("spawn jig {args:?}: {e}"))
    }

    /// Start a background daemon and wait until it answers.
    pub fn start_daemon(&self) -> Daemon {
        let child = self.spawn_jig(&["daemon", "start"]);
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
pub struct Daemon(Child);

impl Daemon {
    pub fn wait_for_exit(&mut self) {
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

/// Poll `ready` every 50ms, failing the test after 30s.
pub fn wait_for(mut ready: impl FnMut() -> bool, what: &str) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(30) {
        if ready() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("timed out waiting for {what}");
}

/// The PID recorded in a PID file.
pub fn pid_in(path: &Path) -> u32 {
    std::fs::read_to_string(path)
        .expect("pid file")
        .trim()
        .parse()
        .expect("pid file holds a number")
}
