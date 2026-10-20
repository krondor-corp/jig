//! `jig exit` — leave the current worktree and remove it.

use crate::common::Sandbox;
use predicates::prelude::*;
use std::fs;

#[test]
fn exit_from_the_base_repo_errors() {
    let repo = Sandbox::with_local_repo();

    repo.jig()
        .args(["exit"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in a worktree"));
}

#[test]
fn exit_removes_only_the_current_worktree() {
    let repo = Sandbox::with_local_repo();
    repo.jig().args(["create", "exit-test1"]).assert().success();
    repo.jig().args(["create", "exit-test2"]).assert().success();

    repo.jig_in(&repo.worktree("exit-test1"))
        .args(["exit"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cd "));

    assert!(!repo.worktree("exit-test1").exists());
    assert!(repo.worktree("exit-test2").exists());
}

#[test]
fn exit_refuses_to_discard_uncommitted_work() {
    let repo = Sandbox::with_local_repo();
    repo.jig()
        .args(["create", "exit-dirty-test"])
        .assert()
        .success();

    let wt = repo.worktree("exit-dirty-test");
    fs::write(wt.join("testfile.txt"), "uncommitted change").unwrap();

    repo.jig_in(&wt)
        .args(["exit"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("uncommitted changes"));

    assert!(wt.exists(), "a refused exit must not remove the worktree");
}

#[test]
fn exit_force_discards_uncommitted_work() {
    let repo = Sandbox::with_local_repo();
    repo.jig()
        .args(["create", "exit-force-test"])
        .assert()
        .success();

    let wt = repo.worktree("exit-force-test");
    fs::write(wt.join("testfile.txt"), "uncommitted change").unwrap();

    repo.jig_in(&wt)
        .args(["exit", "--force"])
        .assert()
        .success();

    assert!(!wt.exists());
}

#[test]
fn exit_works_from_a_nested_worktree() {
    let repo = Sandbox::with_local_repo();
    repo.jig()
        .args(["create", "feature/nested/test"])
        .assert()
        .success();

    let wt = repo.worktree("feature/nested/test");
    repo.jig_in(&wt).args(["exit"]).assert().success();

    assert!(!wt.exists());
}
