//! `jig init` — scaffolding a repository.

use crate::common::Sandbox;
use predicates::prelude::*;
use std::fs;

#[test]
fn init_scaffolds_the_expected_files() {
    let repo = Sandbox::with_repo();

    repo.jig().args(["init", "claude"]).assert().success();

    let root = repo.work_dir();
    assert!(root.join("docs").is_dir());
    assert!(root.join("jig.toml").is_file());
    assert!(root.join(".claude").is_dir());
}

#[test]
fn init_is_idempotent_and_says_so() {
    let repo = Sandbox::with_repo();
    repo.jig().args(["init", "claude"]).assert().success();

    // A second run must not clobber the repo; it only re-checks the hooks.
    repo.jig()
        .args(["init", "claude"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Already initialized"));
}

#[test]
fn init_detects_an_existing_jig_toml() {
    let repo = Sandbox::with_repo();
    fs::write(repo.work_dir().join("jig.toml"), "[spawn]\nauto = true\n").unwrap();

    repo.jig()
        .args(["init", "claude"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Already initialized"));
}

#[test]
fn the_generated_jig_toml_has_no_agents_section() {
    let repo = Sandbox::with_repo();
    repo.jig().args(["init", "claude"]).assert().success();

    let content = fs::read_to_string(repo.work_dir().join("jig.toml")).unwrap();
    assert!(
        !content.contains("[agents]"),
        "jig.toml should not carry an [agents] section: {content}"
    );
}
