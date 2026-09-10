//! Heartbeat — a snapshot the long-running daemon rewrites every tick.
//!
//! `jig daemon status` reads it to tell a healthy daemon from a dead or
//! wedged one. It stands in for the pidfile of `docs/daemon-service-plan.md`
//! until the daemon binds a socket and can answer for itself.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::actors::ActorActivity;
use crate::context::daemon_heartbeat_path;

/// An actor busy longer than this is reported as possibly stuck.
pub const STUCK_ACTOR_SECS: i64 = 10 * 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heartbeat {
    pub pid: u32,
    pub version: String,
    pub started_at: i64,
    pub ticked_at: i64,
    pub tick_interval: u64,
    /// The daemon's own tracing log.
    pub log: Option<PathBuf>,
    pub actors: Vec<ActorActivity>,
}

/// What a heartbeat says about the daemon that wrote it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    /// Process alive and ticking on schedule.
    Running,
    /// Process alive but the tick loop has not run for a while.
    Stalled,
    /// Process gone without a clean shutdown (clean shutdowns remove the file).
    Dead,
}

impl Heartbeat {
    /// Read the current heartbeat; `None` when no daemon has left one.
    pub fn read() -> std::io::Result<Option<Self>> {
        let path = daemon_heartbeat_path()?;
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        serde_json::from_str(&raw)
            .map(Some)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Replace the heartbeat file atomically so readers never see a partial write.
    pub fn write(&self) -> std::io::Result<()> {
        let path = daemon_heartbeat_path()?;
        let tmp = path.with_extension(format!("json.{}", self.pid));
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(&tmp, &path)
    }

    /// Remove the heartbeat on clean shutdown — unless another daemon has
    /// since taken it over.
    pub fn clear(pid: u32) {
        let Ok(path) = daemon_heartbeat_path() else {
            return;
        };
        if matches!(Self::read(), Ok(Some(hb)) if hb.pid == pid) {
            let _ = std::fs::remove_file(path);
        }
    }

    pub fn liveness(&self, now: i64) -> Liveness {
        self.liveness_with(now, pid_alive(self.pid))
    }

    fn liveness_with(&self, now: i64, alive: bool) -> Liveness {
        if !alive {
            Liveness::Dead
        } else if now - self.ticked_at > self.stall_after_secs() {
            Liveness::Stalled
        } else {
            Liveness::Running
        }
    }

    /// Several missed ticks, with slack for a slow tick on a slow machine.
    fn stall_after_secs(&self) -> i64 {
        3 * self.tick_interval as i64 + 30
    }

    /// Actors whose current request has been running suspiciously long.
    pub fn stuck_actors(&self, now: i64) -> impl Iterator<Item = &ActorActivity> {
        self.actors.iter().filter(move |a| {
            a.busy_since
                .is_some_and(|since| now - since > STUCK_ACTOR_SECS)
        })
    }
}

#[cfg(unix)]
fn pid_alive(pid: u32) -> bool {
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
fn pid_alive(_pid: u32) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heartbeat(ticked_at: i64, actors: Vec<ActorActivity>) -> Heartbeat {
        Heartbeat {
            pid: 1,
            version: "0.0.0".into(),
            started_at: 0,
            ticked_at,
            tick_interval: 2,
            log: None,
            actors,
        }
    }

    fn actor(name: &str, busy_since: Option<i64>) -> ActorActivity {
        ActorActivity {
            name: name.into(),
            busy_since,
            last_finished: None,
        }
    }

    #[test]
    fn fresh_tick_is_running() {
        assert_eq!(
            heartbeat(1000, vec![]).liveness_with(1010, true),
            Liveness::Running
        );
    }

    #[test]
    fn missed_ticks_are_stalled() {
        // tick_interval 2 → stalled after 36s without a tick
        let hb = heartbeat(1000, vec![]);
        assert_eq!(hb.liveness_with(1036, true), Liveness::Running);
        assert_eq!(hb.liveness_with(1037, true), Liveness::Stalled);
    }

    #[test]
    fn missing_process_is_dead_regardless_of_tick() {
        assert_eq!(
            heartbeat(1000, vec![]).liveness_with(1001, false),
            Liveness::Dead
        );
    }

    #[test]
    fn only_long_busy_actors_are_stuck() {
        let now = 10_000;
        let hb = heartbeat(
            now,
            vec![
                actor("idle", None),
                actor("brief", Some(now - 30)),
                actor("wedged", Some(now - STUCK_ACTOR_SECS - 1)),
            ],
        );
        let stuck: Vec<_> = hb.stuck_actors(now).map(|a| a.name.as_str()).collect();
        assert_eq!(stuck, vec!["wedged"]);
    }

    #[test]
    fn roundtrips_through_json() {
        let hb = heartbeat(5, vec![actor("jig-sync", Some(3))]);
        let json = serde_json::to_string(&hb).unwrap();
        assert_eq!(serde_json::from_str::<Heartbeat>(&json).unwrap(), hb);
    }

    #[test]
    fn current_process_is_alive() {
        assert!(pid_alive(std::process::id()));
    }
}
