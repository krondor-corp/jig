//! Git hooks recording into a worker's event log.

use crate::common::Sandbox;

/// Git runs hooks from the worktree, whose path is not the repo root the
/// command's context points at. The event must land in that worker's log.
#[test]
fn post_commit_in_a_worktree_records_a_commit() {
    let sandbox = Sandbox::with_repo();
    let repo = sandbox.work_dir();
    std::fs::write(repo.join("jig.toml"), "[worktree]\nbase = \"main\"\n").unwrap();
    sandbox.jig().args(["create", "feat"]).assert().success();

    let worktree = repo.join(".jig/feat");
    sandbox
        .jig_in(&worktree)
        .args(["hooks", "post-commit"])
        .assert()
        .success();

    let repo_name = repo.file_name().unwrap().to_string_lossy().to_string();
    let events = sandbox.paths().worker_events_log(&repo_name, "feat");
    let log = std::fs::read_to_string(&events).unwrap();
    assert!(log.contains(r#""type":"commit""#), "{log}");
}
