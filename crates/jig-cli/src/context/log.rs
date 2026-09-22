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
