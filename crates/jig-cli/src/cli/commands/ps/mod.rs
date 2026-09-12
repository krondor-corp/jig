//! Ps command — show status of spawned sessions.
//!
//! `ps` prefers a running daemon: it asks over the socket and renders the
//! answer, so the display costs one round trip instead of a tick loop of its
//! own. With no daemon listening it falls back to driving a daemon
//! in-process, which is what every `jig ps` used to do.

mod render;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use clap::Args;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{self, disable_raw_mode};

use crate::context::{Context, ScopedCtx};
use crate::daemon::ipc::{self, DaemonStatus};
use crate::daemon::pidfile::PidFile;
use crate::daemon::{Daemon, TriageEntry, WorkerState};

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;

/// Show status of spawned sessions
#[derive(Args, Debug, Clone)]
pub struct Ps {
    /// Watch mode: refresh every N seconds (default 2)
    #[arg(short, long, num_args = 0..=1, default_missing_value = "2")]
    pub watch: Option<u64>,

    /// Maximum number of concurrent auto-spawned workers
    #[arg(long)]
    max_workers: Option<usize>,

    /// Operate on all tracked repos
    #[arg(short = 'g', long)]
    global: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum PsError {
    #[error("failed to list tasks: {0}")]
    Context(#[from] crate::context::ContextError),
    #[error(transparent)]
    Daemon(#[from] crate::daemon::DaemonError),
}

impl Op for Ps {
    type Context = ScopedCtx;
    type Error = PsError;
    type Output = NoOutput;

    fn build_context(&self) -> Result<ScopedCtx, PsError> {
        Ok(ScopedCtx::from_global(self.global)?)
    }

    fn run(&self, ctx: ScopedCtx) -> Result<Self::Output, Self::Error> {
        let global = self.global;
        let cfg: Context = match ctx {
            ScopedCtx::Repo(r) => {
                let max_workers = self
                    .max_workers
                    .unwrap_or(r.jig_toml.spawn.max_concurrent_workers);
                let mut c = Context::from(r);
                c.config.max_concurrent_workers = max_workers;
                c
            }
            ScopedCtx::Global(g) => Context::from(g),
        };
        self.execute_ps(cfg, global)
    }
}

/// One rendered view's worth of daemon state, whatever its source.
#[derive(Default)]
struct Frame {
    workers: Vec<WorkerState>,
    triages: Vec<TriageEntry>,
    spawning: Vec<String>,
    poll_remaining: u64,
}

impl Frame {
    /// Take a global daemon's frame down to one repo.
    ///
    /// The daemon always watches every tracked repo, so a repo-scoped `ps`
    /// narrows the answer rather than asking a narrower question.
    fn from_status(status: DaemonStatus, scope: Option<&Path>) -> Self {
        let repo_name = scope
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy());
        Self {
            workers: status
                .workers
                .iter()
                .filter(|w| scope.is_none_or(|root| w.in_repo(root)))
                .map(|w| w.to_display_state())
                .collect(),
            triages: status
                .triages
                .into_iter()
                .filter(|t| repo_name.as_ref().is_none_or(|name| t.repo_name == **name))
                .collect(),
            spawning: status.spawning,
            poll_remaining: status.poll_remaining,
        }
    }

    fn from_daemon(daemon: &Daemon) -> Self {
        Self {
            workers: daemon.monitor.actor().workers(),
            triages: daemon.triage.actor().active_entries(),
            spawning: daemon.spawn.actor().spawning_workers(),
            poll_remaining: daemon.poll_remaining_secs(),
        }
    }

    fn is_empty(&self) -> bool {
        self.workers.is_empty() && self.triages.is_empty()
    }
}

impl Ps {
    fn execute_ps(&self, mut cfg: Context, global: bool) -> Result<NoOutput, PsError> {
        // A repo-scoped `ps` filters a global daemon's answer down to the
        // repo it was run in; `-g` takes the frame whole.
        let scope = if global {
            None
        } else {
            cfg.repos.first().map(|r| r.repo_root.clone())
        };

        if let Some(interval) = self.watch {
            cfg.config.tick_interval = if interval == 0 { 2 } else { interval };
            run_watch(cfg, global, scope);
            return Ok(NoOutput);
        }

        let frame = match daemon_frame(scope.as_deref()) {
            Some(frame) => frame,
            None => oneshot_frame(cfg)?,
        };
        print_frame(&frame, global);
        Ok(NoOutput)
    }
}

/// The running daemon's frame, or `None` if it cannot give us one.
///
/// A daemon that answers badly — one left from an older jig, say — is worth
/// a log line but not a failed `ps`: the in-process fallback below still
/// produces the right answer.
fn daemon_frame(scope: Option<&Path>) -> Option<Frame> {
    match ipc::status() {
        Ok(Some(status)) => Some(Frame::from_status(status, scope)),
        Ok(None) => None,
        Err(e) => {
            tracing::warn!("daemon did not answer, falling back in-process: {}", e);
            None
        }
    }
}

/// Drive a daemon for exactly one tick — what `ps` does with no daemon up.
fn oneshot_frame(cfg: Context) -> Result<Frame, PsError> {
    let quit = AtomicBool::new(false);
    let mut frame = Frame::default();
    let mut daemon = Daemon::oneshot(cfg)?;
    daemon.run(&quit, |daemon| {
        if !daemon.wait_for_monitor(Daemon::MONITOR_WAIT) {
            // Say so rather than printing a table that silently omits
            // whatever the pass had not reached yet.
            eprintln!(
                "warning: monitor pass still running after {}s — this table may be incomplete",
                Daemon::MONITOR_WAIT.as_secs()
            );
        }
        frame = Frame::from_daemon(daemon);
        false
    });
    Ok(frame)
}

fn print_frame(frame: &Frame, global: bool) {
    if frame.is_empty() {
        eprintln!("No spawned sessions");
        return;
    }

    let (table, triage_section) = if global {
        (
            render::render_worker_table_grouped(&frame.workers, false),
            render::render_triage_section_grouped(&frame.triages, false),
        )
    } else {
        (
            render::render_worker_table(&frame.workers, false).to_string(),
            render::render_triage_section(&frame.triages, false),
        )
    };

    if !frame.workers.is_empty() {
        eprintln!("{table}");
    }
    if !triage_section.is_empty() {
        if !frame.workers.is_empty() {
            eprintln!();
        }
        eprintln!("{triage_section}");
    }
}

/// View mode for the watch display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Table,
    Logs,
}

impl ViewMode {
    fn toggle(&mut self) {
        *self = match self {
            ViewMode::Table => ViewMode::Logs,
            ViewMode::Logs => ViewMode::Table,
        };
    }
}

const LOG_BUFFER_SIZE: usize = 50;

/// Where a watch view's frames come from, for the status line.
#[derive(Debug, Clone, Copy)]
enum Source {
    /// Reading a daemon someone else started.
    Daemon(u32),
    /// No daemon was running, so this process is the daemon.
    Hosted,
    /// No daemon, and this view is repo-scoped so it will not become one.
    Local,
}

impl Source {
    fn label(&self) -> String {
        match self {
            Source::Daemon(pid) => format!("daemon pid {pid}"),
            Source::Hosted => "hosting the daemon".into(),
            Source::Local => "no daemon — jig daemon start".into(),
        }
    }
}

/// The watch TUI: terminal setup, key handling, and rendering. Frames come
/// from either a running daemon or this process, and it renders both the
/// same way.
struct Watch {
    view_mode: ViewMode,
    log_buffer: VecDeque<String>,
    log_tailer: Option<crate::context::log::LogTailer>,
    interval: u64,
    global: bool,
    quit: Arc<AtomicBool>,
    toggle: Arc<AtomicBool>,
}

impl Watch {
    fn new(interval: u64, global: bool) -> Self {
        let quit = Arc::new(AtomicBool::new(false));
        let toggle = Arc::new(AtomicBool::new(false));
        spawn_key_reader(Arc::clone(&quit), Arc::clone(&toggle));

        terminal::enable_raw_mode().ok();
        eprint!("\x1B[2J");

        Self {
            view_mode: ViewMode::Table,
            log_buffer: VecDeque::with_capacity(LOG_BUFFER_SIZE),
            log_tailer: None,
            interval,
            global,
            quit,
            toggle,
        }
    }

    /// Follow `path` in the logs view. Switching targets (the daemon
    /// restarted, or we took over hosting) starts a fresh tail.
    fn follow_log(&mut self, path: Option<PathBuf>) {
        let Some(path) = path else { return };
        if self.log_tailer.as_ref().is_some_and(|t| t.path() == path) {
            return;
        }
        self.log_tailer = Some(crate::context::log::LogTailer::from_end(path));
    }

    fn quitting(&self) -> bool {
        self.quit.load(Ordering::Relaxed)
    }

    fn drain_logs(&mut self) {
        let new_lines = self
            .log_tailer
            .as_mut()
            .map(|t| t.poll(LOG_BUFFER_SIZE))
            .unwrap_or_default();
        for line in new_lines {
            if self.log_buffer.len() >= LOG_BUFFER_SIZE {
                self.log_buffer.pop_front();
            }
            self.log_buffer.push_back(line);
        }
    }

    fn render(&self, frame: &Frame, source: Source) {
        eprint!("\x1B[H");
        match self.view_mode {
            ViewMode::Table => {
                let table_output = if self.global {
                    render::render_worker_table_grouped(&frame.workers, true)
                } else {
                    render::render_worker_table(&frame.workers, true).to_string()
                };
                let triage_output = if self.global {
                    render::render_triage_section_grouped(&frame.triages, true)
                } else {
                    render::render_triage_section(&frame.triages, true)
                };
                let triage_count = if frame.triages.is_empty() {
                    String::new()
                } else {
                    format!(", {} triages", frame.triages.len())
                };
                let spawning_section = if frame.spawning.is_empty() {
                    String::new()
                } else {
                    let names: Vec<&str> = frame.spawning.iter().map(|s| s.as_str()).collect();
                    format!(
                        "\n\x1B[2mspawning:\x1B[0m \x1B[33m{}\x1B[0m\n",
                        names.join(", ")
                    )
                };
                let timer_section = format!(
                    "  poll: {}",
                    ui::format_duration_short(frame.poll_remaining)
                );
                let triage_section = if triage_output.is_empty() {
                    String::new()
                } else {
                    format!("\n{triage_output}\n")
                };
                let output = format!(
                    "\x1B[1mjig ps --watch\x1B[0m — {} workers{triage_count}  \x1B[2m(every {}s · {})\x1B[0m\n\n{table_output}{triage_section}{spawning_section}\n\x1B[2m[l]ogs  [q]uit{timer_section}\x1B[0m",
                    frame.workers.len(),
                    self.interval,
                    source.label(),
                );
                for line in output.lines() {
                    eprint!("{}\x1B[K\r\n", line);
                }
            }
            ViewMode::Logs => {
                eprint!(
                    "\x1B[1mjig ps --watch\x1B[0m — logs  \x1B[2m(every {}s · {})\x1B[0m\x1B[K\r\n",
                    self.interval,
                    source.label(),
                );
                eprint!("\x1B[K\r\n");
                for line in &self.log_buffer {
                    eprint!("{}\x1B[K\r\n", line);
                }
                eprint!("\x1B[K\r\n");
                eprint!("\x1B[2m[t]able  [q]uit\x1B[0m\x1B[K\r\n");
            }
        }
        eprint!("\x1B[J");
    }

    /// Wait out the refresh interval, repainting on a view toggle. Returns
    /// false when the user asked to quit.
    fn wait(&mut self, frame: &Frame, source: Source) -> bool {
        let sleep_end = Instant::now() + std::time::Duration::from_secs(self.interval);
        while Instant::now() < sleep_end {
            if self.quitting() {
                return false;
            }
            if self.toggle.swap(false, Ordering::Relaxed) {
                self.view_mode.toggle();
                self.render(frame, source);
                continue;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        true
    }

    /// One display pass: apply a pending toggle, tail logs, paint, sleep.
    fn step(&mut self, frame: &Frame, source: Source) -> bool {
        if self.toggle.swap(false, Ordering::Relaxed) {
            self.view_mode.toggle();
        }
        self.drain_logs();
        self.render(frame, source);
        self.wait(frame, source)
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        // Stop the key reader too, or it keeps swallowing keystrokes from
        // whatever runs after the view closes.
        self.quit.store(true, Ordering::Relaxed);
        disable_raw_mode().ok();
    }
}

fn spawn_key_reader(quit: Arc<AtomicBool>, toggle: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        while !quit.load(Ordering::Relaxed) {
            if !event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
                continue;
            }
            if let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = event::read()
            {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        quit.store(true, Ordering::Relaxed);
                        return;
                    }
                    KeyCode::Char('c')
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        quit.store(true, Ordering::Relaxed);
                        return;
                    }
                    KeyCode::Char('l') | KeyCode::Char('t') => {
                        toggle.store(true, Ordering::Relaxed);
                    }
                    _ => {}
                }
            }
        }
    });
}

/// Run the watch loop against a running daemon, or against one this process
/// hosts when none is up.
fn run_watch(cfg: Context, global: bool, scope: Option<PathBuf>) {
    if ipc::is_running() {
        run_watch_ipc(cfg.config.tick_interval, global, scope);
    } else {
        run_watch_local(cfg, global);
    }
}

/// Poll the daemon over IPC and render. No tick loop, no actors — several of
/// these can run at once without fighting over the same worktrees.
fn run_watch_ipc(interval: u64, global: bool, scope: Option<PathBuf>) {
    let mut watch = Watch::new(interval, global);
    loop {
        let status = match ipc::status() {
            Ok(Some(status)) => status,
            // The daemon went away mid-watch. Say so rather than silently
            // rendering the last frame forever.
            Ok(None) => {
                drop(watch);
                eprintln!("daemon stopped — run `jig daemon start` to restart it");
                return;
            }
            Err(e) => {
                drop(watch);
                eprintln!("daemon error: {e}");
                return;
            }
        };
        let source = Source::Daemon(status.info.pid);
        watch.follow_log(status.info.log.clone());
        let frame = Frame::from_status(status, scope.as_deref());
        if !watch.step(&frame, source) {
            return;
        }
    }
}

/// No daemon is running, so become one for the life of this view.
///
/// A global watch takes the socket and PID file too, so `jig daemon status`
/// and other `jig ps` invocations see it. A repo-scoped watch does not: the
/// daemon contract is "watches every tracked repo", and answering for one
/// repo under that name would be a lie.
fn run_watch_local(cfg: Context, global: bool) {
    let interval = cfg.config.tick_interval;

    let mut daemon = match Daemon::start(cfg) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("daemon error: {}", e);
            return;
        }
    };

    let mut watch = Watch::new(interval, global);
    watch.follow_log(crate::context::log::session_log().map(Into::into));

    // Held for the life of the view; dropping them releases the socket and
    // the single-daemon claim.
    let hosted = global.then(host_daemon).flatten();
    let source = match &hosted {
        Some(_) => Source::Hosted,
        None => Source::Local,
    };
    let quit = Arc::clone(&watch.quit);
    let listener = hosted.map(|(server, pid_file)| {
        let handle = server.spawn(daemon.shared(), Arc::clone(&quit));
        (handle, pid_file)
    });

    daemon.run(&quit, |daemon| {
        let frame = Frame::from_daemon(daemon);
        watch.step(&frame, source)
    });

    if let Some((handle, pid_file)) = listener {
        quit.store(true, Ordering::Relaxed);
        let _ = handle.join();
        drop(pid_file);
    }
}

/// Claim the daemon slot and socket for a hosting watch view. `None` when
/// another daemon won the race — the view still works, it just does not
/// answer for anyone else.
fn host_daemon() -> Option<(ipc::Server, PidFile)> {
    let pid_file = PidFile::acquire()
        .inspect_err(|e| tracing::info!("not hosting the daemon socket: {}", e))
        .ok()?;
    let server = ipc::Server::bind()
        .inspect_err(|e| tracing::info!("not hosting the daemon socket: {}", e))
        .ok()?;
    Some((server, pid_file))
}
