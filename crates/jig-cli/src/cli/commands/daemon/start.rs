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

    /// Seconds `--once` waits for the monitor pass before giving up [default: 30]
    ///
    /// A pass that polls many repos over a slow network can outrun the
    /// default; raise this rather than accepting a half-filled tick.
    // Deliberately not `default_value_t`: clap treats a defaulted argument as
    // present, which would make `requires` reject every run without `--once`.
    #[arg(long, value_name = "SECONDS", requires = "once")]
    timeout: Option<u64>,
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

impl Start {
    /// How long `--once` waits for the monitor pass it dispatched.
    fn monitor_wait(&self) -> Duration {
        self.timeout
            .map(Duration::from_secs)
            .unwrap_or(Daemon::MONITOR_WAIT)
    }
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
        let monitor_wait = self.monitor_wait();
        daemon.run(&quit, |daemon| {
            if once {
                if !daemon.wait_for_monitor(monitor_wait) {
                    ui::warning(&format!(
                        "monitor pass still running after {}s — exiting anyway {}",
                        monitor_wait.as_secs(),
                        ui::dim("(raise --timeout)")
                    ));
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Parse just this subcommand's arguments.
    #[derive(Parser)]
    struct Harness {
        #[command(flatten)]
        start: Start,
    }

    fn parse(args: &[&str]) -> Start {
        Harness::try_parse_from(args).expect("should parse").start
    }

    #[test]
    fn monitor_wait_defaults_to_the_daemons_bound() {
        assert_eq!(
            parse(&["jig", "--once"]).monitor_wait(),
            Daemon::MONITOR_WAIT
        );
    }

    #[test]
    fn timeout_overrides_the_default() {
        assert_eq!(
            parse(&["jig", "--once", "--timeout", "90"]).monitor_wait(),
            Duration::from_secs(90)
        );
    }

    #[test]
    fn timeout_is_rejected_without_once() {
        // It only bounds the `--once` wait, so accepting it alone would
        // silently do nothing.
        assert!(Harness::try_parse_from(["jig", "--timeout", "90"]).is_err());
    }

    #[test]
    fn a_plain_start_needs_no_arguments() {
        let start = parse(&["jig"]);
        assert!(!start.once);
        assert_eq!(start.monitor_wait(), Daemon::MONITOR_WAIT);
    }
}
