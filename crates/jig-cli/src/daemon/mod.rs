//! Daemon loop — the conductor that ties actors together.
//!
//! Runs a periodic loop:
//! 1. Send monitor request every tick (worker discovery, health, nudges, notifications)
//! 2. Drain prune targets from monitor responses → feed to prune actor
//! 3. Trigger background sync + spawn + triage if poll interval elapsed

pub mod actors;
pub mod checks;
pub mod events;
pub mod heartbeat;

use std::sync::atomic::{AtomicBool, Ordering};
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
use actors::ActorHandle;

pub use crate::worker::events::WorkerState;
pub use actors::triage::TriageEntry;
pub use checks::{PrChecks, PrHealth};

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
    /// Long-running (`ps --watch`) rather than a single one-shot tick. Only
    /// long-running daemons record lifecycle events and a heartbeat.
    persistent: bool,
    started_at: i64,
}

impl Daemon {
    /// Start a long-running daemon: records lifecycle events in
    /// `daemon.jsonl` and keeps the heartbeat `jig daemon status` reads.
    pub fn start(cfg: Context) -> Result<Self, DaemonError> {
        Self::new(cfg, true)
    }

    /// Start a daemon for a one-shot tick (plain `jig ps`). Leaves the
    /// lifecycle log and heartbeat to any long-running daemon.
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
        Ok(Self {
            sync: ActorHandle::new(),
            monitor: ActorHandle::new(),
            prune: ActorHandle::new(),
            spawn: ActorHandle::new(),
            triage: ActorHandle::new(),
            config: cfg.config,
            registry: cfg.registry,
            last_poll,
            persistent,
            started_at: chrono::Utc::now().timestamp(),
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
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
            heartbeat::Heartbeat::clear(std::process::id());
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

        if self.persistent {
            if let Err(e) = self.heartbeat().write() {
                tracing::warn!("failed to write daemon heartbeat: {}", e);
            }
        }

        Ok(())
    }

    fn heartbeat(&self) -> heartbeat::Heartbeat {
        heartbeat::Heartbeat {
            pid: std::process::id(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            started_at: self.started_at,
            ticked_at: chrono::Utc::now().timestamp(),
            tick_interval: self.config.tick_interval,
            log: crate::context::log::session_log().map(Into::into),
            actors: vec![
                self.monitor.activity(),
                self.sync.activity(),
                self.prune.activity(),
                self.spawn.activity(),
                self.triage.activity(),
            ],
        }
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
