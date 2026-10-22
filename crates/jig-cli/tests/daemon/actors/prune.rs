//! Pruning a worker's worktree and event log.

use crate::common::Sandbox;
use jig_cli::daemon::actors::prune::{PruneActor, PruneRequest, PruneTarget};
use jig_cli::daemon::actors::Actor;

/// A target with nothing behind it: no worktree on disk, no event log.
/// Pruning must report failure through its log, not panic.
#[test]
fn pruning_a_worker_that_is_already_gone_is_harmless() {
    let mut sandbox = Sandbox::new();
    let repo = sandbox.add_repo();

    PruneActor.handle(PruneRequest {
        paths: sandbox.paths(),
        targets: vec![PruneTarget {
            repo_path: repo,
            repo_name: "repo".to_string(),
            worker_name: "nonexistent-worker".to_string(),
        }],
    });
}

/// A merged worker's worktree is rarely clean: agents leave build output,
/// edited lockfiles and scratch files behind. The PR is merged, so none of
/// it is work worth keeping — prune must remove the worktree anyway, and
/// drop the worker's event log with it.
#[test]
fn pruning_removes_a_dirty_worktree_and_its_event_log() {
    let mut sandbox = Sandbox::new();
    let repo_path = sandbox.add_repo();
    let repo = jig_core::git::Repo::open(&repo_path).unwrap();
    let worktree = repo
        .create_worktree(&"feature/done".into(), &"main".into())
        .unwrap();
    std::fs::write(worktree.join("scratch.txt"), "left behind").unwrap();

    let paths = sandbox.paths();
    let log = paths.worker_events_log("repo", "feature/done");
    std::fs::create_dir_all(log.parent().unwrap()).unwrap();
    std::fs::write(&log, "{}\n").unwrap();

    PruneActor.handle(PruneRequest {
        paths: paths.clone(),
        targets: vec![PruneTarget {
            repo_path: repo_path.clone(),
            repo_name: "repo".to_string(),
            worker_name: "feature/done".to_string(),
        }],
    });

    assert!(!worktree.exists(), "dirty worktree should be pruned");
    assert!(!log.exists(), "event log should go with the worktree");
}
