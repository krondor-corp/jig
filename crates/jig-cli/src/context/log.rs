//! Session log files — where `ps --watch` and the daemon send their tracing
//! output — and a tailer for following one as it grows.

use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::AppPaths;

static SESSION_LOG: OnceLock<PathBuf> = OnceLock::new();

/// Record where this process's tracing output goes. Set once at startup.
pub fn set_session_log(path: PathBuf) {
    let _ = SESSION_LOG.set(path);
}

/// This process's tracing log, if it has one.
pub fn session_log() -> Option<&'static Path> {
    SESSION_LOG.get().map(PathBuf::as_path)
}

/// Reads lines appended to a log file since the last poll.
pub struct LogTailer {
    path: PathBuf,
    offset: u64,
}

impl LogTailer {
    /// Tail `path` starting from its current end (or the start, if it does
    /// not exist yet).
    pub fn from_end(path: PathBuf) -> Self {
        let offset = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        Self { path, offset }
    }

    /// Tail `path` from its first line.
    pub fn from_start(path: PathBuf) -> Self {
        Self { path, offset: 0 }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// New complete lines since the last poll, keeping at most the last
    /// `max_lines`.
    pub fn poll(&mut self, max_lines: usize) -> Vec<String> {
        let Ok(file) = File::open(&self.path) else {
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

/// How many session logs to keep when pruning.
///
/// Enough to look back over a few daemon restarts, few enough that the
/// directory stays readable.
pub const KEEP_LOGS: usize = 20;

/// Every session log, oldest first.
///
/// Names begin with a sortable UTC timestamp, so sorting them is sorting by
/// age. This is the one place that knows what a session log looks like on
/// disk — callers take an [`AppPaths`] and ask here.
pub fn session_logs(paths: &AppPaths) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(paths.logs_dir()) else {
        return Vec::new();
    };
    let mut logs: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "log"))
        .collect();
    logs.sort();
    logs
}

/// Delete old session logs, keeping the newest `keep` that have content.
///
/// Empty logs go first and do not count against `keep`: they are left by
/// processes that opened a log and never wrote to it, and on one machine they
/// were 1,505 of 1,522 files.
///
/// The log this process is writing is never removed — deleting it would
/// strand everything logged afterwards.
///
/// Returns how many were removed. Errors are ignored throughout: tidying up
/// must not stop the daemon from starting.
pub fn prune(paths: &AppPaths, keep: usize) -> usize {
    prune_keeping(paths, keep, session_log())
}

/// [`prune`], with the live log named explicitly.
///
/// `prune` reads it from this process's session, which is a set-once global.
/// Taking it as an argument is what lets a test cover the "never delete the
/// log being written" rule without writing to that global and leaking into
/// every other test in the binary.
pub fn prune_keeping(paths: &AppPaths, keep: usize, live: Option<&Path>) -> usize {
    let mut kept = Vec::new();
    let mut removed = 0;

    for path in session_logs(paths) {
        if live.is_some_and(|live| live == path) {
            continue;
        }
        match std::fs::metadata(&path) {
            Ok(meta) if meta.len() == 0 => {
                if std::fs::remove_file(&path).is_ok() {
                    removed += 1;
                }
            }
            Ok(_) => kept.push(path),
            Err(_) => {}
        }
    }

    let surplus = kept.len().saturating_sub(keep);
    for path in kept.into_iter().take(surplus) {
        if std::fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Delete every session log, including this process's. For `jig nuke`.
pub fn clear(paths: &AppPaths) -> usize {
    session_logs(paths)
        .into_iter()
        .filter(|path| std::fs::remove_file(path).is_ok())
        .count()
}

/// The last `n` lines of a file.
pub fn tail_lines(path: &Path, n: usize) -> std::io::Result<Vec<String>> {
    let reader = BufReader::new(File::open(path)?);
    let mut lines = std::collections::VecDeque::with_capacity(n);
    for line in reader.lines() {
        let line = line?;
        if lines.len() == n {
            lines.pop_front();
        }
        if n > 0 {
            lines.push_back(line);
        }
    }
    Ok(lines.into())
}
