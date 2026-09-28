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
//!
//! A claim also records which boot it was made in. On Linux the runtime
//! directory is a tmpfs that comes up empty, so this never matters. macOS has
//! no `XDG_RUNTIME_DIR`, so the file lives under `~/.config` and survives a
//! reboot — and after a reboot the recorded PID very often belongs to some
//! unrelated process, which made the daemon refuse to start until someone
//! deleted the file by hand.

use std::io::Write;
use std::path::PathBuf;

use crate::context::AppPaths;

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
    pub fn acquire(paths: &AppPaths) -> Result<Self, PidFileError> {
        let path = paths.pid_file();
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
                write!(file, "{}", Claim::ours())?;
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
        write!(file, "{}", Claim::ours())?;
        Ok(Self { path, pid })
    }

    /// PID of the daemon currently holding the slot, if one is alive.
    pub fn running_pid(paths: &AppPaths) -> std::io::Result<Option<u32>> {
        read_live_pid(&paths.pid_file())
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

/// What a pid file records: who holds the slot, and when.
///
/// Written as `<pid>` or `<pid> <boot>`. The bare form is what older versions
/// wrote, and is still read — a running daemon must not be evicted by an
/// upgrade.
#[derive(Debug, PartialEq, Eq)]
struct Claim {
    pid: u32,
    /// The boot this claim was made in, where the platform will say.
    boot: Option<u64>,
}

impl Claim {
    fn ours() -> Self {
        Self {
            pid: std::process::id(),
            boot: boot_time(),
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        let mut parts = raw.split_whitespace();
        let pid = parts.next()?.parse().ok()?;
        Some(Self {
            pid,
            boot: parts.next().and_then(|b| b.parse().ok()),
        })
    }

    /// Whether this claim still belongs to a running daemon.
    ///
    /// A claim from an earlier boot is stale whatever its PID says: the
    /// number has been handed out again, and `kill(pid, 0)` will happily
    /// confirm that some unrelated process holds it.
    fn is_live(&self) -> bool {
        match (self.boot, boot_time()) {
            (Some(claimed), Some(now)) if claimed != now => false,
            _ => pid_alive(self.pid),
        }
    }
}

impl std::fmt::Display for Claim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.boot {
            Some(boot) => write!(f, "{} {}", self.pid, boot),
            None => write!(f, "{}", self.pid),
        }
    }
}

/// The claim recorded in `path`, whatever its liveness.
fn read_claim(path: &std::path::Path) -> std::io::Result<Option<Claim>> {
    match std::fs::read_to_string(path) {
        Ok(raw) => Ok(Claim::parse(&raw)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// The PID recorded in `path`, whatever its liveness.
fn read_pid(path: &std::path::Path) -> std::io::Result<Option<u32>> {
    Ok(read_claim(path)?.map(|c| c.pid))
}

/// The PID in `path`, but only if that claim is still live.
fn read_live_pid(path: &std::path::Path) -> std::io::Result<Option<u32>> {
    Ok(read_claim(path)?.filter(Claim::is_live).map(|c| c.pid))
}

/// When this machine booted, in seconds since the epoch.
///
/// `None` where the platform will not say, in which case a claim carries no
/// boot stamp and liveness falls back to the PID alone.
#[cfg(target_os = "linux")]
fn boot_time() -> Option<u64> {
    std::fs::read_to_string("/proc/stat")
        .ok()?
        .lines()
        .find_map(|line| line.strip_prefix("btime "))
        .and_then(|secs| secs.trim().parse().ok())
}

#[cfg(target_os = "macos")]
fn boot_time() -> Option<u64> {
    use nix::libc::{c_void, sysctlbyname, timeval};

    let mut boot = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut len = std::mem::size_of::<timeval>();

    // SAFETY: `kern.boottime` is a well-known read-only sysctl returning a
    // `timeval`, and `len` describes the buffer we hand it.
    let rc = unsafe {
        sysctlbyname(
            c"kern.boottime".as_ptr(),
            (&mut boot as *mut timeval).cast::<c_void>(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    (rc == 0).then_some(boot.tv_sec as u64)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn boot_time() -> Option<u64> {
    None
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

    #[test]
    fn current_process_is_alive() {
        assert!(pid_alive(std::process::id()));
    }

    #[test]
    fn this_machine_reports_a_boot_time() {
        // Both platforms jig ships on can answer; the fallback exists for
        // completeness, not because we expect to hit it.
        assert!(
            boot_time().is_some(),
            "expected a boot time on this platform"
        );
    }

    #[test]
    fn a_bare_pid_is_still_readable() {
        // What older versions wrote. A daemon running across an upgrade must
        // not be evicted by one.
        let claim = Claim::parse("4242\n").unwrap();
        assert_eq!(claim.pid, 4242);
        assert_eq!(claim.boot, None);
    }

    #[test]
    fn a_claim_round_trips() {
        let claim = Claim::ours();
        assert_eq!(Claim::parse(&claim.to_string()), Some(claim));
    }

    #[test]
    fn a_live_pid_from_a_previous_boot_is_not_live() {
        // pid 1 always exists, so this isolates the boot check: without it,
        // `kill(1, 0)` succeeds and the claim looks held.
        let stale = Claim {
            pid: 1,
            boot: Some(boot_time().unwrap() - 1),
        };
        assert!(
            !stale.is_live(),
            "a claim from an earlier boot must be stale"
        );

        let current = Claim {
            pid: 1,
            boot: boot_time(),
        };
        assert!(current.is_live(), "pid 1 is running, this boot");
    }

    #[test]
    fn a_dead_pid_from_this_boot_is_not_live() {
        let claim = Claim {
            pid: u32::MAX - 1,
            boot: boot_time(),
        };
        assert!(!claim.is_live());
    }
}
