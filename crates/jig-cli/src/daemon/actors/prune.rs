//! Prune actor — removes worktrees for merged/closed PRs in a background thread.

use std::path::PathBuf;

use jig_core::git::{Repo, Worktree};

use super::Actor;
use crate::context::log::Logs;
use crate::context::AppPaths;

pub struct PruneTarget {
    pub repo_path: PathBuf,
    pub repo_name: String,
    pub worker_name: String,
}

pub struct PruneRequest {
    pub paths: AppPaths,
    pub targets: Vec<PruneTarget>,
}

#[derive(Default)]
pub struct PruneActor;

impl Actor for PruneActor {
    type Request = PruneRequest;
    type Response = ();

    const NAME: &'static str = "jig-prune";
    const QUEUE_SIZE: usize = 1;

    fn handle(&self, req: PruneRequest) {
        for target in &req.targets {
            let key = format!("{}/{}", target.repo_name, target.worker_name);
            match prune_single(&req.paths, target) {
                Ok(()) => tracing::info!(worker = %key, "pruned worktree"),
                Err(msg) => tracing::warn!(worker = %key, "prune failed: {}", msg),
            }
        }
    }
}

/// Remove one worker's worktree and logs — for the actor, and for
/// `jig prune` through [`crate::daemon::commands`].
pub(crate) fn prune_single(
    paths: &AppPaths,
    target: &PruneTarget,
) -> std::result::Result<(), String> {
    let worktree_path = crate::context::worktree_path(&target.repo_path, &target.worker_name);

    if worktree_path.exists() {
        let wt = Worktree::open(&worktree_path)
            .map_err(|e| format!("failed to open worktree: {}", e))?;
        // The PR is merged or closed, so whatever is left in the tree is
        // not work worth keeping — build output, scratch files, an edited
        // lockfile. A non-forced remove refuses on any of it, and a fleet
        // of agents leaves some behind in nearly every worktree, so the
        // daemon would quietly stop pruning. Force, and say what went.
        if wt.has_uncommitted_changes().unwrap_or(false) {
            tracing::warn!(
                worker = %target.worker_name,
                "discarding uncommitted changes in pruned worktree"
            );
        }
        wt.remove(true)
            .map_err(|e| format!("git worktree prune failed: {}", e))?;
    } else {
        let repo =
            Repo::open(&target.repo_path).map_err(|e| format!("failed to open repo: {}", e))?;
        repo.prune_stale_worktrees();
    }

    // The worker's event log: left in place, a respawn of the same branch
    // would wake up already `Merged`.
    let _ = Logs::new(paths)
        .worker_events(&target.repo_name, &target.worker_name)
        .remove();

    {
        let events_dir = paths.events_dir();
        let sanitized = format!(
            "{}-{}",
            target.repo_name,
            target.worker_name.replace('/', "-")
        );
        let event_dir = events_dir.join(&sanitized);
        if event_dir.is_dir() {
            let _ = std::fs::remove_dir_all(&event_dir);
        }
    }

    Ok(())
}
