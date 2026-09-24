//! Daemon command — run and inspect the background daemon.
//!
//! The daemon is a single per-user process bound to a unix socket; `status`
//! and `stop` are IPC clients rather than file readers.

mod logs;
mod start;
mod status;
mod stop;

use std::path::PathBuf;

use clap::Args;

use crate::cli::op::Op;
use crate::context::AppPaths;
use crate::daemon::ipc;

/// Run and inspect the background daemon (start, stop, status, logs)
#[derive(Args, Debug, Clone)]
pub struct Daemon {
    #[command(subcommand)]
    pub command: Option<Command>,
}

crate::command_enum! {
    /// Run the daemon in the foreground
    (Start, start::Start),
    /// Stop the running daemon
    (Stop, stop::Stop),
    /// Show whether the daemon is running, ticking, and unstuck (default)
    (Status, status::Status),
    /// Print the daemon's log
    (Logs, logs::Logs),
}

impl Op for Daemon {
    type Context = AppPaths;
    type Output = OpOutput;
    type Error = OpError;

    fn build_context(&self, paths: &AppPaths) -> Result<AppPaths, Self::Error> {
        Ok(paths.clone())
    }

    fn run(&self, paths: AppPaths) -> Result<Self::Output, Self::Error> {
        match &self.command {
            Some(cmd) => cmd.run(paths),
            None => Command::Status(status::Status).run(paths),
        }
    }

    fn log_sink(&self) -> crate::cli::op::LogSink {
        match &self.command {
            Some(cmd) => cmd.log_sink(),
            None => Command::Status(status::Status).log_sink(),
        }
    }
}

/// The running daemon's log, else the one the last daemon run recorded in
/// its `Started` event.
fn current_daemon_log(paths: &AppPaths) -> Option<PathBuf> {
    let running = ipc::ping(paths).ok().flatten().and_then(|info| info.log);
    running
        .or_else(|| crate::daemon::events::global(paths).reduce().ok()?.log)
        .filter(|p| p.exists())
}

/// Display a path with the home directory collapsed to `~`.
fn display_path(path: &std::path::Path) -> String {
    match dirs::home_dir().and_then(|home| path.strip_prefix(home).ok().map(PathBuf::from)) {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}
