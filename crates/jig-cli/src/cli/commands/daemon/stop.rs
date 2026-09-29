//! `jig daemon stop` — ask the running daemon to shut down.

use clap::Args;

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::AppCtx;
use crate::context::AppPaths;
use crate::daemon::ipc::{self, IpcError, Request, Response};

/// Stop the running daemon
#[derive(Args, Debug, Clone)]
pub struct Stop {
    /// Stop the user service rather than the system one
    #[arg(long)]
    user: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum StopError {
    #[error("daemon is not running")]
    NotRunning,
    #[error(transparent)]
    Ipc(#[from] IpcError),
    #[error(transparent)]
    Service(#[from] super::service::ServiceError),
}

impl Op for Stop {
    type Context = AppPaths;
    type Error = StopError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<AppPaths, StopError> {
        Ok(app.paths)
    }

    /// Stop means stop.
    ///
    /// An installed service is `Restart=always`, so asking the process to
    /// exit would only bounce it — five seconds later it is back and you
    /// were told it stopped. So a service is stopped through the service
    /// manager, which for a system unit needs root. A daemon running in the
    /// foreground is still asked over the socket, as before.
    fn run(&self, paths: AppPaths) -> Result<Self::Output, Self::Error> {
        if let Some(outcome) = super::service::stop_service(self.user)? {
            return Ok(outcome);
        }

        // Ask who we are stopping first, so the confirmation can name a pid
        // and a stale socket is reported as "not running" rather than a
        // connection error.
        let pid = match ipc::ping(&paths)? {
            Some(info) => info.pid,
            None => {
                ui::failure("daemon not running");
                return Err(StopError::NotRunning);
            }
        };

        match ipc::request(&paths, &Request::Shutdown) {
            Ok(Response::Ok) => {
                ui::success(&format!(
                    "daemon stopping  {}",
                    ui::dim(&format!("pid {pid}"))
                ));
                Ok(NoOutput)
            }
            Ok(other) => Err(IpcError::Protocol(format!("expected ok, got {other:?}")).into()),
            // The daemon can drop the connection as it goes down; that is
            // the shutdown working, not a failure.
            Err(IpcError::NotRunning) => {
                ui::success(&format!(
                    "daemon stopping  {}",
                    ui::dim(&format!("pid {pid}"))
                ));
                Ok(NoOutput)
            }
            Err(e) => Err(e.into()),
        }
    }
}
