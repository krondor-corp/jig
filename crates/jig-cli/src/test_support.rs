//! Fixtures for jig's tests.
//!
//! A [`Sandbox`] owns temp config and runtime roots plus any number of git
//! repos, and serves both kinds of test: [`Sandbox::paths`] for calling
//! jig's code in-process, [`Sandbox::jig`] for driving the binary. Nothing
//! here touches the test process's own environment or working directory, so
//! tests run in parallel — including the daemons they start, which get a
//! socket of their own.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use tempfile::TempDir;

use crate::context::AppPaths;

/// Temp roots and repos for one test. Everything is removed on drop.
pub struct Sandbox {
    config: TempDir,
    runtime: TempDir,
    repos: Vec<TempDir>,
    /// Where commands run when the sandbox has no repos.
    scratch: TempDir,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Sandbox {
    /// Config and runtime roots, no repos.
    pub fn new() -> Self {
        Self {
            config: TempDir::new().expect("config root"),
            runtime: TempDir::new().expect("runtime root"),
            repos: Vec::new(),
            scratch: TempDir::new().expect("scratch dir"),
        }
    }

    /// Config and runtime roots plus `n` repos (see [`Self::add_repo`]).
    pub fn with_repos(n: usize) -> Self {
        let mut sandbox = Self::new();
        for _ in 0..n {
            sandbox.add_repo();
        }
        sandbox
    }

    /// One repo, which [`Self::jig`] runs in.
    pub fn with_repo() -> Self {
        Self::with_repos(1)
    }

    /// Add a git repo on `main` with one empty commit, and return its path.
    ///
    /// The identity and signing settings are local to it, so commits work
    /// regardless of the developer's global git config.
    pub fn add_repo(&mut self) -> PathBuf {
        let dir = TempDir::new().expect("repo dir");
        let repo = jig_core::git::Repo::init(dir.path()).expect("git init");
        let inner = repo.inner();
        {
            let mut config = inner.config().expect("repo config");
            config.set_str("user.email", "test@test.com").unwrap();
            config.set_str("user.name", "Test").unwrap();
            config.set_bool("commit.gpgsign", false).unwrap();
        }
        let tree_oid = inner.index().unwrap().write_tree().unwrap();
        let tree = inner.find_tree(tree_oid).unwrap();
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        inner
            .commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .expect("initial commit");

        let path = dir.path().to_path_buf();
        self.repos.push(dir);
        path
    }

    /// The `i`th repo, in the order they were added.
    pub fn repo(&self, i: usize) -> &Path {
        self.repos[i].path()
    }

    /// Every repo, in the order they were added.
    pub fn repos(&self) -> impl Iterator<Item = &Path> {
        self.repos.iter().map(|d| d.path())
    }

    /// jig's paths under this sandbox — the same layout the binary would
    /// resolve from the roots below. For calling jig's code in-process.
    pub fn paths(&self) -> AppPaths {
        AppPaths::under(self.config.path(), self.runtime.path())
    }

    /// What `XDG_CONFIG_HOME` is for commands this sandbox runs.
    pub fn config_home(&self) -> &Path {
        self.config.path()
    }

    /// What `XDG_RUNTIME_DIR` is for commands this sandbox runs.
    pub fn runtime_home(&self) -> &Path {
        self.runtime.path()
    }

    /// Where [`Self::jig`] runs: the first repo, else an empty directory
    /// that is not a git repo.
    pub fn work_dir(&self) -> &Path {
        self.repos.first().map_or(self.scratch.path(), |d| d.path())
    }

    /// Replace the global `config.toml`.
    pub fn write_global_config(&self, toml: &str) {
        let path = self.paths().config_file();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, toml).unwrap();
    }

    /// Run `git` in the working directory.
    pub fn git(&self, args: &[&str]) {
        let status = Command::new("git")
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

    /// The `jig` binary, pointed at this sandbox, running in
    /// [`Self::work_dir`].
    pub fn jig(&self) -> assert_cmd::Command {
        self.jig_in(self.work_dir())
    }

    /// The `jig` binary, pointed at this sandbox, running in `dir`.
    pub fn jig_in(&self, dir: &Path) -> assert_cmd::Command {
        #[allow(deprecated)] // `cargo_bin` is deprecated but has no stable replacement
        let mut cmd = assert_cmd::Command::cargo_bin("jig").expect("jig binary");
        cmd.env("XDG_CONFIG_HOME", self.config.path())
            .env("XDG_RUNTIME_DIR", self.runtime.path())
            .current_dir(dir);
        cmd
    }

    /// `jig <args>` as a background process, output discarded.
    pub fn spawn_jig(&self, args: &[&str]) -> Child {
        #[allow(deprecated)]
        let bin = assert_cmd::cargo::cargo_bin("jig");
        Command::new(bin)
            .args(args)
            .env("XDG_CONFIG_HOME", self.config.path())
            .env("XDG_RUNTIME_DIR", self.runtime.path())
            .current_dir(self.work_dir())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("spawn jig {args:?}: {e}"))
    }

    /// Start a background daemon and wait until it answers.
    pub fn start_daemon(&self) -> Daemon {
        let child = self.spawn_jig(&["daemon", "start"]);
        let socket = self.paths().socket();
        wait_for(|| socket.exists(), "daemon socket to appear");
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
