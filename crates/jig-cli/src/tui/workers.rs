//! The Workers screen: a sidebar of places (the overview, then each
//! machine) beside that place's workers, and the selected worker in detail.
//!
//! The overview lists every worker on every machine that needs a human, by
//! [`attention`], the same rule `jig ps` callers use. A machine lists its
//! workers by repo, each row the `jig ps` columns ([`worker_columns`]), so
//! the TUI and the terminal table can't drift.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState, Wrap};
use ratatui::Frame;

use super::source::{FleetFrame, MachineFrame};
use super::theme::{pane_block, Attention, Theme};
use crate::cli::commands::ps::render::{attention, worker_columns, WORKER_HEADERS};
use crate::cli::ui;
use crate::worker::events::{SetupState, WorkerState};

/// Which pane has the keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    Machines,
    #[default]
    Table,
}

/// The screen's selection. `place` 0 is the overview; `place` n is the
/// frame's machine n-1.
#[derive(Debug, Default)]
pub struct Workers {
    pub focus: Focus,
    pub place: usize,
    pub row: usize,
}

/// A worker on the screen: which machine, which of its workers.
type At = (usize, usize);

impl Workers {
    /// The rows the selected place shows, in order.
    fn rows(&self, frame: &FleetFrame) -> Vec<At> {
        rows(frame, self.place)
    }

    pub fn selected<'a>(
        &self,
        frame: &'a FleetFrame,
    ) -> Option<(&'a MachineFrame, &'a WorkerState)> {
        let (m, w) = *self.rows(frame).get(self.row)?;
        let machine = &frame.machines[m];
        Some((machine, &machine.workers[w]))
    }

    pub fn down(&mut self, frame: &FleetFrame) {
        match self.focus {
            Focus::Machines => {
                self.place = (self.place + 1).min(frame.machines.len());
                self.row = 0;
            }
            Focus::Table => {
                let last = self.rows(frame).len().saturating_sub(1);
                self.row = (self.row + 1).min(last);
            }
        }
    }

    pub fn up(&mut self, _frame: &FleetFrame) {
        match self.focus {
            Focus::Machines => {
                self.place = self.place.saturating_sub(1);
                self.row = 0;
            }
            Focus::Table => self.row = self.row.saturating_sub(1),
        }
    }

    pub fn top(&mut self) {
        match self.focus {
            Focus::Machines => self.place = 0,
            Focus::Table => self.row = 0,
        }
    }

    pub fn bottom(&mut self, frame: &FleetFrame) {
        match self.focus {
            Focus::Machines => self.place = frame.machines.len(),
            Focus::Table => self.row = self.rows(frame).len().saturating_sub(1),
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Machines => Focus::Table,
            Focus::Table => Focus::Machines,
        };
    }

    /// Keep the selection on the same machine and worker across a refresh,
    /// when they are still there.
    pub fn follow(&mut self, old: Option<&FleetFrame>, new: &FleetFrame) {
        let Some(old) = old else {
            return;
        };
        // What was selected, by name, in the old frame.
        let place = (self.place > 0)
            .then(|| old.machines.get(self.place - 1).map(|m| m.name.clone()))
            .flatten();
        let row = rows(old, self.place).get(self.row).map(|&(m, w)| {
            (
                old.machines[m].name.clone(),
                old.machines[m].workers[w].name.clone(),
            )
        });
        // Where it is now.
        if let Some(name) = place {
            self.place = new
                .machines
                .iter()
                .position(|m| m.name == name)
                .map_or(0, |i| i + 1);
        }
        let new_rows = rows(new, self.place);
        self.row = row
            .and_then(|(machine, worker)| {
                new_rows.iter().position(|&(m, w)| {
                    new.machines[m].name == machine && new.machines[m].workers[w].name == worker
                })
            })
            .unwrap_or(self.row)
            .min(new_rows.len().saturating_sub(1));
    }
}

/// `place`'s rows: the overview's flagged workers, or a machine's workers by
/// repo then name.
fn rows(frame: &FleetFrame, place: usize) -> Vec<At> {
    match place {
        0 => frame
            .machines
            .iter()
            .enumerate()
            .flat_map(|(m, machine)| {
                machine
                    .workers
                    .iter()
                    .enumerate()
                    .filter(|(_, w)| attention(w).is_some())
                    .map(move |(w, _)| (m, w))
            })
            .collect(),
        n => {
            let Some(machine) = frame.machines.get(n - 1) else {
                return Vec::new();
            };
            let mut at: Vec<At> = (0..machine.workers.len()).map(|w| (n - 1, w)).collect();
            at.sort_by_key(|&(_, w)| {
                let w = &machine.workers[w];
                (w.repo_name(), w.name.clone())
            });
            at
        }
    }
}

/// How many workers on `machine` need a human.
fn flagged(machine: &MachineFrame) -> usize {
    machine
        .workers
        .iter()
        .filter(|w| attention(w).is_some())
        .count()
}

// ── Drawing ─────────────────────────────────────────────────────────

pub fn render(f: &mut Frame, area: Rect, theme: Theme, state: &Workers, frame: &FleetFrame) {
    let [side, main] =
        Layout::horizontal([Constraint::Length(32), Constraint::Min(20)]).areas(area);
    let [table, detail] = Layout::vertical([Constraint::Min(5), Constraint::Length(9)]).areas(main);
    sidebar(f, side, theme, state, frame);
    workers(f, table, theme, state, frame);
    worker_detail(f, detail, theme, state, frame);
}

fn sidebar(f: &mut Frame, area: Rect, theme: Theme, state: &Workers, frame: &FleetFrame) {
    let attention_on = Attention::when(state.focus == Focus::Machines);
    let total: usize = frame
        .machines
        .iter()
        .map(|m| flagged(m) + usize::from(!m.is_up()))
        .sum();
    let mut items = vec![ListItem::new(Line::from(vec![
        Span::raw("Overview "),
        badge(theme, total),
    ]))];
    items.extend(frame.machines.iter().map(|m| {
        let (glyph, ink, state) = match (m.is_up(), m.seen_at) {
            (true, _) if m.is_local() && !m.daemon => ("●", theme.warning, "no daemon".to_string()),
            (true, _) => (
                "●",
                theme.success,
                if m.is_local() {
                    "this machine".into()
                } else {
                    "up".into()
                },
            ),
            (false, Some(at)) => ("○", theme.warning, format!("stale {}", ago(at))),
            (false, None) => ("✕", theme.danger, "not answering".into()),
        };
        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("{glyph} "), ink.style()),
                Span::raw(m.name.clone()),
                Span::raw(" "),
                badge(theme, flagged(m)),
            ]),
            Line::from(Span::styled(
                format!("  {state} · {} workers", m.workers.len()),
                theme.muted.style(),
            )),
        ])
    }));
    let list = List::new(items)
        .block(pane_block(theme, "Machines", "", attention_on))
        .highlight_symbol(attention_on.marker())
        .highlight_style(attention_on.selected(theme));
    let mut list_state = ListState::default().with_selected(Some(state.place));
    f.render_stateful_widget(list, area, &mut list_state);
}

fn badge(theme: Theme, n: usize) -> Span<'static> {
    if n == 0 {
        Span::raw("")
    } else {
        Span::styled(format!("({n})"), theme.danger.bold())
    }
}

fn workers(f: &mut Frame, area: Rect, theme: Theme, state: &Workers, frame: &FleetFrame) {
    let focus = Attention::when(state.focus == Focus::Table);
    let at = state.rows(frame);
    let now = chrono::Utc::now().timestamp();
    let (title, header, widths, body): (String, Vec<&str>, Vec<Constraint>, Vec<Row>) =
        if state.place == 0 {
            // Machines that aren't answering come first; they can't be
            // selected, so the selection skips past them.
            let down = frame.machines.iter().filter(|m| !m.is_up()).map(|m| {
                let seen = m
                    .seen_at
                    .map(|at| format!(", last seen {} ago", ago(at)))
                    .unwrap_or_default();
                let why = format!("not answering{seen}: {}", m.error.as_deref().unwrap_or(""));
                Row::new(vec![
                    Cell::from(m.name.clone()).style(theme.info.style()),
                    Cell::from("—").style(theme.muted.style()),
                    Cell::from("—").style(theme.muted.style()),
                    Cell::from(why).style(theme.warning.style()),
                ])
            });
            let body = down
                .chain(at.iter().map(|&(m, w)| {
                    let machine = &frame.machines[m];
                    let worker = &machine.workers[w];
                    Row::new(vec![
                        Cell::from(machine.name.clone()).style(theme.info.style()),
                        Cell::from(worker.repo_name()),
                        Cell::from(worker.name.clone()),
                        Cell::from(attention(worker).unwrap_or_default())
                            .style(theme.danger.style()),
                    ])
                }))
                .collect();
            (
                "Needs attention".into(),
                vec!["MACHINE", "REPO", "WORKER", "WHY"],
                vec![
                    Constraint::Length(12),
                    Constraint::Length(14),
                    Constraint::Min(20),
                    Constraint::Min(24),
                ],
                body,
            )
        } else {
            let machine = &frame.machines[state.place - 1];
            let body = at
                .iter()
                .map(|&(_, w)| {
                    let worker = &machine.workers[w];
                    let cells =
                        std::iter::once(Cell::from(worker.repo_name()).style(theme.muted.style()))
                            .chain(
                                worker_columns(worker, now)
                                    .into_iter()
                                    .map(|c| Cell::from(c.text).style(theme.ps(c.color))),
                            );
                    Row::new(cells.collect::<Vec<_>>())
                })
                .collect();
            // REPO, then `ps`'s columns. The short ones get what they need
            // and WORKER and STATE share the rest.
            let widths = vec![
                Constraint::Length(12),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Length(7),
                Constraint::Length(8),
                Constraint::Length(7),
                Constraint::Length(10),
                Constraint::Length(12),
            ];
            (
                machine.name.clone(),
                std::iter::once("REPO").chain(WORKER_HEADERS).collect(),
                widths,
                body,
            )
        };

    let empty = body.is_empty();
    let detail = format!("({})", body.len());
    let block = pane_block(theme, &title, &detail, focus);
    if empty {
        let message = match state.place {
            0 => "Nothing needs you right now.".to_string(),
            n => match &frame.machines[n - 1] {
                m if !m.is_up() && m.seen_at.is_none() => format!(
                    "{} is not answering: {}",
                    m.name,
                    m.error.as_deref().unwrap_or("")
                ),
                _ => "No spawned sessions.".to_string(),
            },
        };
        f.render_widget(
            Paragraph::new(Span::styled(message, theme.muted.style())).block(block),
            area,
        );
        return;
    }
    // On the overview, rows for machines that aren't answering sit above
    // the selectable ones.
    let skip = if state.place == 0 {
        frame.machines.iter().filter(|m| !m.is_up()).count()
    } else {
        0
    };
    let selected = (!at.is_empty()).then_some(state.row + skip);
    let table = Table::new(body, widths)
        .header(Row::new(header).style(theme.muted.bold()))
        .block(block)
        .highlight_symbol(focus.marker())
        .row_highlight_style(focus.selected(theme));
    let mut table_state = TableState::default().with_selected(selected);
    f.render_stateful_widget(table, area, &mut table_state);
}

fn worker_detail(f: &mut Frame, area: Rect, theme: Theme, state: &Workers, frame: &FleetFrame) {
    let block = pane_block(theme, "Worker", "", Attention::Dim);
    let Some((machine, w)) = state.selected(frame) else {
        let hint = if state.place > 0 && !frame.machines[state.place - 1].is_up() {
            frame.machines[state.place - 1]
                .error
                .clone()
                .unwrap_or_default()
        } else {
            String::new()
        };
        f.render_widget(
            Paragraph::new(Span::styled(hint, theme.muted.style())).block(block),
            area,
        );
        return;
    };
    let field = |label: &str, value: String, style: Style| {
        Line::from(vec![
            Span::styled(format!("{label:>9}  "), theme.muted.style()),
            Span::styled(value, style),
        ])
    };
    let mut lines = vec![field(
        "worker",
        format!("{} · {} on {}", w.name, w.repo_name(), machine.name),
        theme.accent.bold(),
    )];
    if let Some(why) = attention(w) {
        lines.push(field("needs you", why, theme.danger.style()));
    }
    if let Some(url) = &w.pr_url {
        lines.push(field("pr", url.to_string(), Style::default()));
    }
    if let Some(issue) = &w.issue_ref {
        lines.push(field("issue", issue.to_string(), Style::default()));
    }
    if let Some(agent) = w.mux_agent_state {
        lines.push(field("agent", agent.to_string(), Style::default()));
    }
    if let SetupState::Failed { tail, .. } = &w.setup {
        for line in tail.iter().rev().take(3).rev() {
            lines.push(field("setup", line.clone(), theme.danger.style()));
        }
    }
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(block),
        area,
    );
}

/// "3m" since `at`, in unix seconds.
fn ago(at: i64) -> String {
    ui::format_duration_short((chrono::Utc::now().timestamp() - at).max(0) as u64)
}
