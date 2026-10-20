//! The `[worktree] on_create` hook, and `jig create --no-hooks`.

use crate::common::Sandbox;
use predicates::prelude::*;

#[test]
fn hook_round_trips_through_config() {
    let repo = Sandbox::with_local_repo();

    repo.jig()
        .args(["config", "on-create", "echo hello"])
        .assert()
        .success();

    repo.jig()
        .args(["config", "on-create"])
        .assert()
        .success()
        .stdout(predicate::str::contains("echo hello"));
}

#[test]
fn hook_appears_in_config_show() {
    let repo = Sandbox::with_local_repo();
    repo.jig()
        .args(["config", "on-create", "echo hello"])
        .assert()
        .success();

    repo.jig()
        .args(["config"])
        .assert()
        .success()
        .stderr(predicate::str::contains("echo hello"));
}

#[test]
fn hook_runs_in_the_new_worktree() {
    let repo = Sandbox::with_local_repo();
    repo.jig()
        .args(["config", "on-create", "touch hook_ran.txt"])
        .assert()
        .success();

    repo.jig().args(["create", "hook-test"]).assert().success();

    assert!(repo.worktree("hook-test").join("hook_ran.txt").exists());
}

#[test]
fn no_hooks_skips_the_hook() {
    let repo = Sandbox::with_local_repo();
    repo.jig()
        .args(["config", "on-create", "touch hook_ran.txt"])
        .assert()
        .success();

    repo.jig()
        .args(["create", "no-hook-test", "--no-hooks"])
        .assert()
        .success();

    assert!(
        repo.worktree("no-hook-test").exists(),
        "--no-hooks should still create the worktree"
    );
    assert!(
        !repo.worktree("no-hook-test").join("hook_ran.txt").exists(),
        "--no-hooks should suppress the on_create hook"
    );
}

#[test]
fn hook_can_be_unset() {
    let repo = Sandbox::with_local_repo();
    repo.jig()
        .args(["config", "on-create", "echo hello"])
        .assert()
        .success();

    repo.jig()
        .args(["config", "on-create", "--unset"])
        .assert()
        .success();

    repo.jig()
        .args(["config", "on-create"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No on-create hook"));
}

#[test]
fn a_failing_hook_still_leaves_a_worktree() {
    // The worktree is the thing the user asked for; a broken hook is their
    // script's problem and must not roll the creation back.
    let repo = Sandbox::with_local_repo();
    repo.jig()
        .args(["config", "on-create", "exit 1"])
        .assert()
        .success();

    let _ = repo.jig().args(["create", "fail-hook-test"]).output();

    assert!(repo.worktree("fail-hook-test").exists());
}
