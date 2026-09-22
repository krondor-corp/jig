//! Notification queue — append-only JSONL file.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::context::AppPaths;

use super::{Notification, NotificationEvent};

/// Append-only notification queue backed by a JSONL file.
pub struct NotificationQueue {
    path: PathBuf,
}

impl NotificationQueue {
    /// Queue at the global state dir (`~/.config/jig/state/notifications.jsonl`).
    pub fn global(paths: &AppPaths) -> Self {
        Self::new(paths.notifications())
    }

    /// Queue at a specific path (useful for testing).
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Append a notification to the queue.
    pub fn emit(&self, event: NotificationEvent) -> Result<(), super::NotifyError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        tracing::debug!(event_type = event.type_name(), "queuing notification");
        let notification = Notification::now(event);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::to_string(&notification)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Read notifications newer than the given timestamp.
    pub fn read_since(&self, since_ts: i64) -> Result<Vec<Notification>, super::NotifyError> {
        let all = self.read_all()?;
        Ok(all.into_iter().filter(|n| n.ts > since_ts).collect())
    }

    /// Return the last N notifications.
    pub fn tail(&self, n: usize) -> Result<Vec<Notification>, super::NotifyError> {
        let all = self.read_all()?;
        let skip = all.len().saturating_sub(n);
        Ok(all.into_iter().skip(skip).collect())
    }

    /// Return the queue file path.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Check if the queue file exists.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    fn read_all(&self) -> Result<Vec<Notification>, super::NotifyError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut notifications = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if !line.is_empty() {
                notifications.push(serde_json::from_str(&line)?);
            }
        }
        Ok(notifications)
    }
}
