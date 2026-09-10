//! Actor trait and generic handle for daemon background workers.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// The actor struct IS the state. Define your struct, implement this trait,
/// and `ActorHandle<A>` manages channels, threading, and pending tracking.
///
/// Use interior mutability (`Mutex`, `AtomicBool`, etc.) for fields that
/// need mutation — `handle(&self)` takes a shared reference because it runs
/// on the background thread while the main thread may read actor state.
pub trait Actor: Default + Send + Sync + 'static {
    type Request: Send + 'static;
    type Response: Send + 'static;

    const NAME: &'static str;
    const QUEUE_SIZE: usize;

    fn handle(&self, req: Self::Request) -> Self::Response;
}

/// Point-in-time view of an actor's activity, for the daemon heartbeat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorActivity {
    pub name: String,
    /// Unix timestamp the in-flight request started, if one is running.
    pub busy_since: Option<i64>,
    /// Unix timestamp the last request finished, if any has.
    pub last_finished: Option<i64>,
}

/// Unix timestamps written by the actor thread; 0 means "none".
#[derive(Default)]
struct Timings {
    /// Start of the in-flight request; reset to 0 when it finishes.
    busy_since: AtomicI64,
    last_finished: AtomicI64,
}

/// Generic wrapper that owns channels, the background thread, and a shared
/// reference to the actor. The main thread uses `send()` for fire-and-forget
/// dispatch, `drain()` to collect responses, and `.actor()` to read actor
/// state directly.
pub struct ActorHandle<A: Actor> {
    tx: flume::Sender<A::Request>,
    rx: flume::Receiver<A::Response>,
    inner: Arc<A>,
    pending: Arc<AtomicBool>,
    timings: Arc<Timings>,
    _handle: std::thread::JoinHandle<()>,
}

impl<A: Actor> ActorHandle<A> {
    pub fn new() -> Self {
        let (req_tx, req_rx) = flume::bounded::<A::Request>(A::QUEUE_SIZE);
        let (resp_tx, resp_rx) = flume::bounded::<A::Response>(A::QUEUE_SIZE);
        let inner = Arc::new(A::default());
        let bg = Arc::clone(&inner);
        let pending = Arc::new(AtomicBool::new(false));
        let bg_pending = Arc::clone(&pending);
        let timings = Arc::new(Timings::default());
        let bg_timings = Arc::clone(&timings);
        let handle = std::thread::Builder::new()
            .name(A::NAME.into())
            .spawn(move || {
                while let Ok(req) = req_rx.recv() {
                    let now = chrono::Utc::now().timestamp();
                    bg_timings.busy_since.store(now, Ordering::Relaxed);
                    let resp = bg.handle(req);
                    let now = chrono::Utc::now().timestamp();
                    bg_timings.last_finished.store(now, Ordering::Relaxed);
                    bg_timings.busy_since.store(0, Ordering::Relaxed);
                    bg_pending.store(false, Ordering::Relaxed);
                    if resp_tx.send(resp).is_err() {
                        break;
                    }
                }
            })
            .unwrap_or_else(|e| panic!("failed to spawn {} thread: {}", A::NAME, e));

        Self {
            tx: req_tx,
            rx: resp_rx,
            inner,
            pending,
            timings,
            _handle: handle,
        }
    }
}

impl<A: Actor> Default for ActorHandle<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Actor> ActorHandle<A> {
    pub fn send(&self, req: A::Request) -> bool {
        if self.pending.load(Ordering::Relaxed) {
            return false;
        }
        if self.tx.try_send(req).is_ok() {
            self.pending.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn drain(&self) -> Vec<A::Response> {
        let mut results = Vec::new();
        while let Ok(resp) = self.rx.try_recv() {
            results.push(resp);
        }
        results
    }

    pub fn is_pending(&self) -> bool {
        self.pending.load(Ordering::Relaxed)
    }

    pub fn actor(&self) -> &A {
        &self.inner
    }

    /// What the actor thread is doing right now. `busy_since` makes a wedged
    /// actor visible even while the tick loop keeps going — `send()` just
    /// drops requests while one is in flight, so a hung `handle` is silent.
    pub fn activity(&self) -> ActorActivity {
        let since = |t: &AtomicI64| Some(t.load(Ordering::Relaxed)).filter(|&ts| ts != 0);
        ActorActivity {
            name: A::NAME.to_string(),
            busy_since: since(&self.timings.busy_since),
            last_finished: since(&self.timings.last_finished),
        }
    }
}
