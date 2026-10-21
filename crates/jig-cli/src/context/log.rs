//! Every log jig keeps, and what can be done with each.
//!
//! Two kinds:
//!
//! - **Text logs** ([`LogFile`]): lines written by something else — tracing
//!   output, a hook's stdout and stderr — read back by a person or quoted in a
//!   message. Created, tailed, followed and removed through [`LogFile`].
//! - **Structured logs**: append-only JSONL that jig reduces into state — a
//!   worker's events, the daemon's lifecycle, the notification queue. Their
//!   formats live with their owners ([`jig_core::EventLog`],
//!   [`NotificationQueue`]); [`Logs`] is just where they are opened.
//!
//! Paths are [`AppPaths`]' business; opening is [`Logs`]'. Get one from
//! [`AppCtx::logs`](super::AppCtx::logs), or [`Logs::new`] in code that was
//! only handed an [`AppPaths`].

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::AppPaths;
use crate::notify::NotificationQueue;

// ── Logs ─────────────────────────────────────────────────────────────

/// Where every log jig keeps is opened.
///
/// NOTE(canonical): this is the one place a log is opened. **Do not** build a
/// log path, call `EventLog::new` or `NotificationQueue::new`, or `File::create`
/// a log anywhere else — add a method here, with its path on [`AppPaths`].
#[derive(Debug, Clone, Copy)]
pub struct Logs<'a> {
    paths: &'a AppPaths,
}

impl<'a> Logs<'a> {
    pub fn new(paths: &'a AppPaths) -> Self {
        Self { paths }
    }

    // ── text ──

    /// A fresh session log for this process's tracing output. Only processes
    /// whose `Op::log_sink` is `File` take one.
    pub fn new_session(&self) -> LogFile {
        LogFile::at(self.paths.new_session_log())
    }

    /// Every session log, oldest first.
    ///
    /// Names begin with a sortable UTC timestamp, so sorting them is sorting
    /// by age.
    pub fn sessions(&self) -> Vec<LogFile> {
        let Ok(entries) = std::fs::read_dir(self.paths.logs_dir()) else {
            return Vec::new();
        };
        let mut logs: Vec<PathBuf> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "log"))
            .collect();
        logs.sort();
        logs.into_iter().map(LogFile::at).collect()
    }

    /// Delete old session logs, keeping the newest `keep` that have content.
    ///
    /// Empty logs go first and do not count against `keep`: they are left by
    /// processes that opened a log and never wrote to it, and on one machine
    /// they were 1,505 of 1,522 files.
    ///
    /// The log this process is writing is never removed — deleting it would
    /// strand everything logged afterwards.
    ///
    /// Returns how many were removed. Errors are ignored throughout: tidying
    /// up must not stop the daemon from starting.
    pub fn prune(&self, keep: usize) -> usize {
        self.prune_keeping(keep, session_log().map(LogFile::path))
    }

    /// [`Self::prune`], with the live log named explicitly.
    ///
    /// `prune` reads it from this process's session, which is a set-once
    /// global. Taking it as an argument is what lets a test cover the "never
    /// delete the log being written" rule without writing to that global and
    /// leaking into every other test in the binary.
    pub fn prune_keeping(&self, keep: usize, live: Option<&Path>) -> usize {
        let mut kept = Vec::new();
        let mut removed = 0;

        for log in self.sessions() {
            if live.is_some_and(|live| live == log.path()) {
                continue;
            }
            match std::fs::metadata(log.path()) {
                Ok(meta) if meta.len() == 0 => {
                    if log.remove().is_ok() {
                        removed += 1;
                    }
                }
                Ok(_) => kept.push(log),
                Err(_) => {}
            }
        }

        let surplus = kept.len().saturating_sub(keep);
        for log in kept.into_iter().take(surplus) {
            if log.remove().is_ok() {
                removed += 1;
            }
        }
        removed
    }

    /// Delete every session log, including this process's. For `jig nuke`.
    pub fn clear(&self) -> usize {
        self.sessions()
            .into_iter()
            .filter(|log| log.remove().is_ok())
            .count()
    }

    // ── structured ──

    /// A worker's event log.
    pub fn worker_events(&self, repo: &str, branch: &str) -> crate::worker::events::EventLog {
        jig_core::EventLog::new(self.paths.worker_events_log(repo, branch))
    }

    /// The daemon's Started/Stopped history.
    pub fn daemon_events(&self) -> crate::daemon::events::DaemonLog {
        jig_core::EventLog::new(self.paths.daemon_lifecycle_log())
    }

    /// The notification queue.
    pub fn notifications(&self) -> NotificationQueue {
        NotificationQueue::new(self.paths.notifications())
    }
}

/// How many session logs to keep when pruning.
///
/// Enough to look back over a few daemon restarts, few enough that the
/// directory stays readable.
pub const KEEP_LOGS: usize = 20;

// ── This process's session log ───────────────────────────────────────

static SESSION_LOG: OnceLock<LogFile> = OnceLock::new();

/// Record where this process's tracing output goes. Set once at startup.
pub fn set_session_log(log: LogFile) {
    let _ = SESSION_LOG.set(log);
}

/// This process's tracing log, if it has one.
pub fn session_log() -> Option<&'static LogFile> {
    SESSION_LOG.get()
}

// ── LogFile ──────────────────────────────────────────────────────────

/// How much of a log to read per step when looking for its tail.
const TAIL_CHUNK: u64 = 64 * 1024;

/// A text log: what can be done with one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFile {
    path: PathBuf,
}

impl LogFile {
    /// A log at a path that was recorded somewhere — the daemon's `Started`
    /// event names its session log. Anything jig decides the path of comes
    /// from [`Logs`] instead.
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Start the log afresh — truncating it, and creating the directory it
    /// goes in — and hand back the file to write it through.
    pub fn create(&self) -> std::io::Result<File> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        File::create(&self.path)
    }

    /// The last `n` lines.
    ///
    /// Reads back from the end only as far as it needs to, so a large log
    /// costs no more than a small one.
    pub fn tail(&self, n: usize) -> std::io::Result<Vec<String>> {
        let mut file = File::open(&self.path)?;
        let len = file.metadata()?.len();

        // Step back a chunk at a time until there are more than `n` newlines
        // in hand (the extra one ends the line before the first we keep), or
        // the start of the file.
        let mut start = len;
        let mut buf = Vec::new();
        while start > 0 && buf.iter().filter(|&&b| b == b'\n').count() <= n {
            let step = TAIL_CHUNK.min(start);
            start -= step;
            file.seek(SeekFrom::Start(start))?;
            let mut chunk = vec![0; step as usize];
            file.read_exact(&mut chunk)?;
            chunk.extend_from_slice(&buf);
            buf = chunk;
        }

        let text = String::from_utf8_lossy(&buf);
        let mut lines: Vec<&str> = text.lines().collect();
        // Not starting at the file's start means the first line is cut.
        if start > 0 && !lines.is_empty() {
            lines.remove(0);
        }
        let skip = lines.len().saturating_sub(n);
        Ok(lines[skip..].iter().map(|l| l.to_string()).collect())
    }

    /// Follow lines written from now on.
    pub fn follow(&self) -> LogTailer {
        let offset = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        LogTailer {
            log: self.clone(),
            offset,
        }
    }

    /// Follow from the first line — for a log that has only just started.
    pub fn follow_from_start(&self) -> LogTailer {
        LogTailer {
            log: self.clone(),
            offset: 0,
        }
    }

    pub fn remove(&self) -> std::io::Result<()> {
        match std::fs::remove_file(&self.path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}

impl std::fmt::Display for LogFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.display().fmt(f)
    }
}

/// Reads lines appended to a log since the last poll. From
/// [`LogFile::follow`].
pub struct LogTailer {
    log: LogFile,
    offset: u64,
}

impl LogTailer {
    pub fn log(&self) -> &LogFile {
        &self.log
    }

    /// New complete lines since the last poll, keeping at most the last
    /// `max_lines`.
    pub fn poll(&mut self, max_lines: usize) -> Vec<String> {
        let Ok(file) = File::open(self.log.path()) else {
            return vec![];
        };
        // Truncated or replaced underneath us — start over.
        if file.metadata().map(|m| m.len()).unwrap_or(0) < self.offset {
            self.offset = 0;
        }
        let mut reader = BufReader::new(file);
        if reader.seek(SeekFrom::Start(self.offset)).is_err() {
            return vec![];
        }

        let mut lines = Vec::new();
        let mut line = String::new();
        while reader.read_line(&mut line).unwrap_or(0) > 0 {
            // A line without its newline is still being written; leave it
            // for the next poll.
            if !line.ends_with('\n') {
                break;
            }
            self.offset += line.len() as u64;
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
            line.clear();
        }

        if lines.len() > max_lines {
            lines.split_off(lines.len() - max_lines)
        } else {
            lines
        }
    }
}
