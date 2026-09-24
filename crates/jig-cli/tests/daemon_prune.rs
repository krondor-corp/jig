//! Pruning a worker's worktree and event log.

mod common;

use common::Sandbox;
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
