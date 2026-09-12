//! `jig daemon status` — is the daemon alive, ticking, and unstuck?
//!
//! The daemon answers for itself over its socket (`Request::Ping`). That a
//! reply came back at all is the liveness proof the heartbeat file could
//! only approximate; the per-actor busy/finished times it used to carry ride
//! along in the response.

use clap::Args;

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::daemon::actors::ActorActivity;
use crate::daemon::ipc::{self, IpcError, Liveness, STUCK_ACTOR_SECS};
use crate::daemon::pidfile::PidFile;

use super::display_path;

/// Show whether the daemon is running, ticking, and unstuck
#[derive(Args, Debug, Clone)]
pub struct Status;

#[derive(Debug, thiserror::Error)]
pub enum StatusError {
    #[error("daemon is not running")]
    NotRunning,
    #[error("daemon is stalled")]
    Stalled,
    #[error("daemon has actors that look stuck")]
    StuckActors,
    #[error(transparent)]
    Ipc(#[from] IpcError),
}

impl Op for Status {
    type Context = ();
    type Error = StatusError;
    type Output = NoOutput;

    fn build_context(&self) -> Result<(), StatusError> {
        Ok(())
    }

    fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
        let now = chrono::Utc::now().timestamp();
        let Some(info) = ipc::ping()? else {
            report_not_running();
            return Err(StatusError::NotRunning);
        };

        let ago = |ts: i64| ui::format_duration_short((now - ts).max(0) as u64);
        let liveness = info.liveness(now);
        match liveness {
            Liveness::Running => ui::success(&format!(
                "daemon running  {}",
                ui::dim(&format!(
                    "pid {} · up {} · v{}",
                    info.pid,
                    ago(info.started_at),
                    info.version
                ))
            )),
            Liveness::Stalled => ui::warning(&format!(
                "daemon stalled — pid {} is answering but has not ticked for {}",
                info.pid,
                ago(info.ticked_at)
            )),
        }

        ui::detail(&format!(
            "last tick {} ago {}",
            ago(info.ticked_at),
            ui::dim(&format!("(every {}s)", info.tick_interval))
        ));
        if let Some(log) = &info.log {
            ui::detail(&format!("log {}", display_path(log)));
        }

        let width = info.actors.iter().map(|a| a.name.len()).max().unwrap_or(0);
        for actor in &info.actors {
            ui::detail(&format!(
                "{:width$}  {}",
                actor.name,
                describe(actor, now, &ago)
            ));
        }

        let stuck: Vec<_> = info.stuck_actors(now).map(|a| a.name.as_str()).collect();
        if !stuck.is_empty() {
            ui::warning(&format!(
                "{} busy far longer than a normal pass — check {}",
                stuck.join(", "),
                ui::highlight("jig daemon logs")
            ));
        }

        match liveness {
            Liveness::Stalled => Err(StatusError::Stalled),
            _ if !stuck.is_empty() => Err(StatusError::StuckActors),
            _ => Ok(NoOutput),
        }
    }
}

/// Nothing answered the socket. A PID file still claiming a live process
/// means the daemon is up but its listener is gone — worth saying, since
/// "not running" would send the user to start a second one.
fn report_not_running() {
    match PidFile::running_pid() {
        Ok(Some(pid)) => {
            ui::failure(&format!(
                "daemon not answering — pid {pid} is alive but its socket is gone"
            ));
            ui::detail(&format!(
                "kill it with {}",
                ui::highlight(&format!("kill {pid}"))
            ));
        }
        _ => {
            ui::failure("daemon not running");
            if let Some(stopped) = last_stop() {
                ui::detail(&stopped);
            }
            ui::detail(&format!(
                "start it with {}",
                ui::highlight("jig daemon start")
            ));
        }
    }
}

fn describe(actor: &ActorActivity, now: i64, ago: &dyn Fn(i64) -> String) -> String {
    let last = actor
        .last_finished
        .map(|ts| format!("last finished {} ago", ago(ts)));
    match actor.busy_since {
        Some(since) => {
            let busy = format!("busy {}", ago(since));
            let busy = if now - since > STUCK_ACTOR_SECS {
                ui::warn_text(&busy)
            } else {
                busy
            };
            match last {
                Some(last) => format!("{busy} {}", ui::dim(&format!("({last})"))),
                None => busy,
            }
        }
        None => ui::dim(&last.unwrap_or_else(|| "not run yet".into())),
    }
}

/// "last stopped 3h ago (normal)", from the lifecycle log, when it is readable.
fn last_stop() -> Option<String> {
    let state = crate::daemon::events::global().ok()?.reduce().ok()?;
    let stopped_at = state.stopped_at?;
    let ago = (chrono::Utc::now().timestamp() - stopped_at).max(0) as u64;
    Some(format!(
        "last stopped {} ago ({})",
        ui::format_duration_short(ago),
        state.stop_reason.as_deref().unwrap_or("unknown")
    ))
}
