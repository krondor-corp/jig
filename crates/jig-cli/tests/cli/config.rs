//! `jig config` get/set/unset, and repo-over-global precedence.

use crate::common::Sandbox;
use predicates::prelude::*;

#[test]
fn show_reports_the_default_base() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["config"])
        .assert()
        .success()
        .stderr(predicate::str::contains("origin/main"));
}

#[test]
fn base_round_trips_for_the_repo() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["config", "base", "origin/develop"])
        .assert()
        .success();

    repo.jig()
        .args(["config", "base"])
        .assert()
        .success()
        .stdout(predicate::str::contains("origin/develop"));
}

#[test]
fn unsetting_base_falls_back_to_the_default() {
    let repo = Sandbox::with_repo();
    repo.jig()
        .args(["config", "base", "origin/develop"])
        .assert()
        .success();

    repo.jig()
        .args(["config", "base", "--unset"])
        .assert()
        .success();

    repo.jig()
        .args(["config", "base"])
        .assert()
        .success()
        .stdout(predicate::str::contains("origin/main"))
        .stdout(predicate::str::contains("origin/develop").not());
}

#[test]
fn base_round_trips_globally() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["config", "base", "--global", "origin/master"])
        .assert()
        .success();

    repo.jig()
        .args(["config", "base", "--global"])
        .assert()
        .success()
        .stdout(predicate::str::contains("origin/master"));
}

#[test]
fn show_falls_back_to_the_global_base() {
    let repo = Sandbox::with_repo();
    repo.jig()
        .args(["config", "base", "--global", "origin/master"])
        .assert()
        .success();

    repo.jig()
        .args(["config"])
        .assert()
        .success()
        .stderr(predicate::str::contains("origin/master"));
}

#[test]
fn a_repo_base_wins_over_the_global_one() {
    let repo = Sandbox::with_repo();
    repo.jig()
        .args(["config", "base", "--global", "origin/master"])
        .assert()
        .success();
    repo.jig()
        .args(["config", "base", "origin/feature"])
        .assert()
        .success();

    repo.jig()
        .args(["config"])
        .assert()
        .success()
        .stderr(predicate::str::contains("origin/feature"));
}

#[test]
fn list_shows_the_global_configuration() {
    let repo = Sandbox::with_repo();
    repo.jig()
        .args(["config", "base", "--global", "origin/master"])
        .assert()
        .success();
    // A repo-level base must not leak into the global listing.
    repo.jig()
        .args(["config", "base", "origin/feature"])
        .assert()
        .success();

    repo.jig()
        .args(["config", "--list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("origin/master"))
        .stderr(predicate::str::contains("origin/feature").not());
}
