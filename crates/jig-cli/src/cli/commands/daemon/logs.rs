//! `jig daemon logs` — print (and follow) the daemon's tracing log.

use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use clap::Args;

use crate::cli::op::Op;
use crate::cli::ui;
use crate::context::log::LogFile;

use super::{current_daemon_log, display_path};
use crate::context::AppCtx;
use crate::context::AppPaths;

/// Print the daemon's log
#[derive(Args, Debug, Clone)]
pub struct Logs {
    /// Keep printing lines as they are written (follows daemon restarts)
    #[arg(short, long)]
    follow: bool,

    /// Number of lines to show
    #[arg(short = 'n', long, default_value_t = 50)]
    lines: usize,

    /// Print the log file's path instead of its contents
    #[arg(long, conflicts_with = "follow")]
    path: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum LogsError {
    #[error("no daemon log found — start the daemon with `jig daemon start`")]
    NoLog,
    #[error("failed to read daemon log {0}: {1}")]
    Read(PathBuf, std::io::Error),
}

/// Log lines (or the log path), for stdout.
#[derive(Debug, Default)]
pub struct LogsOutput(String);

impl std::fmt::Display for LogsOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Op for Logs {
    type Context = AppPaths;
    type Error = LogsError;
    type Output = LogsOutput;

    fn build_context(&self, app: AppCtx) -> Result<AppPaths, LogsError> {
        Ok(app.paths)
    }

    fn run(&self, paths: AppPaths) -> Result<Self::Output, Self::Error> {
        let log = current_daemon_log(&paths).ok_or(LogsError::NoLog)?;
        if self.path {
            return Ok(LogsOutput(log.to_string()));
        }

        let lines = log
            .tail(self.lines)
            .map_err(|e| LogsError::Read(log.path().to_path_buf(), e))?;
        if !self.follow {
            return Ok(LogsOutput(lines.join("\n")));
        }

        ui::progress(&format!("following {}", display_path(log.path())));
        let _ = follow(&paths, &log, lines);
        Ok(LogsOutput::default())
    }
}

/// Stream lines until interrupted or stdout closes (e.g. piped into `head`).
fn follow(paths: &AppPaths, log: &LogFile, backlog: Vec<String>) -> std::io::Result<()> {
    let mut out = std::io::stdout().lock();
    for line in backlog {
        writeln!(out, "{line}")?;
    }
    out.flush()?;

    let mut tailer = log.follow();
    loop {
        for line in tailer.poll(usize::MAX) {
            writeln!(out, "{line}")?;
        }
        out.flush()?;

        // A restarted daemon writes a fresh log; move over to it.
        if let Some(newer) = current_daemon_log(paths).filter(|l| is_newer(l, tailer.log())) {
            ui::progress(&format!(
                "daemon restarted — following {}",
                display_path(newer.path())
            ));
            tailer = newer.follow_from_start();
            continue;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// Daemon log names start with a sortable timestamp.
fn is_newer(candidate: &LogFile, current: &LogFile) -> bool {
    candidate.path().file_name() > current.path().file_name()
}
