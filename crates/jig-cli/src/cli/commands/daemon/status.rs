//! `jig daemon status` — is the daemon alive, ticking, and unstuck?

use clap::Args;

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::daemon::actors::ActorActivity;
use crate::daemon::heartbeat::{Heartbeat, Liveness};

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
    #[error("failed to read daemon heartbeat: {0}")]
    Io(#[from] std::io::Error),
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
        let Some(hb) = Heartbeat::read()? else {
            ui::failure("daemon not running");
            if let Some(stopped) = last_stop() {
                ui::detail(&stopped);
            }
            ui::detail(&format!("start it with {}", ui::highlight("jig ps -gw")));
            return Err(StatusError::NotRunning);
        };

        let ago = |ts: i64| ui::format_duration_short((now - ts).max(0) as u64);
        let liveness = hb.liveness(now);
        match liveness {
            Liveness::Running => ui::success(&format!(
                "daemon running  {}",
                ui::dim(&format!(
                    "pid {} · up {} · v{}",
                    hb.pid,
                    ago(hb.started_at),
                    hb.version
                ))
            )),
            Liveness::Stalled => ui::warning(&format!(
                "daemon stalled — pid {} is alive but has not ticked for {}",
                hb.pid,
                ago(hb.ticked_at)
            )),
            Liveness::Dead => ui::failure(&format!(
                "daemon not running — pid {} exited without shutting down",
                hb.pid
            )),
        }

        ui::detail(&format!(
            "last tick {} ago {}",
            ago(hb.ticked_at),
            ui::dim(&format!("(every {}s)", hb.tick_interval))
        ));
        if let Some(log) = &hb.log {
            ui::detail(&format!("log {}", display_path(log)));
        }

        if liveness == Liveness::Dead {
            ui::detail(&format!("restart it with {}", ui::highlight("jig ps -gw")));
            return Err(StatusError::NotRunning);
        }

        let width = hb.actors.iter().map(|a| a.name.len()).max().unwrap_or(0);
        for actor in &hb.actors {
            ui::detail(&format!(
                "{:width$}  {}",
                actor.name,
                describe(actor, now, &ago)
            ));
        }

        let stuck: Vec<_> = hb.stuck_actors(now).map(|a| a.name.as_str()).collect();
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

fn describe(actor: &ActorActivity, now: i64, ago: &dyn Fn(i64) -> String) -> String {
    let last = actor
        .last_finished
        .map(|ts| format!("last finished {} ago", ago(ts)));
    match actor.busy_since {
        Some(since) => {
            let busy = format!("busy {}", ago(since));
            let busy = if now - since > crate::daemon::heartbeat::STUCK_ACTOR_SECS {
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
