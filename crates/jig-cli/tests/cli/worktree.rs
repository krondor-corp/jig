//! Creating, listing, opening and removing worktrees.

use crate::common::Sandbox;
use predicates::prelude::*;

#[test]
fn create_makes_a_linked_worktree() {
    let repo = Sandbox::with_local_repo();

    repo.jig()
        .args(["create", "test1"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Created worktree"));

    assert!(repo.worktree("test1").exists());
    // A linked worktree's `.git` is a file pointing at the main repo, not a dir.
    assert!(repo.worktree("test1").join(".git").is_file());
}

#[test]
fn list_shows_a_created_worktree() {
    let repo = Sandbox::with_local_repo();
    repo.jig().args(["create", "test1"]).assert().success();

    repo.jig()
        .args(["list", "--plain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test1"));
}

#[test]
fn create_with_o_outputs_cd() {
    let repo = Sandbox::with_local_repo();

    repo.jig()
        .args(["create", "test2", "-o"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cd "));
}

#[test]
fn open_outputs_cd_to_the_worktree() {
    let repo = Sandbox::with_local_repo();
    repo.jig().args(["create", "test1"]).assert().success();

    repo.jig()
        .args(["open", "test1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cd "))
        .stdout(predicate::str::contains("test1"));
}

#[test]
fn remove_deletes_the_worktree() {
    let repo = Sandbox::with_local_repo();
    repo.jig().args(["create", "test1"]).assert().success();
    assert!(repo.worktree("test1").exists());

    repo.jig()
        .args(["remove", "test1"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Removed worktree"));

    assert!(!repo.worktree("test1").exists());
}

#[test]
fn list_omits_a_removed_worktree() {
    let repo = Sandbox::with_local_repo();
    repo.jig().args(["create", "test1"]).assert().success();
    repo.jig().args(["remove", "test1"]).assert().success();

    // --plain, so the absence is asserted against the machine-readable
    // listing rather than against an empty stdout, which would pass anyway.
    repo.jig()
        .args(["list", "--plain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test1").not());
}

#[test]
fn create_keeps_errors_out_of_stdout() {
    // stdout carries the `cd` that the shell wrapper evals, so an error on it
    // — colored or not — would be executed. See docs/STDOUT-FORMATTING.md.
    let repo = Sandbox::with_local_repo();
    repo.jig().args(["create", "errortest"]).assert().success();

    let output = repo
        .jig()
        .args(["create", "errortest", "-o"])
        .output()
        .expect("run jig create");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains('\x1b'),
        "stdout carries color codes: {stdout:?}"
    );
}

#[test]
fn create_accepts_a_nested_branch_name() {
    let repo = Sandbox::with_local_repo();

    repo.jig()
        .args(["create", "feature/test/nested"])
        .assert()
        .success();

    assert!(repo.worktree("feature/test/nested").exists());
}

#[test]
fn list_shows_a_nested_worktree() {
    let repo = Sandbox::with_local_repo();
    repo.jig()
        .args(["create", "feature/test/nested"])
        .assert()
        .success();

    repo.jig()
        .args(["list", "--plain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feature/test/nested"));
}

#[test]
fn remove_accepts_a_glob() {
    let repo = Sandbox::with_local_repo();
    repo.jig().args(["create", "glob-test1"]).assert().success();
    repo.jig().args(["create", "glob-test2"]).assert().success();

    repo.jig().args(["remove", "glob-test*"]).assert().success();

    repo.jig()
        .args(["list", "--plain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("glob-test").not());
}
