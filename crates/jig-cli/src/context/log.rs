//! Session log files — where `ps --watch` and the daemon send their tracing
//! output — and a tailer for following one as it grows.

use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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

/// Delete old session logs, keeping the newest `keep` that have content.
///
/// Empty logs go first and do not count against `keep`: most of them are from
/// commands that opened a log and never wrote to it, and on one machine they
/// were 1,505 of 1,522 files.
///
/// `in_use` is never removed. The daemon writes to its log for as long as it
/// runs, and deleting that file out from under it would strand everything it
/// logged afterwards.
///
/// Returns how many were removed. Errors are ignored throughout: tidying up
/// must not stop the daemon from starting.
pub fn prune(dir: &Path, keep: usize, in_use: Option<&Path>) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };

    let mut kept: Vec<PathBuf> = Vec::new();
    let mut removed = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "log") {
            continue;
        }
        if in_use.is_some_and(|live| live == path) {
            continue;
        }
        match entry.metadata() {
            Ok(meta) if meta.len() == 0 => {
                if std::fs::remove_file(&path).is_ok() {
                    removed += 1;
                }
            }
            Ok(_) => kept.push(path),
            Err(_) => {}
        }
    }

    // Names begin with a sortable UTC timestamp, so this is oldest first.
    kept.sort();
    let surplus = kept.len().saturating_sub(keep);
    for path in kept.into_iter().take(surplus) {
        if std::fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    removed
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
