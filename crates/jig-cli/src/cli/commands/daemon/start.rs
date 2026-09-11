//! `jig daemon start` — run the daemon in the foreground, serving IPC.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::Args;

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::Context;
use crate::daemon::ipc::{IpcError, Server};
use crate::daemon::pidfile::{PidFile, PidFileError};
use crate::daemon::{Daemon, DaemonError};

/// Run the daemon in the foreground
#[derive(Args, Debug, Clone)]
pub struct Start {
    /// Run a single tick and exit, instead of looping
    #[arg(long)]
    once: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum StartError {
    #[error(transparent)]
    Pid(#[from] PidFileError),
    #[error(transparent)]
    Ipc(#[from] IpcError),
    #[error(transparent)]
    Daemon(#[from] DaemonError),
    #[error(transparent)]
    Context(#[from] crate::context::ContextError),
}

impl Op for Start {
    type Context = Context;
    type Error = StartError;
    type Output = NoOutput;

    /// Always global: one daemon per user, watching every tracked repo.
    fn build_context(&self) -> Result<Context, StartError> {
        Ok(Context::from_global()?)
    }

    fn run(&self, cfg: Context) -> Result<Self::Output, Self::Error> {
        // Claim the slot before doing any work, so a second daemon fails
        // fast and loudly rather than half-starting and fighting the first.
        let pid_file = PidFile::acquire()?;
        let server = Server::bind()?;

        let quit = Arc::new(AtomicBool::new(false));
        install_signal_handlers(Arc::clone(&quit));

        let tick_interval = cfg.config.tick_interval;
        let mut daemon = Daemon::start(cfg)?;
        let listener = server.spawn(daemon.shared(), Arc::clone(&quit));

        ui::success(&format!(
            "daemon started  {}",
            ui::dim(&format!("pid {}", pid_file.pid()))
        ));

        let once = self.once;
        daemon.run(&quit, |daemon| {
            if once {
                // The monitor pass runs on its own thread; a one-shot run
                // that returns immediately would exit before it finishes.
                wait_for_monitor(daemon);
                return false;
            }
            sleep_until_tick(&quit, tick_interval);
            !quit.load(Ordering::Relaxed)
        });

        // Stop the listener before dropping the socket and PID file, so no
        // client can connect to a daemon that is already on its way out.
        quit.store(true, Ordering::Relaxed);
        let _ = listener.join();
        drop(pid_file);

        ui::success("daemon stopped");
        Ok(NoOutput)
    }
}

/// How long a `--once` run waits for the monitor pass it just dispatched.
const ONCE_MONITOR_TIMEOUT: Duration = Duration::from_secs(30);

fn wait_for_monitor(daemon: &Daemon) {
    let start = Instant::now();
    while daemon.monitor.is_pending() && start.elapsed() < ONCE_MONITOR_TIMEOUT {
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Sleep between ticks, waking early when a signal or `jig daemon stop`
/// sets `quit`.
fn sleep_until_tick(quit: &AtomicBool, tick_interval: u64) {
    let end = Instant::now() + Duration::from_secs(tick_interval);
    while Instant::now() < end {
        if quit.load(Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// SIGINT/SIGTERM ask for the same orderly stop as `jig daemon stop`, so the
/// socket and PID file are cleaned up rather than left for the next start to
/// treat as stale.
fn install_signal_handlers(quit: Arc<AtomicBool>) {
    if let Err(e) = ctrlc::set_handler(move || {
        quit.store(true, Ordering::Relaxed);
    }) {
        tracing::warn!("failed to install signal handler: {}", e);
    }
}
