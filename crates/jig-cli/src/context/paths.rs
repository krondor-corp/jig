//! Path helpers — all jig directories and file paths.

use std::ffi::OsString;
use std::path::PathBuf;

/// `$<var>/jig` when the XDG base-directory variable `var` is set, else
/// `fallback()`. The single place jig reads an XDG variable.
fn xdg_jig_dir(
    var: &str,
    fallback: impl FnOnce() -> Result<PathBuf, std::io::Error>,
) -> Result<PathBuf, std::io::Error> {
    jig_dir_from(std::env::var_os(var), fallback)
}

/// [`xdg_jig_dir`] with the variable's value passed in, so tests can cover
/// the resolution without touching the process environment. An empty value
/// counts as unset, as the XDG spec requires.
fn jig_dir_from(
    value: Option<OsString>,
    fallback: impl FnOnce() -> Result<PathBuf, std::io::Error>,
) -> Result<PathBuf, std::io::Error> {
    match value {
        Some(dir) if !dir.is_empty() => Ok(PathBuf::from(dir).join("jig")),
        _ => fallback(),
    }
}

/// `$XDG_CONFIG_HOME/jig/`, else `~/.config/jig/`
pub fn global_config_dir() -> Result<PathBuf, std::io::Error> {
    xdg_jig_dir("XDG_CONFIG_HOME", || {
        let home = dirs::home_dir().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "could not find home directory",
            )
        })?;
        Ok(home.join(".config").join("jig"))
    })
}

/// `~/.config/jig/config.toml`
pub fn global_config_path() -> Result<PathBuf, std::io::Error> {
    Ok(global_config_dir()?.join("config.toml"))
}

/// `~/.config/jig/hooks/`
pub fn global_hooks_dir() -> Result<PathBuf, std::io::Error> {
    Ok(global_config_dir()?.join("hooks"))
}

/// `<repo_root>/.jig/hooks/hooks.json`
pub fn hook_registry_path(repo_root: &std::path::Path) -> PathBuf {
    repo_root
        .join(super::JIG_DIR)
        .join("hooks")
        .join("hooks.json")
}

/// `~/.config/jig/state/`
pub fn global_state_dir() -> Result<PathBuf, std::io::Error> {
    Ok(global_config_dir()?.join("state"))
}

/// `~/.config/jig/state/daemon.jsonl`
pub fn daemon_log_path() -> Result<PathBuf, std::io::Error> {
    Ok(global_state_dir()?.join("daemon.jsonl"))
}

/// Where the daemon's socket and PID file live.
///
/// `$XDG_RUNTIME_DIR/jig/` when the session has one — it is user-private,
/// on tmpfs, and cleared on logout, so a reboot can never leave a stale
/// socket behind. Falls back to `~/.config/jig/state/` otherwise (macOS
/// sets no `XDG_RUNTIME_DIR`).
pub fn daemon_runtime_dir() -> Result<PathBuf, std::io::Error> {
    xdg_jig_dir("XDG_RUNTIME_DIR", global_state_dir)
}

/// `$XDG_RUNTIME_DIR/jig/daemon.sock` (or `~/.config/jig/state/daemon.sock`)
pub fn daemon_socket_path() -> Result<PathBuf, std::io::Error> {
    Ok(daemon_runtime_dir()?.join("daemon.sock"))
}

/// `$XDG_RUNTIME_DIR/jig/daemon.pid` (or `~/.config/jig/state/daemon.pid`)
///
/// Lives beside the socket so the pair is created and cleared together.
pub fn daemon_pid_path() -> Result<PathBuf, std::io::Error> {
    Ok(daemon_runtime_dir()?.join("daemon.pid"))
}

/// `~/.config/jig/<repo>/<branch>/`
pub fn worker_events_dir(repo: &str, branch: &str) -> Result<PathBuf, std::io::Error> {
    Ok(global_config_dir()?.join(repo).join(branch))
}

/// `~/.config/jig/repos.json`
pub fn repo_registry_path() -> Result<PathBuf, std::io::Error> {
    Ok(global_config_dir()?.join("repos.json"))
}

/// `~/.config/jig/state/notifications.jsonl`
pub fn notifications_path() -> Result<PathBuf, std::io::Error> {
    Ok(global_state_dir()?.join("notifications.jsonl"))
}

/// `~/.config/jig/state/triages.json`
pub fn triages_path() -> Result<PathBuf, std::io::Error> {
    Ok(global_state_dir()?.join("triages.json"))
}

/// `~/.config/jig/state/events/`
pub fn global_events_dir() -> Result<PathBuf, std::io::Error> {
    Ok(global_state_dir()?.join("events"))
}

/// `~/.config/jig/state/logs/`
pub fn daemon_logs_dir() -> Result<PathBuf, std::io::Error> {
    Ok(global_state_dir()?.join("logs"))
}

const DAEMON_LOG_SUFFIX: &str = "-daemon.log";

/// New log path for a one-off command: `~/.config/jig/state/logs/<YYYYMMDDTHHMMSSZ>.log`
pub fn new_session_log_path() -> Result<PathBuf, std::io::Error> {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    Ok(daemon_logs_dir()?.join(format!("{}.log", ts)))
}

/// New log path for a daemon run: `~/.config/jig/state/logs/<YYYYMMDDTHHMMSSZ>-daemon.log`
///
/// The suffix keeps daemon logs findable among one-off command logs.
pub fn new_daemon_log_path() -> Result<PathBuf, std::io::Error> {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    Ok(daemon_logs_dir()?.join(format!("{}{}", ts, DAEMON_LOG_SUFFIX)))
}

/// Find the most recent daemon run's log (lexicographic sort on ISO timestamps).
pub fn latest_daemon_log() -> Result<Option<PathBuf>, std::io::Error> {
    let dir = daemon_logs_dir()?;
    if !dir.exists() {
        return Ok(None);
    }
    let mut logs: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().ends_with(DAEMON_LOG_SUFFIX))
        })
        .collect();
    logs.sort();
    Ok(logs.pop())
}

/// Create all global directories.
pub fn ensure_global_dirs() -> Result<(), std::io::Error> {
    let dirs = [
        global_config_dir()?,
        global_hooks_dir()?,
        global_state_dir()?,
        global_events_dir()?,
        daemon_logs_dir()?,
        daemon_runtime_dir()?,
    ];
    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_ends_with_jig() {
        let dir = global_config_dir().unwrap();
        assert!(dir.ends_with("jig"));
    }

    #[test]
    fn xdg_value_wins_over_the_fallback() {
        let dir = jig_dir_from(Some("/run/user/501".into()), || panic!("not used")).unwrap();
        assert_eq!(dir, PathBuf::from("/run/user/501/jig"));
    }

    #[test]
    fn unset_or_empty_xdg_value_uses_the_fallback() {
        for value in [None, Some(OsString::new())] {
            let dir = jig_dir_from(value, || Ok(PathBuf::from("/fallback"))).unwrap();
            assert_eq!(dir, PathBuf::from("/fallback"));
        }
    }
}
