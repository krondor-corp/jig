//! The git hook handlers that `jig hooks <name>` runs.

use jig_cli::hooks::handlers::*;

use crate::common::Sandbox;

#[test]
fn pre_commit_is_noop() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(handle_pre_commit(tmp.path()).is_ok());
}

#[test]
fn commit_msg_valid() {
    let tmp = tempfile::tempdir().unwrap();
    let msg_file = tmp.path().join("COMMIT_EDITMSG");
    std::fs::write(&msg_file, "feat: add new feature\n").unwrap();
    assert!(handle_commit_msg(tmp.path(), msg_file.to_str().unwrap()).is_ok());
}

#[test]
fn commit_msg_invalid() {
    let tmp = tempfile::tempdir().unwrap();
    let msg_file = tmp.path().join("COMMIT_EDITMSG");
    std::fs::write(&msg_file, "not a conventional commit\n").unwrap();
    assert!(handle_commit_msg(tmp.path(), msg_file.to_str().unwrap()).is_err());
}

#[test]
fn commit_msg_strips_comments() {
    let tmp = tempfile::tempdir().unwrap();
    let msg_file = tmp.path().join("COMMIT_EDITMSG");
    std::fs::write(
        &msg_file,
        "fix: resolve bug\n# This is a comment\n# Another comment\n",
    )
    .unwrap();
    assert!(handle_commit_msg(tmp.path(), msg_file.to_str().unwrap()).is_ok());
}

#[test]
fn commit_msg_empty_after_stripping_comments() {
    let tmp = tempfile::tempdir().unwrap();
    let msg_file = tmp.path().join("COMMIT_EDITMSG");
    std::fs::write(&msg_file, "# All comments\n# Nothing else\n").unwrap();
    assert!(handle_commit_msg(tmp.path(), msg_file.to_str().unwrap()).is_ok());
}

#[test]
fn post_commit_outside_worktree_is_noop() {
    let sandbox = Sandbox::new();
    let tmp = tempfile::tempdir().unwrap();
    assert!(handle_post_commit(&sandbox.paths(), tmp.path()).is_ok());
}

#[test]
fn post_merge_outside_worktree_is_noop() {
    let sandbox = Sandbox::new();
    let tmp = tempfile::tempdir().unwrap();
    assert!(handle_post_merge(&sandbox.paths(), tmp.path()).is_ok());
}
