//! Session log files — the tracing sink for each jig invocation, and a
//! tailer for following one as it grows (`ps --watch`, `daemon logs -f`).

use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
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

/// A log file created on first write, so commands that log nothing leave
/// nothing behind.
pub struct LazyFile {
    path: PathBuf,
    file: Option<File>,
}

impl LazyFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path, file: None }
    }
}

impl Write for LazyFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.file.is_none() {
            self.file = Some(File::create(&self.path)?);
        }
        match self.file.as_mut() {
            Some(file) => file.write(buf),
            None => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.file.as_mut() {
            Some(file) => file.flush(),
            None => Ok(()),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_file_not_created_until_written() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("session.log");
        let mut file = LazyFile::new(path.clone());
        file.flush().unwrap();
        assert!(!path.exists());

        writeln!(file, "hello").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello\n");
    }

    #[test]
    fn tailer_returns_only_appended_complete_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("d.log");
        std::fs::write(&path, "old\n").unwrap();

        let mut tailer = LogTailer::from_end(path.clone());
        assert!(tailer.poll(10).is_empty());

        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        write!(f, "one\ntw").unwrap();
        assert_eq!(tailer.poll(10), vec!["one"]);

        writeln!(f, "o").unwrap();
        assert_eq!(tailer.poll(10), vec!["two"]);
    }

    #[test]
    fn tailer_restarts_after_truncation() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("d.log");
        std::fs::write(&path, "a long first line\n").unwrap();
        let mut tailer = LogTailer::from_end(path.clone());

        std::fs::write(&path, "new\n").unwrap();
        assert_eq!(tailer.poll(10), vec!["new"]);
    }

    #[test]
    fn tailer_waits_for_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("later.log");
        let mut tailer = LogTailer::from_end(path.clone());
        assert!(tailer.poll(10).is_empty());

        std::fs::write(&path, "hi\n").unwrap();
        assert_eq!(tailer.poll(10), vec!["hi"]);
    }

    #[test]
    fn tail_lines_keeps_last_n() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("d.log");
        std::fs::write(&path, "1\n2\n3\n4\n").unwrap();
        assert_eq!(tail_lines(&path, 2).unwrap(), vec!["3", "4"]);
        assert_eq!(tail_lines(&path, 10).unwrap().len(), 4);
        assert!(tail_lines(&path, 0).unwrap().is_empty());
    }
}
