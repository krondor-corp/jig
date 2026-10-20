//! The small, always-available commands, and the help surface.

use crate::common::Sandbox;
use predicates::prelude::*;

/// Commands removed from jig that the help must not advertise again.
///
/// The hand-maintained help screen used to list `review`, `merge` and
/// `repos` long after they were deleted, and nothing caught it because the
/// only suite that checked lived at the workspace root, where cargo never
/// compiled it. Both help paths are now clap-generated; this is the guard.
const REMOVED_COMMANDS: [&str; 3] = ["review", "merge", "repos"];

#[test]
fn help_lists_the_registered_commands() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("spawn"))
        .stdout(predicate::str::contains("ps"))
        .stdout(predicate::str::contains("attach"))
        .stdout(predicate::str::contains("commit"))
        .stdout(predicate::str::contains("pr"))
        .stdout(predicate::str::contains("resume"));
}

#[test]
fn help_does_not_advertise_removed_commands() {
    let repo = Sandbox::with_repo();
    let output = repo.jig().args(["--help"]).output().expect("jig --help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    for name in REMOVED_COMMANDS {
        assert!(
            !stdout.contains(&format!("  {name} ")),
            "--help still advertises the removed command {name}: {stdout}"
        );
    }
}

#[test]
fn bare_jig_prints_the_same_help_as_the_help_flag() {
    // The two used to disagree: `jig` printed a hand-written screen while
    // `jig --help` printed clap's. Both now come from the same command tree.
    let repo = Sandbox::with_repo();

    let bare = repo.jig().output().expect("bare jig");
    let flag = repo.jig().args(["--help"]).output().expect("jig --help");

    assert!(bare.status.success(), "bare jig should exit 0");
    assert_eq!(
        String::from_utf8_lossy(&bare.stdout).trim(),
        String::from_utf8_lossy(&flag.stdout).trim(),
    );
}

#[test]
fn removed_commands_are_not_runnable() {
    let repo = Sandbox::with_repo();

    for name in REMOVED_COMMANDS {
        repo.jig()
            .args([name])
            .assert()
            .failure()
            .stderr(predicate::str::contains("unrecognized subcommand"));
    }
}

#[test]
fn version_reports_the_binary_name() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["version"])
        .assert()
        .success()
        .stderr(predicate::str::contains("jig"));
}

#[test]
fn which_reports_the_executable_path() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["which"])
        .assert()
        .success()
        .stdout(predicate::str::contains("jig"));
}

#[test]
fn ps_reports_an_empty_session_list() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["ps"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No spawned sessions"));
}

#[test]
fn health_reports_its_sections() {
    // Health exits non-zero when a dependency is missing (`gh auth` in CI),
    // so assert on the report, not the status.
    let repo = Sandbox::with_repo();
    let output = repo.jig().args(["health"]).output().expect("jig health");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("System"), "no System section: {stderr}");
    assert!(
        stderr.contains("Repository"),
        "no Repository section: {stderr}"
    );
    assert!(stderr.contains("Agent"), "no Agent section: {stderr}");
}

#[test]
fn spawn_requires_a_branch_or_issue() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["spawn"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("branch name required"));
}

#[test]
fn kill_requires_a_target() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["kill"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("specify a branch or --all"));
}

#[test]
fn resume_requires_a_branch() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["resume"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn resume_rejects_an_unknown_worktree() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["resume", "FAKE-123"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn home_outputs_cd_to_the_repo_root() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["home"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cd "));
}

#[test]
fn home_has_an_h_alias() {
    let repo = Sandbox::with_repo();

    repo.jig()
        .args(["h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cd "));
}

#[test]
fn home_from_a_worktree_points_at_the_base_repo() {
    let repo = Sandbox::with_local_repo();
    repo.jig().args(["create", "home-test"]).assert().success();

    let output = repo
        .jig_in(&repo.worktree("home-test"))
        .args(["home"])
        .output()
        .expect("jig home");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cd "));
    assert!(
        !stdout.contains("home-test"),
        "home should leave the worktree, not stay in it: {stdout}"
    );
}

#[test]
fn home_outside_a_git_repo_errors() {
    // A sandbox with no repos runs in a scratch dir that is not a git repo.
    let sandbox = Sandbox::new();

    sandbox
        .jig()
        .args(["home"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in a git repository"));
}
