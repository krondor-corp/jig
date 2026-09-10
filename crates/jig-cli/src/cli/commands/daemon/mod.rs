//! Daemon command — inspect the background daemon run by `jig ps --watch`.

mod logs;
mod status;

use std::path::PathBuf;

use clap::Args;

use crate::cli::op::Op;
use crate::daemon::heartbeat::Heartbeat;

/// Inspect the background daemon (status, logs)
#[derive(Args, Debug, Clone)]
pub struct Daemon {
    #[command(subcommand)]
    pub command: Option<Command>,
}

crate::command_enum! {
    /// Show whether the daemon is running, ticking, and unstuck (default)
    (Status, status::Status),
    /// Print the daemon's log
    (Logs, logs::Logs),
}

impl Op for Daemon {
    type Context = ();
    type Output = OpOutput;
    type Error = OpError;

    fn build_context(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
        match &self.command {
            Some(cmd) => cmd.run(()),
            None => Command::Status(status::Status).run(()),
        }
    }
}

/// The log of the running daemon, else of the most recent daemon run.
fn current_daemon_log() -> Option<PathBuf> {
    Heartbeat::read()
        .ok()
        .flatten()
        .and_then(|hb| hb.log)
        .filter(|p| p.exists())
        .or_else(|| crate::context::latest_daemon_log().ok().flatten())
}

/// Display a path with the home directory collapsed to `~`.
fn display_path(path: &std::path::Path) -> String {
    match dirs::home_dir().and_then(|home| path.strip_prefix(home).ok().map(PathBuf::from)) {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}
