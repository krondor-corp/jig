//! Control verbs — `kill`, `nudge`, `prune`, `retry-setup`, `model` — run
//! in-process against a stand-in mux, over a real daemon's socket, and
//! through the CLI.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use predicates::prelude::*;

use crate::common::Sandbox;
use jig_cli::context::{register_globally, AppConfig, AppCtx, Flags, RepoCtx};
use jig_cli::daemon::commands::{self, Command, CommandError, WorkerKey};
use jig_cli::daemon::ipc::{self, IpcError, Request, Response};
use jig_cli::worker::events::{EventKind, WorkerState};
use jig_cli::worker::{OnCreate, Worker, WorkerStatus};
use jig_core::git::Branch;
use jig_core::mux::{Mux, MuxError};

const BRANCH: &str = "feat/verbs";

/// A mux that keeps its windows in a set and remembers what was typed.
#[derive(Default)]
struct FakeMux {
    windows: Mutex<HashSet<String>>,
    messages: Mutex<Vec<(String, String)>>,
    keys: Mutex<Vec<(String, String)>>,
}

impl FakeMux {
    fn with_window(name: &str) -> Self {
        let mux = Self::default();
        mux.windows.lock().unwrap().insert(name.to_string());
        mux
    }

    fn has(&self, name: &str) -> bool {
        self.windows.lock().unwrap().contains(name)
    }

    fn messages(&self) -> Vec<(String, String)> {
        self.messages.lock().unwrap().clone()
    }
}

impl Mux for FakeMux {
    fn create_window(&self, name: &str, _dir: &Path) -> Result<(), MuxError> {
        self.windows.lock().unwrap().insert(name.to_string());
        Ok(())
    }
    fn window_exists(&self, name: &str) -> bool {
        self.has(name)
    }
    fn kill_window(&self, name: &str) -> Result<(), MuxError> {
        self.windows.lock().unwrap().remove(name);
        Ok(())
    }
    fn kill_all(&self) -> Result<(), MuxError> {
        self.windows.lock().unwrap().clear();
        Ok(())
    }
    fn send_keys(&self, name: &str, keys: &[&str]) -> Result<(), MuxError> {
        self.keys
            .lock()
            .unwrap()
            .push((name.to_string(), keys.join(" ")));
        Ok(())
    }
    fn send_message(&self, name: &str, message: &str) -> Result<(), MuxError> {
        self.messages
            .lock()
            .unwrap()
            .push((name.to_string(), message.to_string()));
        Ok(())
    }
    fn is_running(&self, name: &str) -> bool {
        self.has(name)
    }
    fn focus_window(&self, _name: &str) -> Result<(), MuxError> {
        Ok(())
    }
    fn focus(&self) -> Result<(), MuxError> {
        Ok(())
    }
    fn connect(&self) -> Result<(), MuxError> {
        Ok(())
    }
}

/// A sandbox repo based on local `main`, with `on_create` set to `hook`.
fn configured(hook: Option<&str>) -> Sandbox {
    let sandbox = Sandbox::with_repo();
    configure(&sandbox, hook);
    sandbox
}

fn configure(sandbox: &Sandbox, hook: Option<&str>) {
    let hook = hook
        .map(|h| format!("on_create = \"{h}\"\n"))
        .unwrap_or_default();
    std::fs::write(
        sandbox.work_dir().join("jig.toml"),
        format!("[worktree]\nbase = \"main\"\n{hook}"),
    )
    .unwrap();
}

fn app(sandbox: &Sandbox) -> AppCtx {
    AppCtx {
        paths: sandbox.paths(),
        config: AppConfig::default(),
        flags: Flags::default(),
    }
}

fn repo(sandbox: &Sandbox) -> RepoCtx {
    app(sandbox).repo_at(sandbox.work_dir()).unwrap()
}

/// A worker with a worktree and an event log, as `jig create` leaves one.
fn created(sandbox: &Sandbox) -> Worker {
    Worker::create(
        &repo(sandbox),
        &Branch::new(BRANCH),
        &Branch::new("main"),
        OnCreate::Skip,
    )
    .unwrap()
}

fn apply(
    sandbox: &Sandbox,
    worker: &Worker,
    cmd: Command,
    mux: &FakeMux,
) -> Result<(), CommandError> {
    let app = app(sandbox);
    commands::apply(&app.config, &repo(sandbox), worker, &cmd, mux)
}

/// The `Command` events in `worker`'s log, as `(command, detail)`.
fn recorded(worker: &Worker) -> Vec<(String, Option<String>)> {
    worker
        .event_log()
        .read_all()
        .unwrap()
        .into_iter()
        .filter_map(|e| match e.kind {
            EventKind::Command { command, detail } => Some((command, detail)),
            _ => None,
        })
        .collect()
}

fn status(worker: &Worker) -> WorkerStatus {
    let state: WorkerState = worker.event_log().reduce().unwrap();
    state.status
}

fn worktree(sandbox: &Sandbox) -> PathBuf {
    sandbox.work_dir().join(".jig").join(BRANCH)
}

fn key(sandbox: &Sandbox) -> WorkerKey {
    WorkerKey::new(repo(sandbox).name(), Branch::new(BRANCH))
}

// ── In-process ──────────────────────────────────────────────────────

#[test]
fn nudge_types_free_text_and_records_it() {
    let sandbox = configured(None);
    let worker = created(&sandbox);
    let mux = FakeMux::with_window(BRANCH);

    let text = "rebase on main, then {{carry on}}";
    apply(
        &sandbox,
        &worker,
        Command::Nudge { text: text.into() },
        &mux,
    )
    .unwrap();

    assert_eq!(mux.messages(), vec![(BRANCH.into(), text.into())]);
    assert_eq!(recorded(&worker), vec![("nudge".into(), Some(text.into()))]);
}

#[test]
fn nudge_needs_a_window() {
    let sandbox = configured(None);
    let worker = created(&sandbox);
    let mux = FakeMux::default();

    let err = apply(
        &sandbox,
        &worker,
        Command::Nudge { text: "hi".into() },
        &mux,
    )
    .unwrap_err();

    assert!(matches!(err, CommandError::NoWindow(_)), "{err}");
    assert!(
        recorded(&worker).is_empty(),
        "a refused nudge is not recorded"
    );
}

#[test]
fn model_types_the_slash_command_and_records_it() {
    let sandbox = configured(None);
    let worker = created(&sandbox);
    let mux = FakeMux::with_window(BRANCH);

    apply(
        &sandbox,
        &worker,
        Command::Model {
            model: "opus".into(),
        },
        &mux,
    )
    .unwrap();

    assert_eq!(mux.messages(), vec![(BRANCH.into(), "/model opus".into())]);
    assert_eq!(
        recorded(&worker),
        vec![("model".into(), Some("opus".into()))]
    );
    assert_eq!(
        status(&worker),
        WorkerStatus::Created,
        "a record, not a transition"
    );
}

#[test]
fn kill_closes_the_window_and_ends_the_log() {
    let sandbox = configured(None);
    let worker = created(&sandbox);
    let mux = FakeMux::with_window(BRANCH);

    apply(&sandbox, &worker, Command::Kill, &mux).unwrap();

    assert!(!mux.has(BRANCH));
    assert!(!worker.event_log().exists(), "kill forgets the worker");
    assert!(worktree(&sandbox).exists(), "but keeps its worktree");
}

#[test]
fn prune_removes_a_stopped_workers_worktree_and_log() {
    let sandbox = configured(None);
    let worker = created(&sandbox);
    std::fs::write(worktree(&sandbox).join("scratch.txt"), "dirty").unwrap();

    apply(&sandbox, &worker, Command::Prune, &FakeMux::default()).unwrap();

    assert!(!worktree(&sandbox).exists());
    assert!(!worker.event_log().exists());
}

#[test]
fn prune_refuses_a_running_worker() {
    let sandbox = configured(None);
    let worker = created(&sandbox);

    let err = apply(
        &sandbox,
        &worker,
        Command::Prune,
        &FakeMux::with_window(BRANCH),
    )
    .unwrap_err();

    assert!(matches!(err, CommandError::StillRunning(_)), "{err}");
    assert!(worktree(&sandbox).exists());
}

/// A worker whose `on_create` failed, as `jig spawn` leaves one.
fn failed_setup(sandbox: &Sandbox) -> Worker {
    configure(sandbox, Some("exit 3"));
    let err = Worker::create(
        &repo(sandbox),
        &Branch::new(BRANCH),
        &Branch::new("main"),
        OnCreate::Run,
    )
    .unwrap_err();
    assert!(err.to_string().contains("exited with status 3"), "{err}");
    let worker = Worker::open(&repo(sandbox), &Branch::new(BRANCH)).unwrap();
    assert_eq!(status(&worker), WorkerStatus::Failed);
    worker
}

#[test]
fn retry_setup_reruns_the_hook_and_starts_the_agent() {
    let sandbox = Sandbox::with_repo();
    let worker = failed_setup(&sandbox);
    configure(&sandbox, Some("echo ok > hook-ran"));
    let mux = FakeMux::default();

    apply(&sandbox, &worker, Command::RetrySetup, &mux).unwrap();

    assert!(worktree(&sandbox).join("hook-ran").exists());
    assert!(mux.has(BRANCH), "the agent gets a window");
    assert_eq!(mux.keys.lock().unwrap().len(), 1, "and is launched in it");
    assert_eq!(status(&worker), WorkerStatus::Spawned);
    assert_eq!(recorded(&worker), vec![("retry-setup".into(), None)]);
}

#[test]
fn a_retry_that_fails_again_stays_failed() {
    let sandbox = Sandbox::with_repo();
    let worker = failed_setup(&sandbox);
    let mux = FakeMux::default();

    let err = apply(&sandbox, &worker, Command::RetrySetup, &mux).unwrap_err();

    assert!(err.to_string().contains("exited with status 3"), "{err}");
    assert_eq!(status(&worker), WorkerStatus::Failed);
    assert!(!mux.has(BRANCH), "no agent without a working setup");
    assert_eq!(recorded(&worker), vec![("retry-setup".into(), None)]);
}

#[test]
fn retry_setup_refuses_a_worker_that_did_not_fail() {
    let sandbox = configured(None);
    let worker = created(&sandbox);

    let err = apply(&sandbox, &worker, Command::RetrySetup, &FakeMux::default()).unwrap_err();

    assert!(err.to_string().contains("not failed"), "{err}");
}

// ── Over the socket ─────────────────────────────────────────────────

/// Send `req` to the sandbox's daemon; `Ok(())` or the daemon's reason.
fn ask(sandbox: &Sandbox, req: Request) -> Result<(), String> {
    match ipc::request(&sandbox.paths(), &req) {
        Ok(Response::Ok) => Ok(()),
        Ok(other) => panic!("expected ok, got {other:?}"),
        Err(IpcError::Refused(reason)) => Err(reason),
        Err(e) => panic!("ipc failed: {e}"),
    }
}

#[test]
fn every_verb_is_answered_over_the_socket() {
    let sandbox = configured(None);
    register_globally(&sandbox.paths(), sandbox.work_dir());
    let worker = created(&sandbox);
    let _daemon = sandbox.start_daemon();
    let worker_key = key(&sandbox);

    // No agent is running, so the verbs that talk to one are refused — with
    // the daemon's reason, which is the point: it found the worker by key.
    let nudge = ask(
        &sandbox,
        Request::Nudge {
            worker: worker_key.clone(),
            text: "hi".into(),
        },
    );
    assert!(nudge.unwrap_err().contains("has no window"));
    let model = ask(
        &sandbox,
        Request::Model {
            worker: worker_key.clone(),
            model: "opus".into(),
        },
    );
    assert!(model.unwrap_err().contains("has no window"));
    let retry = ask(
        &sandbox,
        Request::RetrySetup {
            worker: worker_key.clone(),
        },
    );
    assert!(retry.unwrap_err().contains("not failed"));

    ask(
        &sandbox,
        Request::Kill {
            worker: worker_key.clone(),
        },
    )
    .unwrap();
    assert!(!worker.event_log().exists());

    ask(&sandbox, Request::Prune { worker: worker_key }).unwrap();
    assert!(!worktree(&sandbox).exists());
}

#[test]
fn an_unknown_worker_is_refused_with_a_reason() {
    let sandbox = configured(None);
    register_globally(&sandbox.paths(), sandbox.work_dir());
    let _daemon = sandbox.start_daemon();

    let err = ask(
        &sandbox,
        Request::Kill {
            worker: WorkerKey::new("no-such-repo", Branch::new("x")),
        },
    )
    .unwrap_err();
    assert!(
        err.contains("no tracked repo named 'no-such-repo'"),
        "{err}"
    );

    let err = ask(
        &sandbox,
        Request::Prune {
            worker: key(&sandbox),
        },
    )
    .unwrap_err();
    assert!(err.contains("not found"), "{err}");
}

#[test]
fn a_slow_command_does_not_block_status() {
    let sandbox = Sandbox::with_repo();
    register_globally(&sandbox.paths(), sandbox.work_dir());
    failed_setup(&sandbox);
    // Long enough to notice, short enough not to slow the suite.
    configure(&sandbox, Some("sleep 3; exit 1"));
    let _daemon = sandbox.start_daemon();

    let paths = sandbox.paths();
    let worker_key = key(&sandbox);
    let retry = std::thread::spawn(move || {
        ipc::request(&paths, &Request::RetrySetup { worker: worker_key })
    });
    // Let the retry reach the daemon and start its hook.
    std::thread::sleep(std::time::Duration::from_millis(500));

    let start = std::time::Instant::now();
    assert!(ipc::status(&sandbox.paths()).unwrap().is_some());
    assert!(
        start.elapsed() < std::time::Duration::from_secs(2),
        "status waited on the command"
    );
    // A second command for the same worker is refused while the first runs.
    let busy = ask(
        &sandbox,
        Request::RetrySetup {
            worker: key(&sandbox),
        },
    );
    assert!(busy.unwrap_err().contains("busy"));

    let result = retry.join().unwrap();
    assert!(matches!(result, Err(IpcError::Refused(r)) if r.contains("exited with status 1")));
}

// ── CLI ─────────────────────────────────────────────────────────────

#[test]
fn the_cli_runs_verbs_in_process_without_a_daemon() {
    let sandbox = configured(None);
    created(&sandbox);

    sandbox
        .jig()
        .args(["nudge", BRANCH, "keep", "going"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("has no window"));

    sandbox
        .jig()
        .args(["retry-setup", BRANCH])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not failed"));

    sandbox
        .jig()
        .args(["prune", BRANCH])
        .assert()
        .success()
        .stderr(predicate::str::contains("Pruned"));
    assert!(!worktree(&sandbox).exists());
}

#[test]
fn the_cli_works_with_a_running_daemon() {
    let sandbox = configured(None);
    // Running any command in the repo registers it, as a user would.
    sandbox.jig().args(["list"]).assert().success();
    let worker = created(&sandbox);
    let _daemon = sandbox.start_daemon();

    sandbox
        .jig()
        .args(["kill", BRANCH])
        .assert()
        .success()
        .stderr(predicate::str::contains("Killed"));
    assert!(!worker.event_log().exists());

    sandbox
        .jig()
        .args(["model", BRANCH, "opus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("has no window"));
}
