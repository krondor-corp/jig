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

#[test]
fn a_failing_hook_names_itself_and_its_status() {
    let sandbox = Sandbox::with_repo();
    configure(&sandbox, "on_create = \"exit 7\"\n");

    sandbox
        .jig()
        .args(["create", "feat"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("on_create hook failed").and(predicate::str::contains('7')),
        );
}
