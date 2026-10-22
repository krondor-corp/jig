//! Generic append-only JSONL event log.

use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum EventLogError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// A single timestamped event with a flattened payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<K> {
    pub ts: i64,
    #[serde(flatten)]
    pub kind: K,
}

impl<K> Event<K> {
    pub fn now(kind: K) -> Self {
        Self {
            ts: chrono::Utc::now().timestamp(),
            kind,
        }
    }
}

/// An event type that knows how to fold into a state.
///
/// Implement on your event enum, then call [`EventLog::reduce()`] to
/// replay the log into a fresh `State`.
pub trait Reducible {
    type State: Default;
    fn apply(state: &mut Self::State, event: &Self);
}

/// Like [`Reducible`], but for the payload of an [`Event<K>`] wrapper.
///
/// Implement this on your `EventKind` enum; the blanket impl below then
/// satisfies `Reducible for Event<K>`, which [`EventLog::reduce()`] needs.
/// The `ts` parameter gives access to the outer timestamp that would otherwise
/// be unavailable when only the inner kind is passed.
pub trait ReducibleKind {
    type State: Default;
    fn apply(state: &mut Self::State, ts: i64, kind: &Self);
}

impl<K: ReducibleKind> Reducible for Event<K> {
    type State = K::State;
    fn apply(state: &mut Self::State, event: &Self) {
        K::apply(state, event.ts, &event.kind);
    }
}

/// Append-only JSONL event log, generic over event type.
pub struct EventLog<E> {
    pub path: PathBuf,
    _phantom: std::marker::PhantomData<E>,
}

// By hand: deriving would demand `E: Clone + Debug`, and `E` is only a
// marker for what the file holds.
impl<E> Clone for EventLog<E> {
    fn clone(&self) -> Self {
        Self::new(self.path.clone())
    }
}

impl<E> fmt::Debug for EventLog<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventLog")
            .field("path", &self.path)
            .finish()
    }
}

impl<E> EventLog<E> {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn reset(&self) -> Result<(), EventLogError> {
        if self.path.exists() {
            fs::write(&self.path, "")?;
        }
        Ok(())
    }

    pub fn remove(&self) -> Result<(), EventLogError> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        if let Some(parent) = self.path.parent() {
            let _ = fs::remove_dir(parent);
        }
        Ok(())
    }
}

impl<E: Serialize> EventLog<E> {
    pub fn append(&self, event: &E) -> Result<(), EventLogError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::to_string(event)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }
}

impl<E: DeserializeOwned + Reducible> EventLog<E> {
    /// Replay all events into the event's associated [`Reducible::State`].
    pub fn reduce(&self) -> Result<E::State, EventLogError> {
        let events = self.read_all()?;
        let mut state = E::State::default();
        for event in &events {
            E::apply(&mut state, event);
        }
        Ok(state)
    }
}

impl<E: DeserializeOwned> EventLog<E> {
    /// Every event in the log, oldest first.
    ///
    /// Lines that don't deserialize as `E` — a torn write from a crash, or
    /// a line in a shape some other build wrote — are skipped with a
    /// `debug!`, so one bad line doesn't cost the rest of the history.
    /// Only I/O errors fail the read.
    pub fn read_all(&self) -> Result<Vec<E>, EventLogError> {
        let mut events = Vec::new();
        self.for_each_parsed(|event| events.push(event))?;
        Ok(events)
    }

    /// The last event in the log that deserializes, skipping bad lines
    /// like [`Self::read_all`].
    pub fn last_event(&self) -> Result<Option<E>, EventLogError> {
        let mut last = None;
        self.for_each_parsed(|event| last = Some(event))?;
        Ok(last)
    }

    fn for_each_parsed(&self, mut f: impl FnMut(E)) -> Result<(), EventLogError> {
        if !self.path.exists() {
            return Ok(());
        }
        let file = fs::File::open(&self.path)?;
        // Split on raw bytes so a line of invalid UTF-8 is skipped like any
        // other bad line rather than failing the read.
        for (i, line) in BufReader::new(file).split(b'\n').enumerate() {
            let line = line?;
            if line.trim_ascii().is_empty() {
                continue;
            }
            match serde_json::from_slice(&line) {
                Ok(event) => f(event),
                Err(e) => tracing::debug!(
                    path = %self.path.display(),
                    line = i + 1,
                    error = %e,
                    "skipping unparseable event log line"
                ),
            }
        }
        Ok(())
    }
}
