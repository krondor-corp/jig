//! Integration tests for `jig commit` commands.

use jig_cli::test_support::Sandbox;
use predicates::prelude::*;
use std::fs;

#[test]
fn validate_valid_conventional_commit() {
    let repo = Sandbox::with_repo();
    repo.commit("feat: add user authentication");

    repo.jig()
        .args(["commit", "validate"])
        .assert()
        .success()
        .stderr(predicate::str::contains("valid conventional commit: feat"));
}

#[test]
fn validate_valid_scoped_commit() {
    let repo = Sandbox::with_repo();
    repo.commit("fix(ui): correct button alignment");

    repo.jig()
        .args(["commit", "validate"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "valid conventional commit: fix(ui)",
        ));
}

#[test]
fn validate_valid_breaking_commit() {
    let repo = Sandbox::with_repo();
    repo.commit("feat!: remove legacy API");

    repo.jig()
        .args(["commit", "validate"])
        .assert()
        .success()
        .stderr(predicate::str::contains("feat!"));
}

#[test]
fn validate_unknown_type() {
    let repo = Sandbox::with_repo();
    repo.commit("invalid: something");

    repo.jig()
        .args(["commit", "validate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown commit type 'invalid'"));
}

#[test]
fn validate_non_conventional_commit() {
    let repo = Sandbox::with_repo();
    repo.commit("just a regular commit message");

    repo.jig()
        .args(["commit", "validate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid commit format"));
}

#[test]
fn validate_stdin() {
    let repo = Sandbox::with_repo();
    repo.commit("init"); // need at least one commit for repo to work

    repo.jig()
        .args(["commit", "validate", "--stdin"])
        .write_stdin("feat: add feature")
        .assert()
        .success()
        .stderr(predicate::str::contains("valid conventional commit"));
}

#[test]
fn validate_stdin_invalid() {
    let repo = Sandbox::with_repo();
    repo.commit("init");

    repo.jig()
        .args(["commit", "validate", "--stdin"])
        .write_stdin("bad message")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid commit format"));
}

#[test]
fn validate_file() {
    let repo = Sandbox::with_repo();
    repo.commit("init");

    let msg_file = repo.work_dir().join("COMMIT_MSG");
    fs::write(&msg_file, "fix: resolve crash").unwrap();

    repo.jig()
        .args(["commit", "validate", "--file", msg_file.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("valid conventional commit: fix"));
}

#[test]
fn validate_multiple_errors() {
    let repo = Sandbox::with_repo();
    repo.commit("init");

    // Use a known type so parsing succeeds, then validation catches multiple issues
    let long_subject = "A".to_string() + &"a".repeat(80);
    let msg = format!("feat: {}", long_subject);

    repo.jig()
        .args(["commit", "validate", "--stdin"])
        .write_stdin(msg)
        .assert()
        .failure()
        .stderr(predicate::str::contains("subject too long"))
        .stderr(predicate::str::contains(
            "subject should start with lowercase",
        ));
}

#[test]
fn examples_command() {
    let repo = Sandbox::with_repo();
    repo.commit("init");

    repo.jig()
        .args(["commit", "examples"])
        .assert()
        .success()
        .stderr(predicate::str::contains("feat"))
        .stderr(predicate::str::contains("fix"))
        .stderr(predicate::str::contains("Breaking changes"));
}

#[test]
fn validate_specific_rev() {
    let repo = Sandbox::with_repo();
    repo.commit("feat: first feature");
    repo.commit("bad commit message");

    // Validate HEAD (the bad one) should fail
    repo.jig()
        .args(["commit", "validate", "HEAD"])
        .assert()
        .failure();

    // Validate HEAD~1 (the good one) should pass
    repo.jig()
        .args(["commit", "validate", "HEAD~1"])
        .assert()
        .success()
        .stderr(predicate::str::contains("valid conventional commit: feat"));
}
