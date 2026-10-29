//! Rendering functions for the ps command.

use comfy_table::{presets, Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};

use jig_core::mux::AgentState;

use crate::cli::ui;
use crate::daemon::checks::PrHealth;
use crate::daemon::TriageEntry;
use crate::worker::events::{SetupState, WorkerState};
use crate::worker::MuxStatus;
use crate::worker::WorkerStatus;

/// Maximum display width for worker names.
pub const NAME_MAX: usize = 36;

/// Build a ps table with the shared preset and styling policy.
///
/// borders: true uses UTF8_BORDERS_ONLY (watch mode), false uses NOTHING.
///
/// Styling is decided here, in one place, rather than at each call site. ps
/// writes to stderr, so comfy-table's TTY auto-detection (which inspects
/// stdout) would suppress color in normal use; styling is therefore forced on.
/// In plain mode it is disabled outright so --plain output is free of escape
/// codes for scripting, per docs/cli/ui/STDOUT-FORMATTING.md.
fn ps_table(header: Vec<Cell>, borders: bool) -> Table {
    let mut table = Table::new();
    let preset = if borders {
        presets::UTF8_BORDERS_ONLY
    } else {
        presets::NOTHING
    };
    table
        .load_preset(preset)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(header);

    if ui::is_plain() {
        table.force_no_tty();
    } else {
        table.enforce_styling();
    }

    table
}

/// Indent every line of a rendered table by two spaces (grouped repo output).
fn indent_lines(block: &str) -> String {
    block
        .lines()
        .map(|line| format!("  {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// One table of `items`, built from a header and a per-item row.
fn render_table<T>(
    items: &[T],
    header: fn() -> Vec<Cell>,
    row: fn(&T) -> Vec<Cell>,
    borders: bool,
) -> Table {
    let mut table = ps_table(header(), borders);
    for item in items {
        table.add_row(row(item));
    }
    table
}

/// One indented table per group, under a bold group header, in the order the
/// groups are first seen.
///
/// The worker and triage grouped views differ only in how the group key is
/// read, what the header says, and how a row is built — grouping, ordering,
/// indentation and the bold key line are the same for both, so they live here
/// and the `--plain` policy in [`ps_table`] applies to both by construction.
fn render_grouped<T>(
    items: &[T],
    key: fn(&T) -> String,
    header: fn() -> Vec<Cell>,
    row: fn(&T) -> Vec<Cell>,
    borders: bool,
) -> String {
    let mut keys: Vec<String> = Vec::new();
    for item in items {
        let k = key(item);
        if !keys.contains(&k) {
            keys.push(k);
        }
    }

    keys.iter()
        .map(|k| {
            let mut table = ps_table(header(), borders);
            for item in items.iter().filter(|i| key(i) == *k) {
                table.add_row(row(item));
            }
            format!("{}\n{}", ui::bold(k), indent_lines(&table.to_string()))
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Single source of truth: WorkerStatus → comfy_table color.
pub fn worker_state_color(status: &WorkerStatus) -> Color {
    match status {
        WorkerStatus::Created => Color::DarkGrey,
        WorkerStatus::Initializing => Color::Blue,
        WorkerStatus::Running => Color::Green,
        WorkerStatus::Spawned => Color::Blue,
        WorkerStatus::Idle => Color::Yellow,
        WorkerStatus::WaitingInput => Color::Magenta,
        WorkerStatus::Stalled => Color::Red,
        WorkerStatus::WaitingReview => Color::Cyan,
        WorkerStatus::Approved => Color::Green,
        WorkerStatus::Merged => Color::Green,
        WorkerStatus::Closed => Color::DarkGrey,
        WorkerStatus::Failed => Color::Red,
        WorkerStatus::Archived => Color::DarkGrey,
    }
}

/// Single source of truth: WorkerStatus → display label.
pub fn worker_state_str(status: &WorkerStatus) -> &'static str {
    match status {
        WorkerStatus::Created => "created",
        WorkerStatus::Initializing => "initializing",
        WorkerStatus::Running => "running",
        WorkerStatus::Spawned => "spawned",
        WorkerStatus::Idle => "idle",
        WorkerStatus::WaitingInput => "waiting",
        WorkerStatus::Stalled => "stalled",
        WorkerStatus::WaitingReview => "review",
        WorkerStatus::Approved => "approved",
        WorkerStatus::Merged => "merged",
        WorkerStatus::Closed => "closed",
        WorkerStatus::Failed => "failed",
        WorkerStatus::Archived => "archived",
    }
}

/// The STATE column: setup, while it is running or once it has failed —
/// `on_create 42s`, `failed: on_create (exit 1)` — and the worker status
/// otherwise.
///
/// The tail and log path don't fit a column; `jig attach` quotes them.
pub fn format_state(w: &WorkerState, now: i64) -> (String, Color) {
    match &w.setup {
        SetupState::Running { phase, since, .. } => {
            let elapsed = (now - since).max(0) as u64;
            (
                format!("{phase} {}", ui::format_duration_short(elapsed)),
                Color::Blue,
            )
        }
        SetupState::Failed { phase, exit, .. } => (format!("failed: {phase} ({exit})"), Color::Red),
        SetupState::Pending | SetupState::Ok { .. } => {
            if w.status == WorkerStatus::WaitingReview && w.is_draft {
                ("draft".to_string(), Color::Blue)
            } else {
                (
                    worker_state_str(&w.status).to_string(),
                    worker_state_color(&w.status),
                )
            }
        }
    }
}

/// Format PR health status for display.
///
/// A failed check wins over `has_pr`: when `gh` is broken, PR discovery is
/// the first thing to fail, so there is no PR to report and the failure
/// would otherwise render as an unexplained `-`. The reason stays out of the
/// column — it would widen the table — and goes to `jig daemon logs`.
pub fn format_health(info: &PrHealth) -> (String, Color) {
    if info.pr_error.is_some() {
        return ("? gh".to_string(), Color::Yellow);
    }

    if !info.has_pr {
        return ("-".to_string(), Color::DarkGrey);
    }

    if info.pr_checks.is_empty() {
        return ("-".to_string(), Color::DarkGrey);
    }

    let problems = info.pr_checks.problems();
    if problems.is_empty() {
        ("ok".to_string(), Color::Green)
    } else {
        (problems.join(" "), Color::Red)
    }
}

/// The worker table's column headers, shared by `ps` and the web view.
pub const WORKER_HEADERS: [&str; 7] = [
    "WORKER", "STATE", "NUDGE", "COMMITS", "PR", "HEALTH", "ISSUE",
];

/// One cell of a worker row, before it becomes a terminal cell or HTML.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub text: String,
    pub color: Color,
    /// Right-aligned (the numeric columns).
    pub right: bool,
}

impl Column {
    fn new(text: impl Into<String>, color: Color) -> Self {
        Self {
            text: text.into(),
            color,
            right: false,
        }
    }

    fn right(mut self) -> Self {
        self.right = true;
        self
    }
}

/// A worker's row as text and color, one [`Column`] per [`WORKER_HEADERS`].
///
/// NOTE(canonical): `ps` and the daemon's web view both render from this, so
/// the two can't drift — change a column here, not in either renderer.
pub fn worker_columns(w: &WorkerState, now: i64) -> Vec<Column> {
    let mux_indicator = match w.mux_status {
        MuxStatus::Running => "●",
        MuxStatus::Exited => "○",
        MuxStatus::NotFound => "✗",
    };
    // When the backend recognizes the agent (herdr), its live read wins:
    // blocked (needs a human) and working are states the event log can't see.
    let mux_color = match (w.mux_status, w.mux_agent_state) {
        (MuxStatus::Running, Some(AgentState::Blocked)) => Color::Red,
        (MuxStatus::Running, Some(AgentState::Working)) => Color::Green,
        (MuxStatus::Running, Some(AgentState::Idle))
        | (MuxStatus::Running, Some(AgentState::Done)) => Color::Yellow,
        (MuxStatus::Running, _) => Color::Green,
        (MuxStatus::Exited, _) => Color::Yellow,
        (MuxStatus::NotFound, _) => Color::DarkGrey,
    };

    let (state_text, state_color) = format_state(w, now);

    let nudge_count = w.nudge_count();
    let (nudge_text, nudge_color) = if nudge_count == 0 {
        if let Some(cd) = w.nudge_cooldown_remaining {
            (
                format!("({})", ui::format_duration_short(cd)),
                Color::DarkGrey,
            )
        } else {
            ("-".to_string(), Color::DarkGrey)
        }
    } else if let Some(cd) = w.nudge_cooldown_remaining {
        (
            format!("{} ({})", nudge_count, ui::format_duration_short(cd)),
            Color::Yellow,
        )
    } else {
        (nudge_count.to_string(), Color::Yellow)
    };

    let dirty_marker = if w.is_dirty { "*" } else { "" };
    let commits = if w.commits_ahead > 0 || w.is_dirty {
        format!("{}{}", w.commits_ahead, dirty_marker)
    } else {
        "-".to_string()
    };
    let commit_color = if w.is_dirty {
        Color::Yellow
    } else if w.commits_ahead > 0 {
        Color::White
    } else {
        Color::DarkGrey
    };

    let pr = w
        .pr_url
        .as_ref()
        .map(|url| {
            url.path_segments()
                .and_then(|mut s| s.next_back())
                .map(|n| format!("#{}", n))
                .unwrap_or_else(|| "yes".to_string())
        })
        .unwrap_or_else(|| "-".to_string());
    let pr_color = if pr == "-" {
        Color::DarkGrey
    } else {
        Color::Cyan
    };

    let issue = w
        .issue_ref
        .as_deref()
        .map(|id| ui::truncate(id.rsplit('/').next().unwrap_or(id), 16))
        .unwrap_or_else(|| "-".to_string());
    let issue_color = if issue == "-" {
        Color::DarkGrey
    } else {
        Color::White
    };

    let (health_text, health_color) = format_health(&w.pr_health);

    let name = format!("{} {}", mux_indicator, ui::truncate(&w.name, NAME_MAX));

    vec![
        Column::new(name, mux_color),
        Column::new(state_text, state_color),
        Column::new(nudge_text, nudge_color).right(),
        Column::new(commits, commit_color).right(),
        Column::new(pr, pr_color),
        Column::new(health_text, health_color),
        Column::new(issue, issue_color),
    ]
}

/// Build a row of cells for a single worker.
fn worker_row(w: &WorkerState) -> Vec<Cell> {
    worker_columns(w, chrono::Utc::now().timestamp())
        .into_iter()
        .map(|c| {
            let cell = Cell::new(&c.text).fg(c.color);
            if c.right {
                cell.set_alignment(CellAlignment::Right)
            } else {
                cell
            }
        })
        .collect()
}

/// Standard table header cells.
fn table_header() -> Vec<Cell> {
    WORKER_HEADERS
        .iter()
        .map(|h| Cell::new(h).add_attribute(Attribute::Bold))
        .collect()
}

/// Render a worker table from display info.
///
/// `borders`: true uses UTF8_BORDERS_ONLY (watch mode), false uses NOTHING (non-watch).
pub fn render_worker_table(workers: &[WorkerState], borders: bool) -> Table {
    render_table(workers, table_header, worker_row, borders)
}

/// Render workers grouped by repo, with bold repo headers.
///
/// Returns a formatted string with separate tables per repo.
pub fn render_worker_table_grouped(workers: &[WorkerState], borders: bool) -> String {
    render_grouped(
        workers,
        WorkerState::repo_name,
        table_header,
        worker_row,
        borders,
    )
}

// ---------------------------------------------------------------------------
// Triage table
// ---------------------------------------------------------------------------

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
fn triage_row(t: &TriageEntry) -> Vec<Cell> {
    let now = chrono::Utc::now().timestamp();
    let elapsed = (now - t.spawned_at).max(0) as u64;
    vec![
        Cell::new(&t.issue_id).fg(Color::Cyan),
        Cell::new(&t.model).fg(Color::White),
        Cell::new(ui::format_duration_short(elapsed))
            .fg(Color::White)
            .set_alignment(CellAlignment::Right),
        Cell::new(&t.repo_name).fg(Color::DarkGrey),
    ]
}

/// Render a triage table from display info.
///
/// `borders`: true uses UTF8_BORDERS_ONLY (watch mode), false uses NOTHING.
pub fn render_triage_table(triages: &[TriageEntry], borders: bool) -> Table {
    render_table(triages, triage_header, triage_row, borders)
}

/// Render the full triage section with header. Returns empty string if no triages.
pub fn render_triage_section(triages: &[TriageEntry], borders: bool) -> String {
    if triages.is_empty() {
        return String::new();
    }
    let table = render_triage_table(triages, borders);
    format!("{}\n{}", ui::bold("TRIAGES"), table)
}

/// Render triage section grouped by repo, with bold repo headers.
pub fn render_triage_section_grouped(triages: &[TriageEntry], borders: bool) -> String {
    if triages.is_empty() {
        return String::new();
    }

    let sections = render_grouped(
        triages,
        |t| t.repo_name.clone(),
        triage_header,
        triage_row,
        borders,
    );
    format!("{}\n{}", ui::bold("TRIAGES"), sections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ui;
    use crate::worker::events::{SetupExit, SetupPhase};

    /// Serializes the tests that flip the global plain-mode flag and restores
    /// it on drop, so parallel tests cannot observe each other's setting and a
    /// panicking test cannot leak it.
    struct PlainGuard(#[allow(dead_code)] std::sync::MutexGuard<'static, ()>);

    impl PlainGuard {
        fn new(plain: bool) -> Self {
            static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
            let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
            ui::set_plain(plain);
            Self(guard)
        }
    }

    impl Drop for PlainGuard {
        fn drop(&mut self) {
            ui::set_plain(false);
        }
    }

    fn triage_entry(issue_id: &str, model: &str, ago_secs: i64, repo: &str) -> TriageEntry {
        TriageEntry {
            issue_id: issue_id.to_string(),
            model: model.to_string(),
            spawned_at: chrono::Utc::now().timestamp() - ago_secs,
            repo_name: repo.to_string(),
        }
    }

    #[test]
    fn health_flags_a_failed_check_even_without_a_pr() {
        // The `gh`-is-broken case: PR discovery failed, so there is no PR to
        // report — a bare "-" would read as "nothing wrong here".
        let health = PrHealth {
            pr_error: Some("gh CLI failed: not logged into github.com".into()),
            has_pr: false,
            ..Default::default()
        };
        let (text, color) = format_health(&health);
        assert_eq!(text, "? gh");
        assert_eq!(color, Color::Yellow);
    }

    #[test]
    fn health_error_does_not_widen_the_column() {
        let health = PrHealth {
            pr_error: Some(format!("{}\nsecond line", "x".repeat(200))),
            has_pr: true,
            ..Default::default()
        };
        let (text, _) = format_health(&health);
        assert_eq!(text, "? gh");
    }

    #[test]
    fn health_without_pr_or_error_stays_blank() {
        let (text, color) = format_health(&PrHealth::default());
        assert_eq!(text, "-");
        assert_eq!(color, Color::DarkGrey);
    }

    #[test]
    fn health_reports_ok_when_checks_pass() {
        let health = PrHealth {
            has_pr: true,
            pr_checks: crate::daemon::PrChecks {
                ci: Some(false),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(format_health(&health).0, "ok");
    }

    #[test]
    fn a_running_setup_shows_its_phase_and_duration() {
        let w = WorkerState {
            status: WorkerStatus::Initializing,
            setup: SetupState::Running {
                phase: SetupPhase::OnCreate,
                since: 1000,
                elapsed_ms: 0,
                log_path: None,
            },
            ..Default::default()
        };
        let (text, color) = format_state(&w, 1042);
        assert_eq!(text, format!("on_create {}", ui::format_duration_short(42)));
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn a_failed_setup_shows_its_phase_and_exit() {
        let w = WorkerState {
            status: WorkerStatus::Failed,
            setup: SetupState::Failed {
                phase: SetupPhase::OnCreate,
                exit: SetupExit::Code(1),
                tail: vec!["boom".into()],
                log_path: None,
            },
            ..Default::default()
        };
        let (text, color) = format_state(&w, 0);
        assert_eq!(text, "failed: on_create (exit 1)");
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn a_finished_setup_shows_the_worker_status() {
        let w = WorkerState {
            status: WorkerStatus::Running,
            setup: SetupState::Ok { duration_ms: 3000 },
            ..Default::default()
        };
        assert_eq!(format_state(&w, 0).0, "running");
    }

    #[test]
    fn render_triage_section_empty_returns_empty() {
        let section = render_triage_section(&[], false);
        assert!(section.is_empty());
    }

    #[test]
    fn render_triage_section_shows_header_and_entries() {
        let _plain = PlainGuard::new(true);
        let triages = vec![
            triage_entry("JIG-77", "triage-77", 134, "my-repo"),
            triage_entry("JIG-81", "triage-81", 45, "my-repo"),
        ];
        let section = render_triage_section(&triages, false);
        assert!(section.contains("TRIAGES"));
        assert!(section.contains("JIG-77"));
        assert!(section.contains("JIG-81"));
        assert!(section.contains("triage-77"));
    }

    #[test]
    fn render_triage_table_has_correct_columns() {
        let _plain = PlainGuard::new(true);
        let triages = vec![triage_entry("JIG-99", "triage-99", 3661, "test-repo")];
        let table = render_triage_table(&triages, false).to_string();
        assert!(table.contains("ISSUE"));
        assert!(table.contains("MODEL"));
        assert!(table.contains("ELAPSED"));
        assert!(table.contains("REPO"));
        assert!(table.contains("JIG-99"));
        assert!(table.contains("triage-99"));
        assert!(table.contains("test-repo"));
    }

    #[test]
    fn render_triage_section_grouped_empty_returns_empty() {
        let section = render_triage_section_grouped(&[], false);
        assert!(section.is_empty());
    }

    #[test]
    fn render_triage_section_grouped_shows_repo_headers() {
        let _plain = PlainGuard::new(true);
        let triages = vec![
            triage_entry("JIG-1", "triage-1", 10, "repo-a"),
            triage_entry("JIG-2", "triage-2", 20, "repo-b"),
        ];
        let section = render_triage_section_grouped(&triages, false);
        assert!(section.contains("TRIAGES"));
        assert!(section.contains("repo-a"));
        assert!(section.contains("repo-b"));
        assert!(section.contains("JIG-1"));
        assert!(section.contains("JIG-2"));
    }

    #[test]
    fn plain_mode_ps_tables_have_no_ansi_escapes() {
        let _plain = PlainGuard::new(true);
        let triages = vec![triage_entry("KRO-1", "w1", 30, "repo-a")];
        let section = render_triage_section(&triages, false);
        let grouped = render_triage_section_grouped(&triages, false);
        let table = render_triage_table(&triages, false).to_string();
        assert!(
            !section.contains(''),
            "plain triage section leaked ANSI: {section:?}"
        );
        assert!(
            !grouped.contains(''),
            "plain grouped triage section leaked ANSI: {grouped:?}"
        );
        assert!(
            !table.contains(''),
            "plain triage table leaked ANSI: {table:?}"
        );
    }
    #[test]
    fn normal_mode_ps_tables_are_still_styled() {
        // ps writes to stderr, so comfy-table's stdout TTY check would strip
        // color under test; styling must be forced on when not in plain mode.
        let _plain = PlainGuard::new(false);
        let triages = vec![triage_entry("KRO-1", "w1", 30, "repo-a")];
        let section = render_triage_section(&triages, false);
        assert!(
            section.contains('\x1B'),
            "normal-mode output lost its styling: {section:?}"
        );
    }
}
