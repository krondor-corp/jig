//! Rendering functions for the ps command.

use comfy_table::{presets, Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};

use crate::cli::ui;
use crate::daemon::checks::PrHealth;
use crate::daemon::TriageEntry;
use crate::worker::events::WorkerState;
use crate::worker::MuxStatus;
use crate::worker::WorkerStatus;

/// Maximum display width for worker names.
pub const NAME_MAX: usize = 36;

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
        WorkerStatus::Failed => "failed",
        WorkerStatus::Archived => "archived",
    }
}

/// Format PR health status for display.
pub fn format_health(info: &PrHealth) -> (String, Color) {
    if !info.has_pr {
        return ("-".to_string(), Color::DarkGrey);
    }

    if let Some(err) = &info.pr_error {
        tracing::debug!(error = %err, "PR health error");
        return ("?".to_string(), Color::Yellow);
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

/// Build a row of cells for a single worker.
fn worker_row(w: &WorkerState) -> Vec<Cell> {
    let mux_indicator = match w.mux_status {
        MuxStatus::Running => "●",
        MuxStatus::Exited => "○",
        MuxStatus::NotFound => "✗",
    };
    // When the backend recognizes the agent (herdr), its live read wins:
    // blocked (needs a human) and working are states the event log can't see.
    let mux_color = match (w.mux_status, w.mux_agent_state) {
        (MuxStatus::Running, Some(jig_core::mux::AgentState::Blocked)) => Color::Red,
        (MuxStatus::Running, Some(jig_core::mux::AgentState::Working)) => Color::Green,
        (MuxStatus::Running, Some(jig_core::mux::AgentState::Idle))
        | (MuxStatus::Running, Some(jig_core::mux::AgentState::Done)) => Color::Yellow,
        (MuxStatus::Running, _) => Color::Green,
        (MuxStatus::Exited, _) => Color::Yellow,
        (MuxStatus::NotFound, _) => Color::DarkGrey,
    };

    let (state_text, state_color) = if w.status == WorkerStatus::WaitingReview && w.is_draft {
        ("draft", Color::Blue)
    } else {
        (worker_state_str(&w.status), worker_state_color(&w.status))
    };

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
        .parsed_pr_url
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
        Cell::new(&name).fg(mux_color),
        Cell::new(state_text).fg(state_color),
        Cell::new(&nudge_text)
            .fg(nudge_color)
            .set_alignment(CellAlignment::Right),
        Cell::new(&commits)
            .fg(commit_color)
            .set_alignment(CellAlignment::Right),
        Cell::new(&pr).fg(pr_color),
        Cell::new(&health_text).fg(health_color),
        Cell::new(&issue).fg(issue_color),
    ]
}

/// Standard table header cells.
fn table_header() -> Vec<Cell> {
    vec![
        Cell::new("WORKER").add_attribute(Attribute::Bold),
        Cell::new("STATE").add_attribute(Attribute::Bold),
        Cell::new("NUDGE").add_attribute(Attribute::Bold),
        Cell::new("COMMITS").add_attribute(Attribute::Bold),
        Cell::new("PR").add_attribute(Attribute::Bold),
        Cell::new("HEALTH").add_attribute(Attribute::Bold),
        Cell::new("ISSUE").add_attribute(Attribute::Bold),
    ]
}

/// Render a worker table from display info.
///
/// `borders`: true uses UTF8_BORDERS_ONLY (watch mode), false uses NOTHING (non-watch).
pub fn render_worker_table(workers: &[WorkerState], borders: bool) -> Table {
    let mut table = Table::new();
    let preset = if borders {
        presets::UTF8_BORDERS_ONLY
    } else {
        presets::NOTHING
    };
    table
        .load_preset(preset)
        .enforce_styling()
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(table_header());

    for w in workers {
        table.add_row(worker_row(w));
    }

    table
}

/// Render workers grouped by repo, with bold repo headers.
///
/// Returns a formatted string with separate tables per repo.
pub fn render_worker_table_grouped(workers: &[WorkerState], borders: bool) -> String {
    let mut repos: Vec<String> = Vec::new();
    for w in workers {
        let name = w.repo_name();
        if !repos.contains(&name) {
            repos.push(name);
        }
    }

    let preset = if borders {
        presets::UTF8_BORDERS_ONLY
    } else {
        presets::NOTHING
    };

    let mut sections: Vec<String> = Vec::new();

    for repo in &repos {
        let repo_workers: Vec<&WorkerState> =
            workers.iter().filter(|w| w.repo_name() == *repo).collect();

        let mut table = Table::new();
        table
            .load_preset(preset)
            .enforce_styling()
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(table_header());

        for w in &repo_workers {
            table.add_row(worker_row(w));
        }

        let table_str = table.to_string();
        let indented: String = table_str
            .lines()
            .map(|line| format!("  {}", line))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("\x1B[1m{}\x1B[0m\n{}", repo, indented));
    }

    sections.join("\n\n")
}

// ---------------------------------------------------------------------------
// Triage table
// ---------------------------------------------------------------------------

/// Standard triage table header cells.
fn triage_header() -> Vec<Cell> {
    vec![
        Cell::new("ISSUE").add_attribute(Attribute::Bold),
        Cell::new("WORKER").add_attribute(Attribute::Bold),
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
        Cell::new(&t.worker_name).fg(Color::White),
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
    let mut table = Table::new();
    let preset = if borders {
        presets::UTF8_BORDERS_ONLY
    } else {
        presets::NOTHING
    };
    table
        .load_preset(preset)
        .enforce_styling()
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(triage_header());

    for t in triages {
        table.add_row(triage_row(t));
    }

    table
}

/// Render the full triage section with header. Returns empty string if no triages.
pub fn render_triage_section(triages: &[TriageEntry], borders: bool) -> String {
    if triages.is_empty() {
        return String::new();
    }
    let table = render_triage_table(triages, borders);
    if ui::is_plain() {
        format!("TRIAGES\n{}", table)
    } else {
        format!("\x1B[1mTRIAGES\x1B[0m\n{}", table)
    }
}

/// Render triage section grouped by repo, with bold repo headers.
pub fn render_triage_section_grouped(triages: &[TriageEntry], borders: bool) -> String {
    if triages.is_empty() {
        return String::new();
    }

    let mut repos: Vec<String> = Vec::new();
    for t in triages {
        if !repos.contains(&t.repo_name) {
            repos.push(t.repo_name.clone());
        }
    }

    let preset = if borders {
        presets::UTF8_BORDERS_ONLY
    } else {
        presets::NOTHING
    };

    let mut sections: Vec<String> = Vec::new();

    for repo in &repos {
        let repo_triages: Vec<&TriageEntry> =
            triages.iter().filter(|t| &t.repo_name == repo).collect();

        let mut table = Table::new();
        table
            .load_preset(preset)
            .enforce_styling()
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(triage_header());

        for t in &repo_triages {
            table.add_row(triage_row(t));
        }

        let table_str = table.to_string();
        let indented: String = table_str
            .lines()
            .map(|line| format!("  {}", line))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("\x1B[1m{}\x1B[0m\n{}", repo, indented));
    }

    if ui::is_plain() {
        format!("TRIAGES\n{}", sections.join("\n\n"))
    } else {
        format!("\x1B[1mTRIAGES\x1B[0m\n{}", sections.join("\n\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ui;

    fn triage_entry(issue_id: &str, worker: &str, ago_secs: i64, repo: &str) -> TriageEntry {
        TriageEntry {
            issue_id: issue_id.to_string(),
            worker_name: worker.to_string(),
            spawned_at: chrono::Utc::now().timestamp() - ago_secs,
            repo_name: repo.to_string(),
        }
    }

    #[test]
    fn render_triage_section_empty_returns_empty() {
        let section = render_triage_section(&[], false);
        assert!(section.is_empty());
    }

    #[test]
    fn render_triage_section_shows_header_and_entries() {
        ui::set_plain(true);
        let triages = vec![
            triage_entry("JIG-77", "triage-77", 134, "my-repo"),
            triage_entry("JIG-81", "triage-81", 45, "my-repo"),
        ];
        let section = render_triage_section(&triages, false);
        assert!(section.contains("TRIAGES"));
        assert!(section.contains("JIG-77"));
        assert!(section.contains("JIG-81"));
        assert!(section.contains("triage-77"));
        ui::set_plain(false);
    }

    #[test]
    fn render_triage_table_has_correct_columns() {
        ui::set_plain(true);
        let triages = vec![triage_entry("JIG-99", "triage-99", 3661, "test-repo")];
        let table = render_triage_table(&triages, false).to_string();
        assert!(table.contains("ISSUE"));
        assert!(table.contains("WORKER"));
        assert!(table.contains("ELAPSED"));
        assert!(table.contains("REPO"));
        assert!(table.contains("JIG-99"));
        assert!(table.contains("triage-99"));
        assert!(table.contains("test-repo"));
        ui::set_plain(false);
    }

    #[test]
    fn render_triage_section_grouped_empty_returns_empty() {
        let section = render_triage_section_grouped(&[], false);
        assert!(section.is_empty());
    }

    #[test]
    fn render_triage_section_grouped_shows_repo_headers() {
        ui::set_plain(true);
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
        ui::set_plain(false);
    }
}
