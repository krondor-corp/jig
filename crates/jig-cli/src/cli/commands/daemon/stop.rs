//! `jig daemon stop` — ask the running daemon to shut down.

use clap::Args;

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::daemon::ipc::{self, IpcError, Request, Response};

/// Stop the running daemon
#[derive(Args, Debug, Clone)]
pub struct Stop;

#[derive(Debug, thiserror::Error)]
pub enum StopError {
    #[error("daemon is not running")]
    NotRunning,
    #[error(transparent)]
    Ipc(#[from] IpcError),
}

impl Op for Stop {
    type Context = ();
    type Error = StopError;
    type Output = NoOutput;

    fn build_context(&self) -> Result<(), StopError> {
        Ok(())
    }

    fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
        // Ask who we are stopping first, so the confirmation can name a pid
        // and a stale socket is reported as "not running" rather than a
        // connection error.
        let pid = match ipc::ping()? {
            Some(info) => info.pid,
            None => {
                ui::failure("daemon not running");
                return Err(StopError::NotRunning);
            }
        };

        match ipc::request(&Request::Shutdown) {
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
