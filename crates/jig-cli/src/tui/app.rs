//! The TUI's state and what a key does to it.
//!
//! Keys never act directly: they change the state and return [`Effect`]s
//! for the event loop (`super::run`) to carry out (a command on a thread, a
//! refresh, handing the terminal to `jig attach`). So everything here runs
//! without a terminal or a daemon, and the tests drive it with keys alone.

use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use jig_core::net::NodeId;

use super::source::FleetFrame;
use super::theme::Theme;
use super::toast::{self, Toast};
use super::workers::{Focus, Workers};
use crate::daemon::commands::{Command, WorkerKey};

/// Something the event loop does on the app's behalf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Run a verb on a worker, on its machine (`None`: this one).
    Command {
        machine: Option<NodeId>,
        key: WorkerKey,
        cmd: Command,
    },
    /// Look at every machine again now.
    Refresh,
    /// Leave the TUI for `jig attach <branch>` in `repo`, then come back.
    Attach {
        repo: std::path::PathBuf,
        branch: String,
    },
}

/// What the keys are doing right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Help,
    /// Typing the argument a verb needs (a nudge's text, a model's name).
    Prompt(Prompt),
    /// `y` to run a verb that can't be undone, anything else to back out.
    Confirm(Pending),
}

/// A verb waiting on its argument or a confirmation, and its worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub verb: Verb,
    pub machine: Option<NodeId>,
    pub key: WorkerKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub pending: Pending,
    pub input: String,
}

/// The verbs a worker row offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Nudge,
    Model,
    Kill,
    Prune,
    RetrySetup,
}

impl Verb {
    /// The question before a verb that destroys something.
    pub fn question(self, key: &WorkerKey) -> Option<String> {
        match self {
            Verb::Kill => Some(format!("kill {key}? its window closes; the worktree stays")),
            Verb::Prune => Some(format!(
                "prune {key}? its worktree is removed, uncommitted changes included"
            )),
            _ => None,
        }
    }

    /// What a prompt for this verb asks for.
    pub fn asks(self) -> Option<&'static str> {
        match self {
            Verb::Nudge => Some("nudge with"),
            Verb::Model => Some("switch to model"),
            _ => None,
        }
    }

    /// The command, given the prompt's answer.
    fn command(self, input: String) -> Command {
        match self {
            Verb::Nudge => Command::Nudge { text: input },
            Verb::Model => Command::Model { model: input },
            Verb::Kill => Command::Kill,
            Verb::Prune => Command::Prune,
            Verb::RetrySetup => Command::RetrySetup,
        }
    }
}

pub struct App {
    pub theme: Theme,
    /// The latest look at the fleet; `None` until the first one lands.
    pub frame: Option<FleetFrame>,
    pub workers: Workers,
    pub mode: Mode,
    pub toast: Option<Toast>,
    pub quit: bool,
}

impl App {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            frame: None,
            workers: Workers::default(),
            mode: Mode::Normal,
            toast: None,
            quit: false,
        }
    }

    /// A fresh look at the fleet. Keeps the selection on the same machine
    /// and worker when they are still there.
    pub fn on_frame(&mut self, frame: FleetFrame) {
        self.workers.follow(self.frame.as_ref(), &frame);
        self.frame = Some(frame);
    }

    pub fn on_toast(&mut self, toast: Toast) {
        self.toast = Some(toast);
    }

    /// Called every loop turn: lets a success toast run out.
    pub fn tick(&mut self, now: Instant) {
        toast::expire(&mut self.toast, now);
    }

    /// One key press, and what the event loop should do about it.
    pub fn on_key(&mut self, key: KeyEvent) -> Vec<Effect> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return Vec::new();
        }
        if toast::apply_key(&mut self.toast, key.code == KeyCode::Esc, Instant::now()) {
            return Vec::new();
        }
        match std::mem::replace(&mut self.mode, Mode::Normal) {
            Mode::Normal => self.normal(key),
            Mode::Help => Vec::new(),
            Mode::Prompt(prompt) => self.prompt(prompt, key),
            Mode::Confirm(pending) => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => vec![run(pending, String::new())],
                _ => Vec::new(),
            },
        }
    }

    fn normal(&mut self, key: KeyEvent) -> Vec<Effect> {
        let Some(frame) = &self.frame else {
            if matches!(key.code, KeyCode::Char('q')) {
                self.quit = true;
            }
            return Vec::new();
        };
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?') => self.mode = Mode::Help,
            KeyCode::Char('R') => return vec![Effect::Refresh],
            KeyCode::Char('j') | KeyCode::Down => self.workers.down(frame),
            KeyCode::Char('k') | KeyCode::Up => self.workers.up(frame),
            KeyCode::Char('g') | KeyCode::Home => self.workers.top(),
            KeyCode::Char('G') | KeyCode::End => self.workers.bottom(frame),
            KeyCode::Tab | KeyCode::BackTab => self.workers.toggle_focus(),
            KeyCode::Char('h') | KeyCode::Left => self.workers.focus = Focus::Machines,
            KeyCode::Char('l') | KeyCode::Right => self.workers.focus = Focus::Table,
            KeyCode::Enter if self.workers.focus == Focus::Machines => {
                self.workers.focus = Focus::Table
            }
            KeyCode::Enter | KeyCode::Char('a') => return self.attach(),
            KeyCode::Char('n') => return self.begin(Verb::Nudge),
            KeyCode::Char('m') => return self.begin(Verb::Model),
            KeyCode::Char('x') => return self.begin(Verb::Kill),
            KeyCode::Char('p') => return self.begin(Verb::Prune),
            KeyCode::Char('r') => return self.begin(Verb::RetrySetup),
            _ => {}
        }
        Vec::new()
    }

    /// Start `verb` on the selected worker: ask for its argument, or for a
    /// confirmation, or run it.
    fn begin(&mut self, verb: Verb) -> Vec<Effect> {
        let Some((machine, key)) = self.selected_target() else {
            return Vec::new();
        };
        let pending = Pending { verb, machine, key };
        if verb.asks().is_some() {
            self.mode = Mode::Prompt(Prompt {
                pending,
                input: String::new(),
            });
        } else if verb.question(&pending.key).is_some() {
            self.mode = Mode::Confirm(pending);
        } else {
            return vec![run(pending, String::new())];
        }
        Vec::new()
    }

    fn prompt(&mut self, mut prompt: Prompt, key: KeyEvent) -> Vec<Effect> {
        match key.code {
            KeyCode::Esc => Vec::new(),
            KeyCode::Enter => {
                let input = prompt.input.trim().to_string();
                if input.is_empty() {
                    return Vec::new();
                }
                vec![run(prompt.pending, input)]
            }
            KeyCode::Backspace => {
                prompt.input.pop();
                self.mode = Mode::Prompt(prompt);
                Vec::new()
            }
            KeyCode::Char(c) => {
                prompt.input.push(c);
                self.mode = Mode::Prompt(prompt);
                Vec::new()
            }
            _ => {
                self.mode = Mode::Prompt(prompt);
                Vec::new()
            }
        }
    }

    /// Attach to the selected worker: this machine's only. A peer's
    /// terminal is that machine's to attach to.
    fn attach(&mut self) -> Vec<Effect> {
        let Some((machine, worker)) = self
            .frame
            .as_ref()
            .and_then(|frame| self.workers.selected(frame))
        else {
            return Vec::new();
        };
        if !machine.is_local() {
            self.toast = Some(Toast::failure(format!(
                "{} runs on {}: attach there",
                worker.name, machine.name
            )));
            return Vec::new();
        }
        match &worker.repo {
            Some(repo) => vec![Effect::Attach {
                repo: repo.path.clone(),
                branch: worker.name.clone(),
            }],
            None => {
                self.toast = Some(Toast::failure(format!(
                    "{}: no repo recorded to attach in",
                    worker.name
                )));
                Vec::new()
            }
        }
    }

    /// The selected worker's machine and key.
    fn selected_target(&self) -> Option<(Option<NodeId>, WorkerKey)> {
        let frame = self.frame.as_ref()?;
        let (machine, worker) = self.workers.selected(frame)?;
        let key = WorkerKey::new(worker.repo_name(), worker.resolved_branch.clone());
        Some((machine.node.clone(), key))
    }
}

fn run(pending: Pending, input: String) -> Effect {
    Effect::Command {
        machine: pending.machine,
        key: pending.key,
        cmd: pending.verb.command(input),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::context::RepoEntry;
    use crate::tui::source::MachineFrame;
    use crate::worker::events::WorkerState;
    use crate::worker::{MuxStatus, WorkerStatus};
    use jig_core::git::Branch;

    fn worker(repo: &str, name: &str) -> WorkerState {
        let now = chrono::Utc::now();
        WorkerState {
            name: name.into(),
            resolved_branch: Branch::new(name),
            status: WorkerStatus::Running,
            mux_status: MuxStatus::Running,
            repo: Some(RepoEntry {
                path: PathBuf::from(format!("/src/{repo}")),
                added: now,
                last_used: now,
            }),
            ..Default::default()
        }
    }

    fn machine(name: &str, node: Option<&str>, workers: Vec<WorkerState>) -> MachineFrame {
        MachineFrame {
            name: name.into(),
            node: node.map(NodeId::new),
            workers,
            error: None,
            seen_at: None,
            daemon: true,
        }
    }

    /// alice (here) with one worker, bob with one.
    fn app() -> App {
        let mut app = App::new(Theme::new(true));
        app.on_frame(FleetFrame {
            machines: vec![
                machine("alice", None, vec![worker("jig", "feat/a")]),
                machine("bob", Some("127.0.0.12"), vec![worker("jig", "feat/b")]),
            ],
            peers_error: None,
        });
        app
    }

    fn press(app: &mut App, code: KeyCode) -> Vec<Effect> {
        app.on_key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn type_text(app: &mut App, text: &str) {
        for c in text.chars() {
            press(app, KeyCode::Char(c));
        }
    }

    /// Select bob in the sidebar and his worker in the table.
    fn select_bob(app: &mut App) {
        press(app, KeyCode::Char('h'));
        // Overview, alice, bob.
        press(app, KeyCode::Char('j'));
        press(app, KeyCode::Char('j'));
        press(app, KeyCode::Enter);
    }

    #[test]
    fn a_nudge_asks_for_its_text_then_runs_on_the_worker_s_machine() {
        let mut app = app();
        select_bob(&mut app);
        assert!(press(&mut app, KeyCode::Char('n')).is_empty());
        assert!(matches!(app.mode, Mode::Prompt(_)));
        type_text(&mut app, "rebase please");
        let effects = press(&mut app, KeyCode::Enter);
        assert_eq!(
            effects,
            [Effect::Command {
                machine: Some(NodeId::new("127.0.0.12")),
                key: "jig/feat/b".parse().unwrap(),
                cmd: Command::Nudge {
                    text: "rebase please".into()
                },
            }]
        );
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn esc_backs_out_of_a_prompt_and_an_empty_one_sends_nothing() {
        let mut app = app();
        select_bob(&mut app);
        press(&mut app, KeyCode::Char('n'));
        type_text(&mut app, "x");
        assert!(press(&mut app, KeyCode::Esc).is_empty());
        assert_eq!(app.mode, Mode::Normal);

        press(&mut app, KeyCode::Char('n'));
        type_text(&mut app, "  ");
        assert!(press(&mut app, KeyCode::Enter).is_empty());
    }

    #[test]
    fn kill_and_prune_run_only_on_y() {
        let mut app = app();
        select_bob(&mut app);
        for verb in ['x', 'p'] {
            press(&mut app, KeyCode::Char(verb));
            assert!(matches!(app.mode, Mode::Confirm(_)));
            assert!(press(&mut app, KeyCode::Char('n')).is_empty(), "{verb}");

            press(&mut app, KeyCode::Char(verb));
            let effects = press(&mut app, KeyCode::Char('y'));
            assert_eq!(effects.len(), 1, "{verb}");
        }
    }

    #[test]
    fn a_local_worker_attaches_and_a_peer_s_says_where_to() {
        let mut app = app();
        // The overview lists nothing here; go to alice's table.
        press(&mut app, KeyCode::Char('h'));
        press(&mut app, KeyCode::Char('j'));
        press(&mut app, KeyCode::Enter);
        assert_eq!(
            press(&mut app, KeyCode::Enter),
            [Effect::Attach {
                repo: PathBuf::from("/src/jig"),
                branch: "feat/a".into()
            }]
        );

        let mut app = self::app();
        select_bob(&mut app);
        assert!(press(&mut app, KeyCode::Enter).is_empty());
        assert!(app
            .toast
            .as_ref()
            .unwrap()
            .message()
            .contains("runs on bob"));
    }

    #[test]
    fn q_quits_but_not_while_typing() {
        let mut app = app();
        select_bob(&mut app);
        press(&mut app, KeyCode::Char('n'));
        press(&mut app, KeyCode::Char('q'));
        assert!(!app.quit);
        press(&mut app, KeyCode::Esc);
        press(&mut app, KeyCode::Char('q'));
        assert!(app.quit);
    }
}
