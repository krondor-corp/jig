//! Ps command — show status of spawned sessions

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use clap::Args;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{self, disable_raw_mode};

use comfy_table::{Attribute, Cell, CellAlignment, Color, Table};

use jig_core::config::JigToml;
use jig_core::daemon::{
    DaemonConfig, RuntimeConfig, TickResult, TimerInfo, TriageDisplayInfo, WorkerDisplayInfo,
};

use crate::op::{GlobalCtx, Op, RepoCtx};
use crate::ui;

/// Show status of spawned sessions
#[derive(Args, Debug, Clone)]
pub struct Ps {
    /// Watch mode: refresh every N seconds (default 2)
    #[arg(short, long, num_args = 0..=1, default_missing_value = "2")]
    pub watch: Option<u64>,

    /// Maximum number of concurrent auto-spawned workers
    #[arg(long)]
    max_workers: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum PsError {
    #[error("failed to list tasks: {0}")]
    ListTasks(#[from] jig_core::Error),
}

// ---------------------------------------------------------------------------
// Triage rendering
// ---------------------------------------------------------------------------
//
// Triage tables are domain presentation, so they live with the command that
// owns them rather than in `ui`, which holds only generic primitives. See
// docs/cli/ui/STDOUT-FORMATTING.md.

/// Standard triage table header cells.
fn triage_header() -> Vec<Cell> {
    vec![
        Cell::new("ISSUE").add_attribute(Attribute::Bold),
        Cell::new("MODEL").add_attribute(Attribute::Bold),
        Cell::new("ELAPSED").add_attribute(Attribute::Bold),
        Cell::new("REPO").add_attribute(Attribute::Bold),
    ]
}

/// Build a row of cells for a single triage entry.
///
/// Colors are applied unconditionally; suppression is handled by the table
/// itself (see `ui::new_domain_table`), which strips styling in plain mode and
/// when the stream is not a TTY.
fn triage_row(t: &TriageDisplayInfo) -> Vec<Cell> {
    let elapsed = ui::format_duration_short(t.elapsed_secs);
    vec![
        Cell::new(&t.issue_id).fg(Color::Cyan),
        Cell::new(&t.model).fg(Color::White),
        Cell::new(&elapsed)
            .fg(Color::White)
            .set_alignment(CellAlignment::Right),
        Cell::new(&t.repo_name).fg(Color::DarkGrey),
    ]
}

/// Render a triage table from display info.
fn render_triage_table(triages: &[TriageDisplayInfo], borders: bool, force_style: bool) -> Table {
    let mut table = ui::new_domain_table(triage_header(), borders, force_style);
    for t in triages {
        table.add_row(triage_row(t));
    }
    table
}

/// Render the full triage section with header. Empty string if no triages.
fn render_triage_section(
    triages: &[TriageDisplayInfo],
    borders: bool,
    force_style: bool,
) -> String {
    if triages.is_empty() {
        return String::new();
    }
    let table = render_triage_table(triages, borders, force_style);
    format!("{}\n{}", ui::bold("TRIAGES"), table)
}

/// Render triage section grouped by repo, with bold repo headers.
fn render_triage_section_grouped(
    triages: &[TriageDisplayInfo],
    borders: bool,
    force_style: bool,
) -> String {
    if triages.is_empty() {
        return String::new();
    }

    // Collect unique repos in order of appearance
    let mut repos: Vec<String> = Vec::new();
    for t in triages {
        if !repos.contains(&t.repo_name) {
            repos.push(t.repo_name.clone());
        }
    }

    let mut sections: Vec<String> = Vec::new();
    for repo in &repos {
        let repo_triages: Vec<&TriageDisplayInfo> =
            triages.iter().filter(|t| &t.repo_name == repo).collect();

        let mut table = ui::new_domain_table(triage_header(), borders, force_style);
        for t in &repo_triages {
            table.add_row(triage_row(t));
        }

        sections.push(format!(
            "{}\n{}",
            ui::bold(repo),
            ui::indent_lines(&table.to_string())
        ));
    }

    format!("{}\n{}", ui::bold("TRIAGES"), sections.join("\n\n"))
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// Typed result of a non-watch `jig ps` run.
#[derive(Debug, Default)]
pub struct PsOutput {
    /// Workers to display.
    pub workers: Vec<WorkerDisplayInfo>,
    /// In-flight triage subprocesses to display.
    pub triages: Vec<TriageDisplayInfo>,
    /// Whether this was a `-g/--global` run (groups tables by repo).
    pub global: bool,
}

impl PsOutput {
    /// True when there is nothing at all to show.
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty() && self.triages.is_empty()
    }
}

impl std::fmt::Display for PsOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Non-watch output is printed to stdout by the command boundary, so
        // styling is left to comfy-table's TTY detection: piped output carries
        // no escape codes.
        let force_style = false;

        let workers = if self.workers.is_empty() {
            String::new()
        } else if self.global {
            ui::render_worker_table_grouped(&self.workers, false, force_style)
        } else {
            ui::render_worker_table(&self.workers, false, force_style).to_string()
        };

        let triages = if self.global {
            render_triage_section_grouped(&self.triages, false, force_style)
        } else {
            render_triage_section(&self.triages, false, force_style)
        };

        match (workers.is_empty(), triages.is_empty()) {
            (true, true) => Ok(()),
            (false, true) => write!(f, "{workers}"),
            (true, false) => write!(f, "{triages}"),
            (false, false) => write!(f, "{workers}\n\n{triages}"),
        }
    }
}

impl Op for Ps {
    type Error = PsError;
    type Output = PsOutput;

    fn run(&self, ctx: &RepoCtx) -> Result<Self::Output, Self::Error> {
        let repo = ctx.repo()?;
        let repo_filter = repo
            .repo_root
            .file_name()
            .map(|n| n.to_string_lossy().to_string());
        let runtime_config = self.build_runtime_config(&repo.repo_root);
        self.execute_ps(repo_filter, runtime_config, false)
    }

    fn run_global(&self, _ctx: &GlobalCtx) -> Result<Self::Output, Self::Error> {
        self.execute_ps(None, RuntimeConfig::default(), true)
    }
}

impl Ps {
    fn execute_ps(
        &self,
        repo_filter: Option<String>,
        runtime_config: RuntimeConfig,
        global: bool,
    ) -> Result<PsOutput, PsError> {
        if let Some(interval) = self.watch {
            let interval = if interval == 0 { 2 } else { interval };
            run_watch(interval, runtime_config, repo_filter, global);
            return Ok(PsOutput::default());
        }

        let daemon_config = DaemonConfig {
            once: true,
            skip_sync: true,
            repo_filter,
            ..Default::default()
        };

        let mut workers = vec![];
        let mut triages = vec![];
        jig_core::daemon::run_with(&daemon_config, runtime_config, |tick, _| {
            workers.clone_from(&tick.worker_display);
            triages.clone_from(&tick.triage_display);
            false
        })?;

        let output = PsOutput {
            workers,
            triages,
            global,
        };

        // "No spawned sessions" is a status message, not data, so it stays on
        // stderr per CLAUDE.md while stdout is reserved for the tables.
        if output.is_empty() {
            eprintln!("No spawned sessions");
        }

        Ok(output)
    }

    /// Build RuntimeConfig from CLI flags + jig.toml + global config.
    fn build_runtime_config(&self, repo_root: &std::path::Path) -> RuntimeConfig {
        let jig_toml = JigToml::load(repo_root).ok().flatten().unwrap_or_default();
        let global_config = jig_core::global::GlobalConfig::load().unwrap_or_default();
        let spawn_config = &jig_toml.spawn;

        let max_concurrent_workers = self
            .max_workers
            .unwrap_or_else(|| spawn_config.resolve_max_concurrent_workers(&global_config.spawn));

        RuntimeConfig {
            max_concurrent_workers,
            auto_spawn_interval: spawn_config.resolve_auto_spawn_interval(&global_config.spawn),
            sync_interval: 60,
        }
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

/// Format structured log lines from a TickResult.
fn format_tick_log(tick: &TickResult) -> Vec<String> {
    let now = chrono::Local::now().format("%H:%M:%S");
    let mut lines = vec![];

    lines.push(format!(
        "[{}] tick: {} workers, {} actions, {} nudges, {} errors",
        now,
        tick.workers_checked,
        tick.actions_dispatched,
        tick.nudges_sent,
        tick.errors.len(),
    ));

    for (key, info) in &tick.worker_info {
        if !info.has_pr {
            continue;
        }
        if let Some(err) = &info.pr_error {
            lines.push(format!("[{}]   {} PR: {}", now, key, err));
        } else if !info.pr_checks.is_empty() {
            let problems: Vec<&str> = info
                .pr_checks
                .iter()
                .filter(|(_, bad)| *bad)
                .map(|(name, _)| name.as_str())
                .collect();
            if problems.is_empty() {
                lines.push(format!("[{}]   {} PR: ok", now, key));
            } else {
                lines.push(format!("[{}]   {} PR: {}", now, key, problems.join(", ")));
            }
        }
    }

    for spawned in &tick.auto_spawned {
        lines.push(format!("[{}]   auto-spawned: {}", now, spawned));
    }

    for pruned in &tick.pruned {
        lines.push(format!("[{}]   pruned: {}", now, pruned));
    }

    for err in &tick.errors {
        lines.push(format!("[{}]   error: {}", now, err));
    }

    lines
}

/// Run the watch loop: display + orchestrate via daemon::run_with.
fn run_watch(
    interval: u64,
    runtime_config: RuntimeConfig,
    repo_filter: Option<String>,
    global: bool,
) {
    let daemon_config = DaemonConfig {
        interval_seconds: interval,
        once: false,
        skip_sync: false,
        repo_filter,
        ..Default::default()
    };

    // Shared state for the callback
    let mut view_mode = ViewMode::Table;
    let mut log_buffer: VecDeque<String> = VecDeque::with_capacity(LOG_BUFFER_SIZE);

    // Enable raw mode for keypress detection
    terminal::enable_raw_mode().ok();

    // Clear screen once on first render
    eprint!("\x1B[2J");

    // Spawn a dedicated key-polling thread. It continuously reads crossterm events
    // and sets `quit` when q/Esc/Ctrl-C is pressed. This runs DURING ticks too,
    // so 'q' pressed mid-tick is caught immediately rather than after the tick finishes.
    // Toggle keys (l/t) are stored in a separate flag for the callback to pick up.
    let quit_for_thread = Arc::new(AtomicBool::new(false));
    let toggle_flag = Arc::new(AtomicBool::new(false));
    {
        let quit_bg = Arc::clone(&quit_for_thread);
        let toggle_bg = Arc::clone(&toggle_flag);
        std::thread::spawn(move || {
            while !quit_bg.load(Ordering::Relaxed) {
                if !event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
                    continue;
                }
                if let Ok(Event::Key(KeyEvent {
                    code, modifiers, ..
                })) = event::read()
                {
                    match code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            quit_bg.store(true, Ordering::Relaxed);
                            return;
                        }
                        KeyCode::Char('c')
                            if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            quit_bg.store(true, Ordering::Relaxed);
                            return;
                        }
                        KeyCode::Char('l') | KeyCode::Char('t') => {
                            toggle_bg.store(true, Ordering::Relaxed);
                        }
                        _ => {}
                    }
                }
            }
        });
    }

    let result = jig_core::daemon::run_with(&daemon_config, runtime_config, |tick, quit| {
        // The background thread sets quit_for_thread; propagate to the daemon's quit flag
        if quit_for_thread.load(Ordering::Relaxed) {
            quit.store(true, Ordering::Relaxed);
            return false;
        }

        // Check for toggle
        if toggle_flag.swap(false, Ordering::Relaxed) {
            view_mode.toggle();
        }

        // Append log entries
        for line in format_tick_log(tick) {
            if log_buffer.len() >= LOG_BUFFER_SIZE {
                log_buffer.pop_front();
            }
            log_buffer.push_back(line);
        }

        let render = |view: &ViewMode, tick: &TickResult, logs: &VecDeque<String>| {
            eprint!("\x1B[H");
            match view {
                ViewMode::Table => {
                    let table_output = if global {
                        ui::render_worker_table_grouped(&tick.worker_display, true, true)
                    } else {
                        ui::render_worker_table(&tick.worker_display, true, true).to_string()
                    };
                    let triage_output = if global {
                        render_triage_section_grouped(&tick.triage_display, true, true)
                    } else {
                        render_triage_section(&tick.triage_display, true, true)
                    };
                    let status_line = format_tick_status(&Some(tick));
                    let triage_count = if tick.triage_display.is_empty() {
                        String::new()
                    } else {
                        format!(", {} triages", tick.triage_display.len())
                    };
                    let spawning_section = if tick.spawning.is_empty() {
                        String::new()
                    } else {
                        let names: Vec<&str> = tick.spawning.iter().map(|s| s.as_str()).collect();
                        format!(
                            "\n\x1B[2mspawning:\x1B[0m \x1B[33m{}\x1B[0m\n",
                            names.join(", ")
                        )
                    };
                    let nudge_section = format_nudge_messages(&tick.nudge_messages);
                    let timer_section = format_timer_info(&tick.timer_info);
                    let triage_section = if triage_output.is_empty() {
                        String::new()
                    } else {
                        format!("\n{triage_output}\n")
                    };
                    let output = format!(
                        "\x1B[1mjig ps --watch\x1B[0m — {} workers{triage_count}  \x1B[2m(every {}s)\x1B[0m{status_line}\n\n{table_output}{triage_section}{spawning_section}{nudge_section}\n\x1B[2m[l]ogs  [q]uit{timer_section}\x1B[0m",
                        tick.worker_display.len(), interval,
                    );
                    for line in output.lines() {
                        eprint!("{}\x1B[K\r\n", line);
                    }
                }
                ViewMode::Logs => {
                    eprint!(
                        "\x1B[1mjig ps --watch\x1B[0m — logs  \x1B[2m(every {}s)\x1B[0m\x1B[K\r\n",
                        interval
                    );
                    eprint!("\x1B[K\r\n");
                    for line in logs {
                        eprint!("{}\x1B[K\r\n", line);
                    }
                    eprint!("\x1B[K\r\n");
                    eprint!("\x1B[2m[t]able  [q]uit\x1B[0m\x1B[K\r\n");
                }
            }
            eprint!("\x1B[J");
        };

        render(&view_mode, tick, &log_buffer);

        // Sleep interval — the background thread handles all key polling,
        // so we just need to wait and check for quit/toggle periodically.
        let sleep_end = Instant::now() + std::time::Duration::from_secs(interval);
        while Instant::now() < sleep_end {
            if quit_for_thread.load(Ordering::Relaxed) {
                quit.store(true, Ordering::Relaxed);
                return false;
            }
            if toggle_flag.swap(false, Ordering::Relaxed) {
                view_mode.toggle();
                render(&view_mode, tick, &log_buffer);
                continue;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        true // keep looping
    });

    disable_raw_mode().ok();

    match result {
        Ok(_) => {}
        Err(e) => eprintln!("daemon error: {}", e),
    }
}

/// Format the daemon tick result as a compact status suffix.
fn format_tick_status(tick: &Option<&TickResult>) -> String {
    let Some(tick) = tick else {
        return String::new();
    };
    let mut parts = vec![];
    if tick.nudges_sent > 0 {
        parts.push(format!(
            "{} nudge{}",
            tick.nudges_sent,
            if tick.nudges_sent == 1 { "" } else { "s" }
        ));
    }
    if tick.notifications_sent > 0 {
        parts.push(format!("{} notify", tick.notifications_sent));
    }
    if !tick.errors.is_empty() {
        parts.push(format!(
            "{} err{}",
            tick.errors.len(),
            if tick.errors.len() == 1 { "" } else { "s" }
        ));
    }
    if !tick.auto_spawned.is_empty() {
        parts.push(format!("{} spawned", tick.auto_spawned.len()));
    }
    if !tick.spawning.is_empty() {
        parts.push(format!("spawning {}", tick.spawning.len()));
    }
    if !tick.pruned.is_empty() {
        parts.push(format!("{} pruned", tick.pruned.len()));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("  \x1B[2m[{}]\x1B[0m", parts.join(", "))
    }
}

/// Format nudge messages delivered this tick for display below the worker table.
fn format_nudge_messages(messages: &[(String, String, String)]) -> String {
    if messages.is_empty() {
        return String::new();
    }

    let term_width = terminal::size().map(|(w, _)| w as usize).unwrap_or(120);
    let mut lines = Vec::new();
    for (worker, ntype, msg) in messages {
        let prefix = format!("  \u{21b3} {} [{}]: ", worker, ntype);
        let max_msg_len = term_width.saturating_sub(prefix.len() + 1);
        let truncated = ui::truncate(msg, max_msg_len);
        lines.push(format!(
            "\x1B[2m  \u{21b3} {} [{}]:\x1B[0m {}",
            worker, ntype, truncated
        ));
    }
    format!("\n{}\n", lines.join("\n"))
}

/// Format timer info for the footer line.
fn format_timer_info(timer: &Option<TimerInfo>) -> String {
    let Some(timer) = timer else {
        return String::new();
    };
    let mut parts = vec![format!(
        "sync: {}",
        ui::format_duration_short(timer.sync_remaining)
    )];
    if let Some(poll) = timer.poll_remaining {
        parts.push(format!("poll: {}", ui::format_duration_short(poll)));
    }
    format!("  {}", parts.join("  "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::set_plain;

    #[test]
    fn render_triage_section_empty_returns_empty() {
        let section = render_triage_section(&[], false, false);
        assert!(section.is_empty());
    }

    #[test]
    fn render_triage_section_shows_header_and_entries() {
        set_plain(true);
        let triages = vec![
            TriageDisplayInfo {
                issue_id: "JIG-77".to_string(),
                model: "sonnet".to_string(),
                elapsed_secs: 134,
                repo_name: "my-repo".to_string(),
            },
            TriageDisplayInfo {
                issue_id: "JIG-81".to_string(),
                model: "sonnet".to_string(),
                elapsed_secs: 45,
                repo_name: "my-repo".to_string(),
            },
        ];
        let section = render_triage_section(&triages, false, false);
        assert!(section.contains("TRIAGES"));
        assert!(section.contains("JIG-77"));
        assert!(section.contains("JIG-81"));
        assert!(section.contains("sonnet"));
        assert!(section.contains("2m14s"));
        assert!(section.contains("45s"));
        set_plain(false);
    }

    #[test]
    fn render_triage_table_has_correct_columns() {
        set_plain(true);
        let triages = vec![TriageDisplayInfo {
            issue_id: "JIG-99".to_string(),
            model: "haiku".to_string(),
            elapsed_secs: 3661,
            repo_name: "test-repo".to_string(),
        }];
        let table = render_triage_table(&triages, false, false).to_string();
        assert!(table.contains("ISSUE"));
        assert!(table.contains("MODEL"));
        assert!(table.contains("ELAPSED"));
        assert!(table.contains("REPO"));
        assert!(table.contains("JIG-99"));
        assert!(table.contains("haiku"));
        assert!(table.contains("1h1m"));
        assert!(table.contains("test-repo"));
        set_plain(false);
    }

    #[test]
    fn render_triage_section_grouped_empty_returns_empty() {
        let section = render_triage_section_grouped(&[], false, false);
        assert!(section.is_empty());
    }

    #[test]
    fn render_triage_section_grouped_shows_repo_headers() {
        set_plain(true);
        let triages = vec![
            TriageDisplayInfo {
                issue_id: "JIG-1".to_string(),
                model: "sonnet".to_string(),
                elapsed_secs: 10,
                repo_name: "repo-a".to_string(),
            },
            TriageDisplayInfo {
                issue_id: "JIG-2".to_string(),
                model: "sonnet".to_string(),
                elapsed_secs: 20,
                repo_name: "repo-b".to_string(),
            },
        ];
        let section = render_triage_section_grouped(&triages, false, false);
        assert!(section.contains("TRIAGES"));
        assert!(section.contains("repo-a"));
        assert!(section.contains("repo-b"));
        assert!(section.contains("JIG-1"));
        assert!(section.contains("JIG-2"));
        set_plain(false);
    }

    #[test]
    fn plain_mode_triage_output_has_no_ansi_escapes() {
        set_plain(true);
        let triages = vec![TriageDisplayInfo {
            issue_id: "JIG-7".to_string(),
            model: "sonnet".to_string(),
            elapsed_secs: 30,
            repo_name: "repo-a".to_string(),
        }];
        let section = render_triage_section(&triages, false, false);
        let grouped = render_triage_section_grouped(&triages, false, false);
        set_plain(false);
        assert!(
            !section.contains('\x1B'),
            "plain section leaked ANSI: {section:?}"
        );
        assert!(
            !grouped.contains('\x1B'),
            "plain grouped leaked ANSI: {grouped:?}"
        );
    }

    #[test]
    fn ps_output_empty_renders_nothing() {
        let out = PsOutput::default();
        assert!(out.is_empty());
        assert_eq!(out.to_string(), "");
    }

    #[test]
    fn ps_output_renders_triages_without_workers() {
        set_plain(true);
        let out = PsOutput {
            workers: vec![],
            triages: vec![TriageDisplayInfo {
                issue_id: "JIG-42".to_string(),
                model: "haiku".to_string(),
                elapsed_secs: 5,
                repo_name: "repo-a".to_string(),
            }],
            global: false,
        };
        let rendered = out.to_string();
        set_plain(false);
        assert!(!out.is_empty());
        assert!(rendered.contains("TRIAGES"));
        assert!(rendered.contains("JIG-42"));
        assert!(!rendered.contains('\x1B'));
    }
}
