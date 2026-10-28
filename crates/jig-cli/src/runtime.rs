//! Service lifecycle and graceful shutdown for the daemon's async parts.
//!
//! The [`Service`] trait is `zim_peer::runtime`'s (krondor-corp/zim), which
//! that module took from krondor-corp/pack, and this is meant to become the
//! same shared crate. Keep it that way: tokio and tracing only, no jig types
//! in here.
//!
//! jig's daemon is std threads, so this differs from zim's `ShutdownHandle`
//! in three ways:
//!
//! * **The daemon owns signals.** It already handles SIGINT/SIGTERM, and a
//!   second handler would fight it. A [`Supervisor`] is shut down by its
//!   caller instead.
//! * **A service that ends early is logged, not fatal.** Everything that runs
//!   here (the web view) is optional, and the daemon outlives it.
//! * **One thread holds the async world.** [`Supervisor::start`] runs a
//!   current-thread tokio runtime on a named thread of its own. Nothing else
//!   in the daemon sees tokio.
//!
//! Draining is bounded, as in zim: [`Supervisor::shutdown`] returns within
//! its timeout even if a service never finishes, so `jig daemon stop` can't
//! hang on a slow request.

use std::future::Future;
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::sync::watch;
use tokio::task::{Id, JoinSet};

/// A long-running async component. It runs until `shutdown` changes, then
/// winds down and returns.
pub trait Service: Send + 'static {
    type State: Send + 'static;

    fn run(state: Self::State, shutdown: watch::Receiver<()>) -> impl Future<Output = ()> + Send;
}

/// What a [`Supervisor`]'s `register` closure spawns services into.
pub struct Services {
    shutdown: watch::Receiver<()>,
    set: JoinSet<()>,
    names: Vec<(Id, &'static str)>,
}

impl Services {
    /// Run `S` as `name`, on the supervisor's runtime.
    pub fn spawn<S: Service>(&mut self, name: &'static str, state: S::State) {
        let handle = self.set.spawn(S::run(state, self.shutdown.clone()));
        self.names.push((handle.id(), name));
    }

    fn name(&self, id: Id) -> &'static str {
        self.names
            .iter()
            .find(|(i, _)| *i == id)
            .map_or("service", |(_, n)| n)
    }
}

/// A thread running services on its own runtime until shut down.
pub struct Supervisor {
    shutdown: watch::Sender<()>,
    drain: watch::Sender<Duration>,
    thread: std::thread::JoinHandle<()>,
}

impl Supervisor {
    /// Start a thread called `thread`, build its runtime, and let `register`
    /// spawn the services that run there.
    pub fn start(
        thread: &str,
        register: impl FnOnce(&mut Services) + Send + 'static,
    ) -> std::io::Result<Self> {
        let (shutdown, shutdown_rx) = watch::channel(());
        let (drain, drain_rx) = watch::channel(Duration::ZERO);
        let thread = std::thread::Builder::new()
            .name(thread.to_string())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        tracing::warn!("no async runtime, so no services: {}", e);
                        return;
                    }
                };
                supervise(&runtime, shutdown_rx, &drain_rx, register);
                // A blocking call a service started (a daemon command, say)
                // may still be running; leave it rather than wait on it.
                runtime.shutdown_timeout(*drain_rx.borrow());
            })?;
        Ok(Self {
            shutdown,
            drain,
            thread,
        })
    }

    /// Signal every service to stop and wait for them, up to `drain`. A
    /// service still running after that is abandoned.
    pub fn shutdown(self, drain: Duration) {
        let _ = self.drain.send(drain);
        let _ = self.shutdown.send(());
        if self.thread.join().is_err() {
            tracing::warn!("service thread panicked");
        }
    }
}

fn supervise(
    runtime: &Runtime,
    mut shutdown: watch::Receiver<()>,
    drain: &watch::Receiver<Duration>,
    register: impl FnOnce(&mut Services),
) {
    runtime.block_on(async {
        let mut services = Services {
            shutdown: shutdown.clone(),
            set: JoinSet::new(),
            names: Vec::new(),
        };
        register(&mut services);

        // Until shutdown: report any service that stops on its own.
        loop {
            tokio::select! {
                _ = shutdown.changed() => break,
                Some(done) = services.set.join_next_with_id() => match done {
                    Ok((id, ())) => tracing::warn!(
                        service = services.name(id),
                        "service stopped before shutdown"
                    ),
                    Err(e) => tracing::warn!(
                        service = services.name(e.id()),
                        "service failed before shutdown: {}",
                        e
                    ),
                },
            }
        }

        let limit = *drain.borrow();
        let drained = tokio::time::timeout(limit, async {
            while services.set.join_next().await.is_some() {}
        })
        .await;
        match drained {
            Ok(()) => tracing::debug!("services drained"),
            Err(_) => {
                tracing::warn!(
                    "services still running {}s after shutdown — abandoning them",
                    limit.as_secs()
                );
                services.set.abort_all();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    use super::*;

    /// Counts its runs, then waits for shutdown.
    struct Polite;

    impl Service for Polite {
        type State = Arc<AtomicUsize>;

        async fn run(stopped: Self::State, mut shutdown: watch::Receiver<()>) {
            let _ = shutdown.changed().await;
            stopped.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Ignores shutdown entirely.
    struct Stubborn;

    impl Service for Stubborn {
        type State = ();

        async fn run((): Self::State, _shutdown: watch::Receiver<()>) {
            std::future::pending::<()>().await;
        }
    }

    /// Returns at once, long before shutdown.
    struct Quitter;

    impl Service for Quitter {
        type State = ();

        async fn run((): Self::State, _shutdown: watch::Receiver<()>) {}
    }

    #[test]
    fn services_stop_on_shutdown() {
        let stopped = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&stopped);
        let supervisor = Supervisor::start("test-services", move |s| {
            s.spawn::<Polite>("one", Arc::clone(&counter));
            s.spawn::<Polite>("two", counter);
        })
        .unwrap();
        supervisor.shutdown(Duration::from_secs(5));
        assert_eq!(stopped.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn a_service_that_ignores_shutdown_is_abandoned_within_the_drain() {
        let supervisor = Supervisor::start("test-stubborn", |s| {
            s.spawn::<Stubborn>("stubborn", ());
        })
        .unwrap();
        let started = Instant::now();
        supervisor.shutdown(Duration::from_millis(200));
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn a_service_ending_early_leaves_the_others_running() {
        let stopped = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&stopped);
        let supervisor = Supervisor::start("test-early", move |s| {
            s.spawn::<Quitter>("quitter", ());
            s.spawn::<Polite>("polite", counter);
        })
        .unwrap();
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(stopped.load(Ordering::SeqCst), 0, "still running");
        supervisor.shutdown(Duration::from_secs(5));
        assert_eq!(stopped.load(Ordering::SeqCst), 1);
    }
}
