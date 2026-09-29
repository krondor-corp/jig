//! Daemon command — run and inspect the background daemon.
//!
//! The daemon is a single per-user process bound to a unix socket; `status`
//! and `stop` are IPC clients rather than file readers.

mod logs;
mod service;
mod start;
mod status;
mod stop;

use std::path::PathBuf;

use clap::Args;

use crate::cli::op::Op;
use crate::context::AppCtx;
use crate::context::AppPaths;
use crate::daemon::ipc;

/// Run and inspect the background daemon (start, stop, status, logs, install)
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
    /// Run the daemon as an OS service (Linux: system service, needs sudo)
    (Install, service::Install),
    /// Remove the daemon's OS service
    (Uninstall, service::Uninstall),
    /// Restart the daemon service, e.g. after `jig update`
    (Restart, service::Restart),
}

impl Op for Daemon {
    type Context = AppCtx;
    type Output = OpOutput;
    type Error = OpError;

    fn build_context(&self, app: AppCtx) -> Result<AppCtx, Self::Error> {
        Ok(app)
    }

    fn run(&self, app: AppCtx) -> Result<Self::Output, Self::Error> {
        match &self.command {
            Some(cmd) => cmd.run(app),
            None => Command::Status(status::Status).run(app),
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
fn current_daemon_log(paths: &AppPaths) -> Option<DaemonLog> {
    let running = ipc::ping(paths).ok().flatten().and_then(|info| info.log);
    let path = running.or_else(|| crate::daemon::events::global(paths).reduce().ok()?.log)?;
    let on_disk = path.exists();
    Some(DaemonLog { path, on_disk })
}

/// Where the daemon says its log is, and whether that file is actually there.
///
/// One type for both answers because `status` and `logs` have to agree.
/// `status` printed the reported path unchecked while `logs` silently
/// dropped a path that was missing and reported "no daemon log found — start
/// the daemon", about a daemon that was running and named in the line above.
pub(super) struct DaemonLog {
    pub path: PathBuf,
    pub on_disk: bool,
}

/// Display a path with the home directory collapsed to `~`.
fn display_path(path: &std::path::Path) -> String {
    match dirs::home_dir().and_then(|home| path.strip_prefix(home).ok().map(PathBuf::from)) {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}
