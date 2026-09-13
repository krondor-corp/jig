//! Where jig keeps its files.
//!
//! [`AppPaths`] is resolved from the environment once, in `main`, and passed
//! down. Nothing below `main` reads an XDG variable, so a test can point any
//! code path at temp directories by handing it an `AppPaths` of its own — no
//! process-wide `set_var`, no serialized tests.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Where jig keeps its own files — as opposed to the user's repos and
/// worktrees. Holds the config/state root and the runtime root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// `$XDG_CONFIG_HOME/jig`, else `~/.config/jig` — config, the repo
    /// registry, worker event logs, and `state/`.
    config: PathBuf,
    /// `$XDG_RUNTIME_DIR/jig`, else `<config>/state` — the daemon's socket
    /// and PID file, which must not outlive the session when the OS offers
    /// somewhere that doesn't.
    runtime: PathBuf,
}

impl AppPaths {
    /// Resolve from `XDG_CONFIG_HOME`, `XDG_RUNTIME_DIR` and the home
    /// directory. Called once, by `main`.
    pub fn from_env() -> Result<Self, std::io::Error> {
        Self::resolve(
            std::env::var_os("XDG_CONFIG_HOME"),
            std::env::var_os("XDG_RUNTIME_DIR"),
            dirs::home_dir(),
        )
    }

    /// The layout the binary would use if `XDG_CONFIG_HOME` and
    /// `XDG_RUNTIME_DIR` were these roots — how tests get the real layout
    /// under temp paths.
    pub fn under(config_home: &Path, runtime_home: &Path) -> Self {
        Self {
            config: config_home.join("jig"),
            runtime: runtime_home.join("jig"),
        }
    }

    /// [`Self::from_env`] with the inputs passed in. An empty XDG value
    /// counts as unset, as the spec requires.
    fn resolve(
        config_home: Option<OsString>,
        runtime_home: Option<OsString>,
        home: Option<PathBuf>,
    ) -> Result<Self, std::io::Error> {
        let set = |value: Option<OsString>| value.filter(|v| !v.is_empty()).map(PathBuf::from);
        let config = match set(config_home) {
            Some(dir) => dir.join("jig"),
            None => home
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "could not find home directory",
                    )
                })?
                .join(".config")
                .join("jig"),
        };
        let runtime = match set(runtime_home) {
            Some(dir) => dir.join("jig"),
            None => config.join("state"),
        };
        Ok(Self { config, runtime })
    }

    // ── config root ──────────────────────────────────────────────

    /// `~/.config/jig/`
    pub fn config_dir(&self) -> &Path {
        &self.config
    }

    /// `~/.config/jig/config.toml`
    pub fn config_file(&self) -> PathBuf {
        self.config.join("config.toml")
    }

    /// `~/.config/jig/hooks/`
    pub fn hooks_dir(&self) -> PathBuf {
        self.config.join("hooks")
    }

    /// `~/.config/jig/repos.json`
    pub fn repo_registry(&self) -> PathBuf {
        self.config.join("repos.json")
    }

    /// `~/.config/jig/<repo>/<branch>/` — a worker's event log lives here.
    pub fn worker_events_dir(&self, repo: &str, branch: &str) -> PathBuf {
        self.config.join(repo).join(branch)
    }

    // ── state ────────────────────────────────────────────────────

    /// `~/.config/jig/state/`
    pub fn state_dir(&self) -> PathBuf {
        self.config.join("state")
    }

    /// `~/.config/jig/state/daemon.jsonl` — daemon Started/Stopped events.
    pub fn daemon_lifecycle_log(&self) -> PathBuf {
        self.state_dir().join("daemon.jsonl")
    }

    /// `~/.config/jig/state/notifications.jsonl`
    pub fn notifications(&self) -> PathBuf {
        self.state_dir().join("notifications.jsonl")
    }

    /// `~/.config/jig/state/events/`
    pub fn events_dir(&self) -> PathBuf {
        self.state_dir().join("events")
    }

    /// `~/.config/jig/state/logs/`
    pub fn logs_dir(&self) -> PathBuf {
        self.state_dir().join("logs")
    }

    /// A new log file: `logs/<YYYYMMDDTHHMMSSZ>.log`. Only processes whose
    /// `Op::log_sink` is `File` write one.
    pub fn new_session_log(&self) -> PathBuf {
        let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        self.logs_dir().join(format!("{ts}.log"))
    }

    // ── runtime ──────────────────────────────────────────────────

    /// Where the daemon's socket and PID file live — see the field docs.
    pub fn runtime_dir(&self) -> &Path {
        &self.runtime
    }

    /// `<runtime>/daemon.sock`
    pub fn socket(&self) -> PathBuf {
        self.runtime.join("daemon.sock")
    }

    /// `<runtime>/daemon.pid` — beside the socket, so the pair is created
    /// and cleared together.
    pub fn pid_file(&self) -> PathBuf {
        self.runtime.join("daemon.pid")
    }

    /// Create every directory jig writes into.
    pub fn ensure(&self) -> Result<(), std::io::Error> {
        for dir in [
            self.config.clone(),
            self.hooks_dir(),
            self.state_dir(),
            self.events_dir(),
            self.logs_dir(),
            self.runtime.clone(),
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}

/// A test fixture's roots, laid out exactly as the binary would lay them out.
#[cfg(test)]
impl From<&jig_core::test_support::Fixture> for AppPaths {
    fn from(fixture: &jig_core::test_support::Fixture) -> Self {
        Self::under(fixture.config_home(), fixture.runtime_home())
    }
}

/// `<repo_root>/.jig/hooks/hooks.json`
pub fn hook_registry_path(repo_root: &Path) -> PathBuf {
    repo_root
        .join(super::JIG_DIR)
        .join("hooks")
        .join("hooks.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> Option<PathBuf> {
        Some(PathBuf::from("/home/me"))
    }

    #[test]
    fn xdg_roots_win_over_the_fallbacks() {
        let paths =
            AppPaths::resolve(Some("/cfg".into()), Some("/run/user/501".into()), home()).unwrap();
        assert_eq!(paths.config_dir(), Path::new("/cfg/jig"));
        assert_eq!(paths.runtime_dir(), Path::new("/run/user/501/jig"));
    }

    #[test]
    fn unset_or_empty_xdg_values_use_the_fallbacks() {
        for (config, runtime) in [(None, None), (Some(OsString::new()), Some(OsString::new()))] {
            let paths = AppPaths::resolve(config, runtime, home()).unwrap();
            assert_eq!(paths.config_dir(), Path::new("/home/me/.config/jig"));
            assert_eq!(paths.runtime_dir(), Path::new("/home/me/.config/jig/state"));
        }
    }

    #[test]
    fn no_home_and_no_config_home_is_an_error() {
        assert!(AppPaths::resolve(None, None, None).is_err());
    }

    #[test]
    fn under_matches_what_the_env_would_resolve() {
        let from_env = AppPaths::resolve(Some("/c".into()), Some("/r".into()), None).unwrap();
        assert_eq!(AppPaths::under(Path::new("/c"), Path::new("/r")), from_env);
    }

    #[test]
    fn ensure_creates_every_dir() {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::under(&root.path().join("config"), &root.path().join("run"));
        paths.ensure().unwrap();
        for dir in [
            paths.config_dir().to_path_buf(),
            paths.hooks_dir(),
            paths.state_dir(),
            paths.events_dir(),
            paths.logs_dir(),
            paths.runtime_dir().to_path_buf(),
        ] {
            assert!(dir.is_dir(), "missing {}", dir.display());
        }
    }
}
