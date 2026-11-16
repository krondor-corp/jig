//! jig's TUI: a control hub over every worker jig spawned, on this machine
//! and across the fleet. Plain `jig` opens it.
//!
//! Built on ratatui + crossterm, with pieces ported from octorus
//! (ushironoko/octorus, MIT): the toast rules, the theme's inks and the
//! pane chrome. Unlike octorus it runs no async runtime: like the rest of
//! jig it is std threads, with background work reporting over a channel.
//!
//! * `app`: the state and what a key does to it, as [`app::Effect`]s.
//! * `source`: background threads: the poller (this machine through its
//!   daemon or off disk, peers over the fleet network) and commands.
//! * `workers`: the Workers screen.
//! * `theme`, `toast`: ported from octorus.
//!
//! The loop here owns the terminal: it draws, reads keys, applies messages,
//! and carries out effects, handing the terminal to `jig attach` and taking
//! it back.

mod app;
mod source;
mod theme;
mod toast;
mod workers;

use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::{DefaultTerminal, Frame};

use app::{App, Effect, Mode};
use source::Msg;
use theme::{pane_block, Attention, Theme};
use toast::ToastKind;

use crate::context::{AppCtx, Ctx, Machine};
use crate::daemon::fleet::Fleet;

/// How long the loop waits for a key before looking at its messages.
const POLL: Duration = Duration::from_millis(200);

/// Run the TUI until the user quits.
pub fn run(ctx: Ctx, no_color: bool) -> io::Result<()> {
    let name = Machine::name(&ctx.config);
    // Only ever a client here: whether peers may command us is moot.
    let fleet = match ctx.config.fleet.net(&name) {
        Ok(net) => net.map(|net| Arc::new(Fleet::new(net, &name, false))),
        Err(e) => {
            tracing::warn!("fleet network off: {}", e);
            None
        }
    };
    let app_ctx = Arc::new(AppCtx {
        paths: ctx.paths.clone(),
        config: ctx.config.clone(),
        flags: ctx.flags,
    });
    let interval = Duration::from_secs(ctx.config.tick_interval.max(1));

    let (tx, rx) = flume::unbounded();
    let (refresh_tx, refresh_rx) = flume::unbounded();
    source::spawn_poller(ctx, fleet.clone(), name, interval, refresh_rx, tx.clone());

    let mut app = App::new(Theme::new(no_color));
    let mut terminal = ratatui::init();
    let result = (|| -> io::Result<()> {
        while !app.quit {
            terminal.draw(|f| draw(f, &app))?;
            if event::poll(POLL)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        for effect in app.on_key(key) {
                            match effect {
                                Effect::Refresh => {
                                    let _ = refresh_tx.send(());
                                }
                                Effect::Command { machine, key, cmd } => source::run_command(
                                    Arc::clone(&app_ctx),
                                    fleet.clone(),
                                    machine,
                                    key,
                                    cmd,
                                    tx.clone(),
                                    refresh_tx.clone(),
                                ),
                                Effect::Attach { repo, branch } => {
                                    attach(&mut terminal, &mut app, &repo, &branch)?;
                                    let _ = refresh_tx.send(());
                                }
                            }
                        }
                    }
                }
            }
            for msg in rx.try_iter() {
                match msg {
                    Msg::Fleet(frame) => app.on_frame(frame),
                    Msg::Outcome(toast) => app.on_toast(toast),
                }
            }
            app.tick(Instant::now());
        }
        Ok(())
    })();
    ratatui::restore();
    result
}

/// Hand the terminal to `jig attach <branch>` in `repo`, and take it back
/// when it returns.
fn attach(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    repo: &std::path::Path,
    branch: &str,
) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    let status = std::env::current_exe().and_then(|jig| {
        std::process::Command::new(jig)
            .args(["attach", branch])
            .current_dir(repo)
            .status()
    });
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => app.on_toast(toast::Toast::failure(format!(
            "attach {branch}: exited {s}"
        ))),
        Err(e) => app.on_toast(toast::Toast::failure(format!("attach {branch}: {e}"))),
    }
    Ok(())
}

// ── Drawing ─────────────────────────────────────────────────────────

fn draw(f: &mut Frame, app: &App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .areas(f.area());
    let theme = app.theme;

    let here = app
        .frame
        .as_ref()
        .and_then(|frame| frame.machines.first())
        .map(|m| m.name.clone())
        .unwrap_or_default();
    let mut title = vec![
        Span::styled(" JIG ", theme.brand.bold()),
        Span::styled(here, theme.muted.style()),
    ];
    if let Some(e) = app.frame.as_ref().and_then(|f| f.peers_error.as_ref()) {
        title.push(Span::styled(
            format!("  can't list the other machines: {e}"),
            theme.warning.style(),
        ));
    }
    f.render_widget(Line::from(title), header);

    match &app.frame {
        None => f.render_widget(
            Paragraph::new(Span::styled(" Looking at the fleet…", theme.muted.style())),
            body,
        ),
        Some(frame) => workers::render(f, body, theme, &app.workers, frame),
    }

    f.render_widget(footer_line(app), footer);
    if app.mode == Mode::Help {
        help(f, theme);
    }
}

/// The footer: a prompt or a question when one is up, else the toast, else
/// the keys.
fn footer_line(app: &App) -> Line<'static> {
    let theme = app.theme;
    match &app.mode {
        Mode::Prompt(prompt) => Line::from(vec![
            Span::styled(
                format!(
                    " {} {}: ",
                    prompt.pending.verb.asks().unwrap_or(""),
                    prompt.pending.key
                ),
                theme.accent.bold(),
            ),
            Span::raw(prompt.input.clone()),
            Span::styled("▏", theme.brand.style()),
            Span::styled("   enter send · esc cancel", theme.muted.style()),
        ]),
        Mode::Confirm(pending) => Line::from(vec![
            Span::styled(
                format!(
                    " {} ",
                    pending.verb.question(&pending.key).unwrap_or_default()
                ),
                theme.warning.bold(),
            ),
            Span::styled("y to confirm, any other key to cancel", theme.muted.style()),
        ]),
        _ => match &app.toast {
            Some(toast) => {
                let (mark, ink) = match toast.kind() {
                    ToastKind::Success => ("✓", theme.success),
                    ToastKind::Failure => ("✗", theme.danger),
                };
                Line::from(Span::styled(format!(" {mark} {}", toast.message()), ink.style()))
            }
            None => Line::from(Span::styled(
                " j/k move · tab pane · enter attach · n nudge · m model · x kill · p prune · r retry-setup · R refresh · ? help · q quit",
                theme.muted.style(),
            )),
        },
    }
}

const HELP: [(&str, &str); 14] = [
    ("j / k, ↓ / ↑", "move"),
    ("g / G", "first / last"),
    ("tab, h / l", "switch between machines and workers"),
    (
        "enter",
        "pick a machine; attach to a worker (this machine's)",
    ),
    ("a", "attach"),
    ("n", "nudge: type a message to the agent"),
    ("m", "switch the agent's model"),
    ("x", "kill: close its window (asks first)"),
    ("p", "prune: remove its worktree (asks first)"),
    ("r", "retry-setup after a failed on_create"),
    ("R", "refresh now"),
    ("esc", "dismiss a failure, back out of a prompt"),
    ("?", "this help (any key closes it)"),
    ("q, ctrl-c", "quit"),
];

fn help(f: &mut Frame, theme: Theme) {
    let area = centered(f.area(), 72, HELP.len() as u16 + 2);
    let lines: Vec<Line> = HELP
        .iter()
        .map(|(keys, what)| {
            Line::from(vec![
                Span::styled(format!(" {keys:<14}"), theme.accent.bold()),
                Span::raw(*what),
            ])
        })
        .collect();
    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(lines).block(pane_block(theme, "Keys", "", Attention::Focused)),
        area,
    );
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::source::{FleetFrame, MachineFrame};
    use super::*;
    use crate::worker::events::WorkerState;
    use crate::worker::{MuxStatus, WorkerStatus};
    use jig_core::git::Branch;
    use jig_core::net::NodeId;

    fn screen(app: &App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(140, 30)).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn stalled(name: &str) -> WorkerState {
        WorkerState {
            name: name.into(),
            resolved_branch: Branch::new(name),
            status: WorkerStatus::Stalled,
            mux_status: MuxStatus::Running,
            ..Default::default()
        }
    }

    #[test]
    fn it_opens_on_what_needs_you_across_the_fleet() {
        let mut app = App::new(Theme::new(true));
        app.on_frame(FleetFrame {
            machines: vec![
                MachineFrame {
                    name: "alice".into(),
                    node: None,
                    workers: vec![],
                    error: None,
                    seen_at: None,
                    daemon: true,
                },
                MachineFrame {
                    name: "bob".into(),
                    node: Some(NodeId::new("127.0.0.12")),
                    workers: vec![stalled("feat/stuck")],
                    error: None,
                    seen_at: None,
                    daemon: true,
                },
            ],
            peers_error: None,
        });
        let text = screen(&app);
        assert!(text.contains("NEEDS ATTENTION"), "{text}");
        assert!(text.contains("feat/stuck"), "{text}");
        assert!(text.contains("stalled"), "{text}");
        assert!(text.contains("bob"), "{text}");
        assert!(text.contains("this machine"), "{text}");
    }

    #[test]
    fn a_machine_shows_its_workers_as_ps_would() {
        let mut app = App::new(Theme::new(true));
        let mut failed = stalled("feat/broken");
        failed.status = WorkerStatus::Failed;
        failed.setup = crate::worker::events::SetupState::Failed {
            phase: crate::worker::events::SetupPhase::OnCreate,
            exit: crate::worker::events::SetupExit::Code(1),
            tail: vec!["error: lockfile out of date".into()],
            log_path: None,
        };
        app.on_frame(FleetFrame {
            machines: vec![MachineFrame {
                name: "alice".into(),
                node: None,
                workers: vec![stalled("feat/stuck"), failed],
                error: None,
                seen_at: None,
                daemon: true,
            }],
            peers_error: None,
        });
        app.workers.place = 1;
        let text = screen(&app);
        println!("{text}");
        assert!(text.contains("STATE"), "{text}");
        assert!(text.contains("failed: on_create (exit 1)"), "{text}");
        // The selected worker's detail: its setup's last line.
        assert!(text.contains("lockfile out of date"), "{text}");
    }

    #[test]
    fn the_help_lists_the_keys() {
        let mut app = App::new(Theme::new(true));
        app.on_frame(FleetFrame::default());
        app.mode = Mode::Help;
        let text = screen(&app);
        assert!(
            text.contains("retry-setup after a failed on_create"),
            "{text}"
        );
    }
}
