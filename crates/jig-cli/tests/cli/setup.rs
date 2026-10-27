//! Worktree setup: copied files and the `on_create` hook.

use crate::common::Sandbox;
use predicates::prelude::*;

/// Base worktrees on the local `main` — sandbox repos have no `origin` —
/// plus whatever else the test configures.
fn configure(sandbox: &Sandbox, worktree: &str) {
    std::fs::write(
        sandbox.work_dir().join("jig.toml"),
        format!("[worktree]\nbase = \"main\"\n{worktree}"),
    )
    .unwrap();
}

#[test]
fn setup_copies_files_and_runs_the_hook() {
    let sandbox = Sandbox::with_repo();
    let repo = sandbox.work_dir();
    std::fs::write(repo.join(".env"), "SECRET=1\n").unwrap();
    configure(
        &sandbox,
        "copy = [\".env\"]\non_create = \"echo ran > hook-ran\"\n",
    );

    sandbox.jig().args(["create", "feat"]).assert().success();

    let wt = repo.join(".jig/feat");
    assert_eq!(
        std::fs::read_to_string(wt.join(".env")).unwrap(),
        "SECRET=1\n"
    );
    assert!(wt.join("hook-ran").exists());
}

fn repo_name(sandbox: &Sandbox) -> String {
    let name = sandbox.work_dir().file_name().unwrap();
    name.to_string_lossy().to_string()
}

#[test]
fn a_failing_hook_logs_both_streams_and_quotes_the_end() {
    let sandbox = Sandbox::with_repo();
    configure(
        &sandbox,
        "on_create = \"echo on-stdout; echo on-stderr >&2; exit 7\"\n",
    );
    let paths = sandbox.paths();
    let log = paths.worker_setup_log(&repo_name(&sandbox), "feat");

    sandbox
        .jig()
        .args(["create", "feat"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("on_create hook exited with status 7:")
                .and(predicate::str::contains("  on-stdout"))
                .and(predicate::str::contains("  on-stderr"))
                .and(predicate::str::contains(format!(
                    "(full log: {})",
                    log.display()
                ))),
        );

    assert_eq!(
        std::fs::read_to_string(&log).unwrap(),
        "on-stdout\non-stderr\n"
    );
}

#[test]
fn a_passing_setup_clears_an_old_log_when_there_is_no_hook() {
    let sandbox = Sandbox::with_repo();
    configure(&sandbox, "");
    let paths = sandbox.paths();
    let log = paths.worker_setup_log(&repo_name(&sandbox), "feat");
    std::fs::create_dir_all(log.parent().unwrap()).unwrap();
    std::fs::write(&log, "from an earlier spawn\n").unwrap();

    sandbox.jig().args(["create", "feat"]).assert().success();
    assert!(!log.exists());
}

/// `jig create` goes through the same `Worker::create` as spawn, so a failed
/// setup is recorded where `jig list` and `jig attach` look for it.
#[test]
fn a_failed_create_records_why_in_the_event_log() {
    let sandbox = Sandbox::with_repo();
    configure(&sandbox, "on_create = \"echo boom; exit 3\"\n");
    let events = sandbox
        .paths()
        .worker_events_log(&repo_name(&sandbox), "feat");

    sandbox.jig().args(["create", "feat"]).assert().failure();

    let log = std::fs::read_to_string(&events).unwrap();
    assert!(log.contains(r#""terminal":"failed""#), "{log}");
    assert!(log.contains("exited with status 3"), "{log}");

    sandbox
        .jig()
        .args(["ls"])
        .assert()
        .success()
        .stderr(predicate::str::contains("setup failed"));
}

/// The failure is structured, not just a string: the event log records the
/// phase, exit status and output tail, and `ps` renders them.
#[test]
fn a_failing_hook_is_a_structured_setup_failure_ps_shows() {
    let sandbox = Sandbox::with_repo();
    configure(&sandbox, "on_create = \"echo boom; exit 3\"\n");
    let events = sandbox
        .paths()
        .worker_events_log(&repo_name(&sandbox), "feat");

    sandbox.jig().args(["create", "feat"]).assert().failure();

    let log = std::fs::read_to_string(&events).unwrap();
    let finished: Vec<serde_json::Value> = log
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|e: &serde_json::Value| e["type"] == "setup_finished")
        .collect();
    let phases: Vec<&str> = finished
        .iter()
        .map(|e| e["phase"].as_str().unwrap())
        .collect();
    assert_eq!(phases, ["worktree", "copy_files", "on_create"], "{log}");
    let on_create = &finished[2];
    assert_eq!(on_create["ok"], false);
    assert_eq!(on_create["exit_code"], 3);
    assert_eq!(on_create["tail"], serde_json::json!(["boom"]));

    sandbox
        .jig()
        .args(["ps"])
        .assert()
        .success()
        .stderr(predicate::str::contains("failed: on_create (exit 3)"));
}

#[test]
fn resuming_a_worker_with_no_worktree_says_not_found() {
    let sandbox = Sandbox::with_repo();
    configure(&sandbox, "");

    sandbox
        .jig()
        .args(["resume", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("worker 'nope' not found"));
}
