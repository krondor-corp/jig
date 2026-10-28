//! The work view as HTML.
//!
//! Every worker row comes from [`worker_columns`] — the same text and colors
//! `jig ps` puts in its table — so the page and the terminal cannot drift.
//! What the page adds is what a terminal table can't hold: links, the
//! failed-setups panel with its output tails, and the command buttons.

use std::fmt::Write;

use comfy_table::Color;
use url::Url;

use jig_core::git::Branch;

use super::super::ipc::DaemonStatus;
use crate::cli::commands::ps::render::{format_state, worker_columns, WORKER_HEADERS};
use crate::cli::ui;
use crate::worker::events::{SetupState, WorkerState};

/// The whole document. `token` is the session's command token: embedded
/// here, where only a same-origin script can read it.
pub fn document(status: &DaemonStatus, token: &str, now: i64) -> String {
    let title = match &status.info.machine {
        Some(m) => format!("jig · {}", m.name),
        None => "jig".to_string(),
    };
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="jig-token" content="{token}">
<title>{title}</title>
<link rel="stylesheet" href="/app.css">
<script src="/app.js" defer></script>
</head>
<body>
<div id="result" role="status" hidden></div>
<main id="view">
{view}
</main>
</body>
</html>
"#,
        token = escape(token),
        title = escape(&title),
        view = view(status, now),
    )
}

/// The part of the page that refreshes: everything the daemon knows now.
pub fn view(status: &DaemonStatus, now: i64) -> String {
    let workers: Vec<WorkerState> = status
        .workers
        .iter()
        .map(|w| w.to_display_state())
        .collect();

    let mut out = String::new();
    let machine = status
        .info
        .machine
        .as_ref()
        .map(|m| m.name.as_str())
        .unwrap_or("this machine");
    let _ = write!(
        out,
        r#"<header><h1>jig <span class="c-dim">· {}</span></h1><p class="c-dim">{} workers · {} triages · next poll in {} · daemon pid {} · v{}</p></header>"#,
        escape(machine),
        workers.len(),
        status.triages.len(),
        escape(&ui::format_duration_short(status.poll_remaining)),
        status.info.pid,
        escape(&status.info.version),
    );

    out.push_str(&failed_setups(&workers, now));

    if workers.is_empty() && status.triages.is_empty() {
        out.push_str(r#"<p class="empty">No spawned sessions</p>"#);
    }
    out.push_str(&worker_tables(&workers, now));
    out.push_str(&triages(status, now));

    if !status.spawning.is_empty() {
        let names: Vec<String> = status.spawning.iter().map(Branch::to_string).collect();
        let _ = write!(
            out,
            r#"<p><span class="c-dim">spawning:</span> <span class="c-yellow">{}</span></p>"#,
            escape(&names.join(", "))
        );
    }
    out
}

/// The `WorkerKey` a command for `w` goes to, as text.
fn key_of(w: &WorkerState) -> String {
    format!("{}/{}", w.repo_name(), w.name)
}

/// One table per repo, in the order repos are first seen — `ps -g`'s
/// grouping, since the daemon always watches every repo.
fn worker_tables(workers: &[WorkerState], now: i64) -> String {
    let mut repos: Vec<String> = Vec::new();
    for w in workers {
        let repo = w.repo_name();
        if !repos.contains(&repo) {
            repos.push(repo);
        }
    }

    let mut out = String::new();
    for repo in repos {
        let _ = write!(
            out,
            "<section class=\"repo\"><h2>{}</h2><table><thead><tr>",
            escape(&repo)
        );
        for h in WORKER_HEADERS {
            let _ = write!(out, "<th>{h}</th>");
        }
        out.push_str("<th></th></tr></thead><tbody>");
        for w in workers.iter().filter(|w| w.repo_name() == repo) {
            out.push_str(&worker_row(w, now));
        }
        out.push_str("</tbody></table></section>");
    }
    out
}

fn worker_row(w: &WorkerState, now: i64) -> String {
    let columns = worker_columns(w, now);
    let mut out = String::from("<tr>");
    for (i, c) in columns.iter().enumerate() {
        let mut class = class_of(c.color).to_string();
        if c.right {
            class.push_str(" right");
        }
        let text = escape(&c.text);
        let cell = match i {
            // The name is truncated to fit a terminal; the tooltip has it
            // whole, with the agent's live state.
            0 => {
                let agent = w
                    .mux_agent_state
                    .map(|a| format!(" · agent {a}"))
                    .unwrap_or_default();
                format!(
                    r#"<span title="{}{}">{text}</span>"#,
                    escape(&w.name),
                    escape(&agent)
                )
            }
            4 => match w.pr_url.as_ref().filter(|u| is_web_url(u)) {
                Some(url) => format!(
                    r#"<a href="{}" target="_blank" rel="noopener noreferrer">{text}</a>"#,
                    escape(url.as_str())
                ),
                None => text,
            },
            _ => text,
        };
        let _ = write!(out, r#"<td class="{}">{cell}</td>"#, class.trim());
    }
    let _ = write!(out, r#"<td class="actions">{}</td></tr>"#, buttons(w));
    out
}

/// The command buttons for one worker. `retry-setup` only shows where it
/// can work; the daemon refuses the rest with a reason the page shows.
fn buttons(w: &WorkerState) -> String {
    let key = escape(&key_of(w));
    let mut verbs = vec!["nudge", "model", "kill", "prune"];
    if matches!(w.setup, SetupState::Failed { .. }) {
        verbs.push("retry-setup");
    }
    verbs
        .iter()
        .map(|v| {
            format!(r#"<button type="button" data-verb="{v}" data-worker="{key}">{v}</button>"#)
        })
        .collect()
}

/// Workers whose setup failed, with the end of the output and the whole
/// `setup.log` a click away.
fn failed_setups(workers: &[WorkerState], now: i64) -> String {
    let failed: Vec<&WorkerState> = workers
        .iter()
        .filter(|w| matches!(w.setup, SetupState::Failed { .. }))
        .collect();
    if failed.is_empty() {
        return String::new();
    }

    let mut out = String::from(r#"<section class="failed"><h2>Failed setups</h2>"#);
    for w in failed {
        let SetupState::Failed { tail, log_path, .. } = &w.setup else {
            continue;
        };
        let key = key_of(w);
        let (state, _) = format_state(w, now);
        let _ = write!(
            out,
            r#"<article><h3>{} <span class="c-red">{}</span></h3>"#,
            escape(&key),
            escape(&state)
        );
        if !tail.is_empty() {
            let _ = write!(out, "<pre>{}</pre>", escape(&tail.join("\n")));
        }
        out.push_str("<p>");
        if log_path.is_some() {
            let _ = write!(
                out,
                r#"<a href="/setup-log?worker={}" target="_blank">setup.log</a> "#,
                escape(&urlencoding::encode(&key))
            );
        }
        let key = escape(&key);
        let _ = write!(
            out,
            r#"<button type="button" data-verb="retry-setup" data-worker="{key}">retry-setup</button> <button type="button" data-verb="prune" data-worker="{key}">prune</button></p></article>"#
        );
    }
    out.push_str("</section>");
    out
}

fn triages(status: &DaemonStatus, now: i64) -> String {
    if status.triages.is_empty() {
        return String::new();
    }
    let mut out = String::from(
        "<section><h2>Triages</h2><table><thead><tr><th>ISSUE</th><th>MODEL</th><th>ELAPSED</th><th>REPO</th></tr></thead><tbody>",
    );
    for t in &status.triages {
        let elapsed = (now - t.spawned_at).max(0) as u64;
        let _ = write!(
            out,
            r#"<tr><td class="c-cyan">{}</td><td>{}</td><td class="right">{}</td><td class="c-dim">{}</td></tr>"#,
            escape(&t.issue_id),
            escape(&t.model),
            escape(&ui::format_duration_short(elapsed)),
            escape(&t.repo_name),
        );
    }
    out.push_str("</tbody></table></section>");
    out
}

/// The CSS class for a `ps` color.
fn class_of(color: Color) -> &'static str {
    match color {
        Color::Green | Color::DarkGreen => "c-green",
        Color::Red | Color::DarkRed => "c-red",
        Color::Yellow | Color::DarkYellow => "c-yellow",
        Color::Blue | Color::DarkBlue => "c-blue",
        Color::Cyan | Color::DarkCyan => "c-cyan",
        Color::Magenta | Color::DarkMagenta => "c-magenta",
        Color::DarkGrey | Color::Grey => "c-dim",
        _ => "",
    }
}

/// Only link what a browser should follow: never a `javascript:` URL.
fn is_web_url(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
}

/// Escape text for an HTML element or a quoted attribute.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::context::RepoEntry;
    use crate::daemon::ipc::{DaemonInfo, WorkerSnapshot};
    use crate::worker::events::{SetupExit, SetupPhase};
    use crate::worker::{MuxStatus, WorkerStatus};

    fn worker(name: &str) -> WorkerState {
        let now = chrono::Utc::now();
        WorkerState {
            name: name.into(),
            resolved_branch: Branch::new(name),
            status: WorkerStatus::Running,
            mux_status: MuxStatus::Running,
            repo: Some(RepoEntry {
                path: PathBuf::from("/src/jig"),
                added: now,
                last_used: now,
            }),
            ..Default::default()
        }
    }

    fn status(workers: &[WorkerState]) -> DaemonStatus {
        DaemonStatus {
            info: DaemonInfo {
                pid: 42,
                version: "0.0.0".into(),
                started_at: 0,
                ticked_at: 0,
                tick_interval: 2,
                log: None,
                actors: vec![],
                machine: None,
                web: None,
            },
            workers: workers.iter().map(WorkerSnapshot::from).collect(),
            triages: vec![],
            spawning: vec![],
            poll_remaining: 0,
        }
    }

    #[test]
    fn every_ps_column_shows_up_in_the_row() {
        let mut w = worker("feat/web");
        w.pr_url = Url::parse("https://github.com/o/r/pull/12").ok();
        w.issue_ref = Some("KRO-250".into());
        w.commits_ahead = 3;
        let html = view(&status(std::slice::from_ref(&w)), 0);

        // Round-tripped through the snapshot, as the daemon serves it.
        let display = WorkerSnapshot::from(&w).to_display_state();
        for column in worker_columns(&display, 0) {
            assert!(
                html.contains(&escape(&column.text)),
                "missing {:?} in {html}",
                column.text
            );
        }
        for header in WORKER_HEADERS {
            assert!(html.contains(&format!("<th>{header}</th>")));
        }
        assert!(html.contains(r#"href="https://github.com/o/r/pull/12""#));
    }

    #[test]
    fn buttons_address_the_worker_by_key() {
        let html = view(&status(&[worker("feat/web")]), 0);
        for verb in ["nudge", "model", "kill", "prune"] {
            assert!(html.contains(&format!(r#"data-verb="{verb}" data-worker="jig/feat/web""#)));
        }
        // Only a failed setup can be retried.
        assert!(!html.contains("retry-setup"));
    }

    #[test]
    fn a_failed_setup_gets_a_panel_with_its_tail_and_log() {
        let mut w = worker("feat/broken");
        w.status = WorkerStatus::Failed;
        w.setup = SetupState::Failed {
            phase: SetupPhase::OnCreate,
            exit: SetupExit::Code(1),
            tail: vec!["npm ERR! <boom>".into()],
            log_path: Some(PathBuf::from("/tmp/setup.log")),
        };
        let html = view(&status(&[w]), 0);
        assert!(html.contains("Failed setups"));
        assert!(html.contains("failed: on_create (exit 1)"));
        assert!(html.contains("npm ERR! &lt;boom&gt;"));
        assert!(html.contains(r#"href="/setup-log?worker=jig%2Ffeat%2Fbroken""#));
        assert!(html.contains(r#"data-verb="retry-setup" data-worker="jig/feat/broken""#));
        // The log's path on disk is not the page's business.
        assert!(!html.contains("/tmp/setup.log"));
    }

    #[test]
    fn no_failures_no_panel() {
        assert!(!view(&status(&[worker("feat/ok")]), 0).contains("Failed setups"));
    }

    #[test]
    fn worker_text_is_escaped() {
        let html = view(&status(&[worker("feat/<script>")]), 0);
        assert!(!html.contains("<script>"));
        assert!(html.contains("feat/&lt;script&gt;"));
    }

    #[test]
    fn a_non_web_pr_url_is_not_linked() {
        let mut w = worker("feat/x");
        w.pr_url = Url::parse("javascript:alert(1)").ok();
        let html = view(&status(&[w]), 0);
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn the_document_carries_the_token() {
        let html = document(&status(&[]), "abc123", 0);
        assert!(html.contains(r#"<meta name="jig-token" content="abc123">"#));
        assert!(html.contains("No spawned sessions"));
    }
}
