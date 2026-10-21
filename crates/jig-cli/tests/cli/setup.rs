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
