//! Daemon loop — the conductor that ties actors together.
//!
//! Runs a periodic loop:
//! 1. Send monitor request every tick (worker discovery, health, nudges, notifications)
//! 2. Drain prune targets from monitor responses → feed to prune actor
//! 3. Trigger background sync + spawn + triage if poll interval elapsed

pub mod actors;
pub mod checks;
pub mod commands;
pub mod events;
pub mod ipc;
pub mod pidfile;
pub mod web;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::context::log::Logs;
use crate::context::{
    AppConfig, AppCtx, AppPaths, ContextError, Ctx, Machine, RepoEntry, RepoRegistry,
};
use jig_core::mux;

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error(transparent)]
    Ctx(#[from] ContextError),
    #[error(transparent)]
    Worker(#[from] crate::worker::WorkerError),
    #[error(transparent)]
    Notify(#[from] crate::notify::NotifyError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Shared context built once per daemon tick, passed to all actors.
#[derive(Clone)]
pub struct TickContext {
    pub app: Arc<AppCtx>,
    pub repos: Arc<Vec<RepoEntry>>,
    pub session_prefix: String,
}

type Worker = crate::worker::Worker;

use actors::monitor::MonitorActor;
use actors::prune::PruneActor;
use actors::spawn::SpawnActor;
use actors::sync::SyncActor;
use actors::triage::TriageActor;
use actors::{ActivityProbe, ActorHandle};
use commands::{Command, InFlight, WorkerKey};
use ipc::Response;

pub use crate::worker::events::WorkerState;
pub use actors::triage::TriageEntry;
pub use checks::{PrChecks, PrHealth};
pub use ipc::{DaemonInfo, DaemonStatus, WorkerSnapshot};

/// Everything the IPC listener needs, without the `Daemon` itself.
///
/// The listener runs on its own thread and must never block a tick (nor be
/// blocked by one). Actors already hold their state behind interior
/// mutability, so sharing the `Arc`s plus a couple of atomics is enough —
/// no `Arc<Mutex<Daemon>>`, and no chance of a slow status request wedging
/// the loop.
#[derive(Clone)]
pub struct DaemonShared {
    monitor: Arc<MonitorActor>,
    triage: Arc<TriageActor>,
    spawn: Arc<SpawnActor>,
    probes: Arc<Vec<ActivityProbe>>,
    /// Unix timestamp of the last completed tick.
    ticked_at: Arc<AtomicI64>,
    /// Unix timestamp the next poll tick is due.
    poll_deadline: Arc<AtomicI64>,
    started_at: i64,
    tick_interval: u64,
    log: Option<PathBuf>,
    machine: Machine,
    /// The web view's address, once it is listening.
    web: Option<String>,
    /// What control verbs run against.
    app: Arc<AppCtx>,
    in_flight: InFlight,
}

impl DaemonShared {
    /// Run a control verb — the answer to `Kill`, `Nudge`, … Called on the
    /// command's own thread, never the listener's or the tick loop's.
    pub fn run_command(&self, worker: WorkerKey, cmd: Command) -> Response {
        commands::serve(&self.app, &self.in_flight, worker, cmd)
    }

    /// Identity and liveness — the answer to `Request::Ping`.
    pub fn info(&self) -> DaemonInfo {
        DaemonInfo {
            pid: std::process::id(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            started_at: self.started_at,
            ticked_at: self.ticked_at.load(Ordering::Relaxed),
            tick_interval: self.tick_interval,
            log: self.log.clone(),
            actors: self.probes.iter().map(ActivityProbe::activity).collect(),
            machine: Some(self.machine.clone()),
            web: self.web.clone(),
        }
    }

    /// The full frame `jig ps` renders — the answer to `Request::GetStatus`.
    pub fn status(&self) -> DaemonStatus {
        let now = chrono::Utc::now().timestamp();
        DaemonStatus {
            info: self.info(),
            workers: self.monitor.workers().iter().map(Into::into).collect(),
            triages: self.triage.active_entries(),
            spawning: self.spawn.spawning_workers(),
            poll_remaining: (self.poll_deadline.load(Ordering::Relaxed) - now).max(0) as u64,
        }
    }
}

/// The daemon — owns actors and drives the tick loop.
pub struct Daemon {
    pub sync: ActorHandle<SyncActor>,
    pub monitor: ActorHandle<MonitorActor>,
    pub prune: ActorHandle<PruneActor>,
    pub spawn: ActorHandle<SpawnActor>,
    pub triage: ActorHandle<TriageActor>,

    app: Arc<AppCtx>,
    registry: RepoRegistry,
    last_poll: Instant,
    shared: DaemonShared,
}

impl Daemon {
    /// Start the daemon: records lifecycle events in `daemon.jsonl` and
    /// serves IPC via [`Self::shared`].
    ///
    /// There is one of these. `jig ps` used to build a second, unnamed one to
    /// render a table — ticking, spawning and pruning as a side effect of
    /// being looked at. It reads state off disk instead.
    pub fn start(cfg: Ctx) -> Result<Self, DaemonError> {
        let app = Arc::new(AppCtx {
            paths: cfg.paths,
            config: cfg.config,
            flags: cfg.flags,
        });
        log_startup(&app.paths);
        let machine = Machine::load(&app.paths, &app.config)?;
        tracing::info!(name = %machine.name, node_id = %machine.node_id, "machine identity");
        recover_orphans(&app, &cfg.registry);

        let last_poll = Instant::now() - Duration::from_secs(app.config.poll_interval + 1);
        let started_at = chrono::Utc::now().timestamp();

        let sync = ActorHandle::<SyncActor>::new();
        let monitor = ActorHandle::<MonitorActor>::new();
        let prune = ActorHandle::<PruneActor>::new();
        let spawn = ActorHandle::<SpawnActor>::new();
        let triage = ActorHandle::<TriageActor>::new();

        let shared = DaemonShared {
            monitor: monitor.shared(),
            triage: triage.shared(),
            spawn: spawn.shared(),
            probes: Arc::new(vec![
                monitor.probe(),
                sync.probe(),
                prune.probe(),
                spawn.probe(),
                triage.probe(),
            ]),
            ticked_at: Arc::new(AtomicI64::new(started_at)),
            poll_deadline: Arc::new(AtomicI64::new(started_at)),
            started_at,
            tick_interval: app.config.tick_interval,
            log: crate::context::log::session_log().map(|l| l.path().to_path_buf()),
            machine,
            web: None,
            app: Arc::clone(&app),
            in_flight: InFlight::default(),
        };

        Ok(Self {
            sync,
            monitor,
            prune,
            spawn,
            triage,
            app,
            registry: cfg.registry,
            last_poll,
            shared,
        })
    }

    pub fn config(&self) -> &AppConfig {
        &self.app.config
    }

    /// Advertise the web view at `url` in `Ping`/`Status`. Call before
    /// handing out [`Self::shared`]: earlier handles keep the old value.
    pub fn set_web_url(&mut self, url: String) {
        self.shared.web = Some(url);
    }

    /// A handle the IPC listener can hold while the tick loop keeps running.
    pub fn shared(&self) -> DaemonShared {
        self.shared.clone()
    }

    /// The frame `jig ps` renders, as the daemon would answer it over IPC.
    pub fn snapshot(&self) -> DaemonStatus {
        self.shared.status()
    }

    /// Default bound on [`Self::wait_for_monitor`].
    pub const MONITOR_WAIT: Duration = Duration::from_secs(30);

    /// Block until the monitor pass `tick()` dispatched finishes, or
    /// `timeout` elapses. Returns whether it finished.
    ///
    /// A caller that reads worker state after a single tick must do this:
    /// the pass runs on the monitor's own thread, so returning immediately
    /// reads the pre-tick (empty) state.
    pub fn wait_for_monitor(&self, timeout: Duration) -> bool {
        let start = Instant::now();
        while self.monitor.is_pending() {
            if start.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        true
    }

    /// Run the tick loop. Checks `quit` between ticks; calls `on_tick` after
    /// each successful tick — return `false` to stop.
    pub fn run<F>(&mut self, quit: &AtomicBool, mut on_tick: F)
    where
        F: FnMut(&Self) -> bool,
    {
        loop {
            match self.tick() {
                Ok(()) => {
                    if quit.load(Ordering::Relaxed) {
                        break;
                    }
                    if !on_tick(self) {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("tick failed: {}", e);
                    if quit.load(Ordering::Relaxed) {
                        break;
                    }
                    std::thread::sleep(Duration::from_secs(self.app.config.tick_interval));
                }
            }
        }
        log_shutdown(&self.app.paths, "normal");
    }

    /// Whether the poll interval has elapsed.
    pub fn poll_is_due(&self) -> bool {
        self.last_poll.elapsed().as_secs() >= self.app.config.poll_interval
    }

    /// Mark that a poll tick just fired.
    fn mark_polled(&mut self) {
        self.last_poll = Instant::now();
        self.shared.poll_deadline.store(
            chrono::Utc::now().timestamp() + self.app.config.poll_interval as i64,
            Ordering::Relaxed,
        );
    }

    /// Seconds until the next poll tick.
    pub fn poll_remaining_secs(&self) -> u64 {
        self.app
            .config
            .poll_interval
            .saturating_sub(self.last_poll.elapsed().as_secs())
    }

    /// Execute a single tick of the daemon.
    pub fn tick(&mut self) -> Result<(), DaemonError> {
        let ctx = TickContext {
            app: Arc::clone(&self.app),
            repos: Arc::new(self.registry.repos().to_vec()),
            session_prefix: self.app.config.session_prefix.clone(),
        };

        // Send monitor request every tick
        self.monitor
            .send(actors::monitor::MonitorRequest { ctx: ctx.clone() });

        // Drain prune targets from completed monitor passes
        let prune_targets: Vec<_> = self.monitor.drain().into_iter().flatten().collect();
        if !prune_targets.is_empty() {
            self.prune.send(actors::prune::PruneRequest {
                paths: self.app.paths.clone(),
                targets: prune_targets,
            });
        }

        // Trigger background sync + spawn + triage if interval elapsed
        if self.poll_is_due() {
            self.sync
                .send(actors::sync::SyncRequest { ctx: ctx.clone() });
            self.spawn
                .send(actors::spawn::SpawnRequest { ctx: ctx.clone() });
            self.triage
                .send(actors::triage::TriageRequest { ctx: ctx.clone() });
            self.mark_polled();
        }

        // Published last, so a stale `ticked_at` over IPC means the tick
        // itself wedged rather than that we simply have not started one.
        self.shared
            .ticked_at
            .store(chrono::Utc::now().timestamp(), Ordering::Relaxed);

        Ok(())
    }
}

/// Log the Started lifecycle event, noting if the previous run crashed.
fn log_startup(paths: &AppPaths) {
    let log = Logs::new(paths).daemon_events();

    match log.reduce() {
        Ok(state) => {
            if state.previous_run_crashed() {
                tracing::warn!(
                    "previous daemon run did not shut down cleanly — checking for orphaned workers"
                );
            }
        }
        // `reduce` skips lines it can't parse, so this is an unreadable
        // file, not a bad line — the history is already lost.
        Err(e) => {
            tracing::warn!("unreadable daemon event log, resetting: {}", e);
            let _ = log.reset();
        }
    }

    let session_log = crate::context::log::session_log().map(|l| l.path().to_path_buf());
    if let Err(e) = log.append(&events::started(session_log)) {
        tracing::warn!("failed to write daemon Started event: {}", e);
    }
}

/// Resume workers whose mux window died, if `auto_recover` is on.
fn recover_orphans(app: &AppCtx, registry: &RepoRegistry) {
    if app.config.auto_recover {
        let mut recovered = Vec::new();
        for entry in registry.repos() {
            let Ok(repo) = app.repo_at(&entry.path) else {
                continue;
            };
            let Ok(workers) = Worker::discover(&repo) else {
                continue;
            };
            let repo_name = repo.name();
            let mux = mux::for_repo(app.config.mux, &repo_name);
            for worker in workers {
                if worker.is_orphaned(&mux) {
                    let branch = worker.branch().to_string();
                    match worker.recover(&repo, &mux) {
                        Ok(true) => {
                            tracing::info!(repo = %repo_name, worker = %branch, "recovered");
                            recovered.push((repo_name.clone(), branch));
                        }
                        Ok(false) => {}
                        Err(e) => {
                            tracing::warn!(repo = %repo_name, worker = %branch, error = %e, "recovery failed");
                        }
                    }
                }
            }
        }
        if !recovered.is_empty() {
            tracing::info!(
                count = recovered.len(),
                "recovered orphaned workers on startup"
            );
        }
    }
}

/// Log a graceful shutdown event.
fn log_shutdown(paths: &AppPaths, reason: &str) {
    if let Err(e) = Logs::new(paths)
        .daemon_events()
        .append(&events::stopped(reason))
    {
        tracing::warn!("failed to write daemon Stopped event: {}", e);
    }
}

#[cfg(test)]
mod tests {

    fn should_auto_complete(
        auto_complete_on_merge: bool,
        issue_ref: Option<&str>,
    ) -> Option<String> {
        if auto_complete_on_merge {
            issue_ref.map(|id| id.to_string())
        } else {
            None
        }
    }

    #[test]
    fn auto_complete_pushes_when_enabled_and_has_issue() {
        let result = should_auto_complete(true, Some("ENG-42"));
        assert_eq!(result, Some("ENG-42".to_string()));
    }

    #[test]
    fn auto_complete_skips_when_disabled() {
        let result = should_auto_complete(false, Some("ENG-42"));
        assert_eq!(result, None);
    }

    #[test]
    fn auto_complete_skips_when_no_issue() {
        let result = should_auto_complete(true, None);
        assert_eq!(result, None);
    }

    fn should_update_issue_status(current_status: jig_core::issues::issue::IssueStatus) -> bool {
        !matches!(
            current_status,
            jig_core::issues::issue::IssueStatus::Complete
        )
    }

    #[test]
    fn auto_complete_updates_in_progress_issue() {
        assert!(should_update_issue_status(
            jig_core::issues::issue::IssueStatus::InProgress
        ));
    }

    #[test]
    fn auto_complete_skips_already_complete_issue() {
        assert!(!should_update_issue_status(
            jig_core::issues::issue::IssueStatus::Complete
        ));
    }

    #[test]
    fn auto_complete_updates_planned_issue() {
        assert!(should_update_issue_status(
            jig_core::issues::issue::IssueStatus::Planned
        ));
    }
}
