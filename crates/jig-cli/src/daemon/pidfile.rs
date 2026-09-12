//! PID file — one daemon per user, enforced.
//!
//! The daemon is always global (it watches every tracked repo), so two of
//! them fight over the same worktrees, nudges and spawn slots. Claiming this
//! file is the gate: `acquire()` succeeds for exactly one live process and
//! tells the loser which PID already holds it.
//!
//! A daemon that is killed or OOMs leaves the file behind. That is fine — a
//! PID whose process is gone is a *stale* claim, and the next `acquire()`
//! takes it over. Nothing has to clean up after a crash for the daemon to
//! restart.

use std::io::Write;
use std::path::PathBuf;

use crate::context::daemon_pid_path;

#[derive(Debug, thiserror::Error)]
pub enum PidFileError {
    #[error("daemon already running (pid {0})")]
    AlreadyRunning(u32),
    #[error("failed to claim the daemon pid file: {0}")]
    Io(#[from] std::io::Error),
}

/// A held claim on the single-daemon slot. Releases on drop.
#[derive(Debug)]
pub struct PidFile {
    path: PathBuf,
    pid: u32,
}

impl PidFile {
    /// Claim the slot for this process, taking over a stale claim.
    pub fn acquire() -> Result<Self, PidFileError> {
        let path = daemon_pid_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let pid = std::process::id();

        // `create_new` is the atomic part: two daemons racing here, only one
        // creates the file. The loser falls through to the liveness check.
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                write!(file, "{pid}")?;
                return Ok(Self { path, pid });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }

        if let Some(holder) = read_live_pid(&path)? {
            if holder != pid {
                return Err(PidFileError::AlreadyRunning(holder));
            }
        }

        // Stale (or ours): take it over.
        let mut file = std::fs::File::create(&path)?;
        write!(file, "{pid}")?;
        Ok(Self { path, pid })
    }

    /// PID of the daemon currently holding the slot, if one is alive.
    pub fn running_pid() -> std::io::Result<Option<u32>> {
        read_live_pid(&daemon_pid_path()?)
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        // Only clear our own claim — a daemon that took over after we went
        // stale must keep its file.
        if matches!(read_pid(&self.path), Ok(Some(pid)) if pid == self.pid) {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// The PID recorded in `path`, whatever its liveness.
fn read_pid(path: &std::path::Path) -> std::io::Result<Option<u32>> {
    match std::fs::read_to_string(path) {
        Ok(raw) => Ok(raw.trim().parse().ok()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// The PID in `path`, but only if that process still exists.
fn read_live_pid(path: &std::path::Path) -> std::io::Result<Option<u32>> {
    Ok(read_pid(path)?.filter(|&pid| pid_alive(pid)))
}

/// Whether a process with this PID exists.
#[cfg(unix)]
pub fn pid_alive(pid: u32) -> bool {
    use nix::errno::Errno;
    use nix::sys::signal::kill;
    use nix::unistd::Pid;

    // Signal 0 checks existence without delivering anything; EPERM means the
    // process exists but belongs to someone else.
    match kill(Pid::from_raw(pid as i32), None) {
        Ok(()) | Err(Errno::EPERM) => true,
        Err(_) => false,
    }
}

#[cfg(not(unix))]
pub fn pid_alive(_pid: u32) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Point `daemon_pid_path()` at a temp dir for the duration of a test.
    /// Serialized, because the env var it sets is process-wide.
    struct Sandbox {
        _tmp: tempfile::TempDir,
        _guard: std::sync::MutexGuard<'static, ()>,
        previous: Option<String>,
    }

    impl Sandbox {
        fn new() -> Self {
            static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
            let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let previous = std::env::var("XDG_RUNTIME_DIR").ok();
            let tmp = tempfile::tempdir().unwrap();
            std::env::set_var("XDG_RUNTIME_DIR", tmp.path());
            Self {
                _tmp: tmp,
                _guard: guard,
                previous,
            }
        }
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => std::env::set_var("XDG_RUNTIME_DIR", v),
                None => std::env::remove_var("XDG_RUNTIME_DIR"),
            }
        }
    }

    #[test]
    fn acquire_writes_our_pid() {
        let _sandbox = Sandbox::new();
        let held = PidFile::acquire().unwrap();
        assert_eq!(held.pid(), std::process::id());
        assert_eq!(
            PidFile::running_pid().unwrap(),
            Some(std::process::id()),
            "a held slot should report its pid as running"
        );
    }

    #[test]
    fn drop_releases_the_slot() {
        let _sandbox = Sandbox::new();
        let path = daemon_pid_path().unwrap();
        drop(PidFile::acquire().unwrap());
        assert!(!path.exists(), "drop should remove our own pid file");
        assert_eq!(PidFile::running_pid().unwrap(), None);
    }

    #[test]
    fn a_live_foreign_pid_blocks_acquire() {
        let _sandbox = Sandbox::new();
        let path = daemon_pid_path().unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        // PID 1 always exists and is never us.
        std::fs::write(&path, "1").unwrap();

        match PidFile::acquire() {
            Err(PidFileError::AlreadyRunning(1)) => {}
            other => panic!("expected AlreadyRunning(1), got {other:?}"),
        }
    }

    #[test]
    fn a_stale_pid_is_taken_over() {
        let _sandbox = Sandbox::new();
        let path = daemon_pid_path().unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        // Above the pid_max ceiling on every platform we run on, so it is
        // guaranteed to belong to nobody.
        std::fs::write(&path, "4294967290").unwrap();

        let held = PidFile::acquire().expect("a dead pid must not block a restart");
        assert_eq!(held.pid(), std::process::id());
    }

    #[test]
    fn a_garbage_pid_file_is_taken_over() {
        let _sandbox = Sandbox::new();
        let path = daemon_pid_path().unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "not a pid").unwrap();

        let held = PidFile::acquire().expect("an unparseable pid file must not wedge the daemon");
        assert_eq!(held.pid(), std::process::id());
    }

    #[test]
    fn current_process_is_alive() {
        assert!(pid_alive(std::process::id()));
    }
}
