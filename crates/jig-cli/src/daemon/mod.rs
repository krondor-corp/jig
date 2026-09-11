//! Daemon loop — the conductor that ties actors together.
//!
//! Runs a periodic loop:
//! 1. Send monitor request every tick (worker discovery, health, nudges, notifications)
//! 2. Drain prune targets from monitor responses → feed to prune actor
//! 3. Trigger background sync + spawn + triage if poll interval elapsed

pub mod actors;
pub mod checks;
pub mod events;
pub mod ipc;
pub mod pidfile;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::context::{Config, Context, ContextError, JigToml, RepoEntry, RepoRegistry};

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error(transparent)]
    Context(#[from] ContextError),
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
    pub config: Arc<Config>,
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
}

impl DaemonShared {
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

    config: Config,
    registry: RepoRegistry,
    last_poll: Instant,
    /// Long-running (`jig daemon start`, `ps --watch`) rather than a single
    /// one-shot tick. Only long-running daemons record lifecycle events.
    persistent: bool,
    shared: DaemonShared,
}

impl Daemon {
    /// Start a long-running daemon: records lifecycle events in
    /// `daemon.jsonl` and can serve IPC via [`Self::shared`].
    pub fn start(cfg: Context) -> Result<Self, DaemonError> {
        Self::new(cfg, true)
    }

    /// Start a daemon for a one-shot tick (plain `jig ps` with no daemon
    /// running). Leaves the lifecycle log to any long-running daemon.
    pub fn oneshot(cfg: Context) -> Result<Self, DaemonError> {
        Self::new(cfg, false)
    }

    fn new(cfg: Context, persistent: bool) -> Result<Self, DaemonError> {
        if persistent {
            log_startup();
        }
        recover_orphans(&cfg.config, &cfg.registry);
        let _notifier = make_notifier(&cfg.config)?;

        let last_poll = Instant::now() - Duration::from_secs(cfg.config.poll_interval + 1);
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
            tick_interval: cfg.config.tick_interval,
            log: crate::context::log::session_log().map(Into::into),
        };

        Ok(Self {
            sync,
            monitor,
            prune,
            spawn,
            triage,
            config: cfg.config,
            registry: cfg.registry,
            last_poll,
            persistent,
            shared,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// A handle the IPC listener can hold while the tick loop keeps running.
    pub fn shared(&self) -> DaemonShared {
        self.shared.clone()
    }

    /// The frame `jig ps` renders, as the daemon would answer it over IPC.
    pub fn snapshot(&self) -> DaemonStatus {
        self.shared.status()
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
                    std::thread::sleep(Duration::from_secs(self.config.tick_interval));
                }
            }
        }
        if self.persistent {
            log_shutdown("normal");
        }
    }

    /// Whether the poll interval has elapsed.
    pub fn poll_is_due(&self) -> bool {
        self.last_poll.elapsed().as_secs() >= self.config.poll_interval
    }

    /// Mark that a poll tick just fired.
    fn mark_polled(&mut self) {
        self.last_poll = Instant::now();
        self.shared.poll_deadline.store(
            chrono::Utc::now().timestamp() + self.config.poll_interval as i64,
            Ordering::Relaxed,
        );
    }

    /// Seconds until the next poll tick.
    pub fn poll_remaining_secs(&self) -> u64 {
        self.config
            .poll_interval
            .saturating_sub(self.last_poll.elapsed().as_secs())
    }

    /// Execute a single tick of the daemon.
    pub fn tick(&mut self) -> Result<(), DaemonError> {
        let ctx = TickContext {
            config: Arc::new(self.config.clone()),
            repos: Arc::new(self.registry.repos().to_vec()),
            session_prefix: self.config.session_prefix.clone(),
        };

        // Send monitor request every tick
        self.monitor
            .send(actors::monitor::MonitorRequest { ctx: ctx.clone() });

        // Drain prune targets from completed monitor passes
        let prune_targets: Vec<_> = self.monitor.drain().into_iter().flatten().collect();
        if !prune_targets.is_empty() {
            self.prune.send(actors::prune::PruneRequest {
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

/// Try to resume a worker whose mux window is dead.
fn try_resume_worker(
    repo_root: &std::path::Path,
    worker_name: &str,
    mux: &dyn jig_core::mux::Mux,
) -> Result<bool, DaemonError> {
    let worker = Worker::from_branch(repo_root, worker_name.into());
    if worker.has_mux_window(mux) {
        return Ok(false);
    }
    let wt = worker.worktree()?;
    let jig_config = JigToml::load(repo_root)?.unwrap_or_default();
    let agent = jig_core::agents::Agent::from_config(
        &jig_config.agent.agent_type,
        jig_config.agent.model.as_deref(),
        &jig_config.agent.disallowed_tools,
    )
    .unwrap_or_else(|| jig_core::agents::Agent::from_config("claude", None, &[]).unwrap());
    let prompt = crate::prompts::resume_task("You were interrupted. Resume your previous task.");
    Worker::resume(&wt, &agent, prompt, mux)?;
    Ok(true)
}

/// Build a Notifier from global config.
fn make_notifier(global_config: &Config) -> Result<crate::notify::Notifier, DaemonError> {
    let queue = crate::notify::NotificationQueue::global()?;
    Ok(crate::notify::Notifier::new(
        global_config.notify.clone(),
        queue,
    ))
}

/// Log the Started lifecycle event, noting if the previous run crashed.
fn log_startup() {
    let log = match events::global() {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("failed to open daemon event log: {}", e);
            return;
        }
    };

    match log.reduce() {
        Ok(state) => {
            if state.previous_run_crashed() {
                tracing::warn!(
                    "previous daemon run did not shut down cleanly — checking for orphaned workers"
                );
            }
        }
        Err(e) => {
            tracing::warn!("stale daemon event log, resetting: {}", e);
            let _ = log.reset();
        }
    }

    if let Err(e) = log.append(&events::started()) {
        tracing::warn!("failed to write daemon Started event: {}", e);
    }
}

/// Resume workers whose mux window died, if `auto_recover` is on.
fn recover_orphans(global_config: &Config, registry: &RepoRegistry) {
    if global_config.auto_recover {
        let mut recovered = Vec::new();
        for entry in registry.repos() {
            let repo_name = entry
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let repo = match jig_core::git::Repo::open(&entry.path) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let mux = jig_core::mux::for_repo(global_config.mux, &repo_name);
            for worker in Worker::discover(&repo) {
                if worker.is_orphaned(&mux) {
                    let branch = worker.branch().to_string();
                    match try_resume_worker(&entry.path, &branch, &mux) {
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
fn log_shutdown(reason: &str) {
    let log = match events::global() {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("failed to open daemon event log: {}", e);
            return;
        }
    };
    if let Err(e) = log.append(&events::stopped(reason)) {
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
