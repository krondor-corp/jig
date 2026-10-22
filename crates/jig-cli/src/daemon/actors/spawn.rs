//! Spawn actor — polls for spawnable issues and launches workers in a
//! background thread.

use std::sync::Mutex;

use crate::context::{worktree_path, RepoCtx};
use crate::prompts::spawn_task;
use jig_core::git::{Branch, Repo};
use jig_core::issues::issue::{IssueFilter, IssueStatus};
use jig_core::issues::{Issue, IssueProvider};
use jig_core::mux::{self, Mux};

type Worker = crate::worker::Worker;

use super::Actor;
use crate::daemon::TickContext;

pub struct SpawnRequest {
    pub ctx: TickContext,
}

#[derive(Default)]
pub struct SpawnActor {
    spawning_workers: Mutex<Vec<Branch>>,
}

impl SpawnActor {
    pub fn spawning_workers(&self) -> Vec<Branch> {
        self.spawning_workers.lock().unwrap().clone()
    }
}

impl Actor for SpawnActor {
    type Request = SpawnRequest;
    type Response = ();

    const NAME: &'static str = "jig-spawn";
    const QUEUE_SIZE: usize = 1;

    fn handle(&self, req: SpawnRequest) {
        let mut spawning = Vec::new();
        let global = &req.ctx.app.config;

        for entry in req.ctx.repos.iter() {
            let repo_root = entry.path.clone();
            let repo_name = repo_root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let repo = match Repo::open(&repo_root) {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!(repo = %repo_name, error = %e, "failed to open repo");
                    continue;
                }
            };

            let cfg = match req.ctx.app.repo_at(&repo_root) {
                Ok(cfg) => cfg,
                Err(e) => {
                    tracing::debug!(repo = %repo_name, error = %e, "failed to load config");
                    continue;
                }
            };

            let provider = match cfg.issue_provider(global) {
                Ok(p) => p,
                Err(e) => {
                    tracing::debug!(repo = %repo_name, error = %e, "failed to create issue provider");
                    continue;
                }
            };

            // Branches that already have a worktree — an issue whose branch
            // is checked out somewhere is already being worked on.
            let existing_branches: Vec<Branch> = repo
                .list_worktrees()
                .unwrap_or_default()
                .iter()
                .filter_map(|wt| wt.branch().ok())
                .collect();

            // -- Auto-spawn --
            let Some(labels) = &cfg.config.issues.auto_spawn_labels else {
                continue;
            };

            let planned = provider
                .list(&IssueFilter {
                    status: Some(IssueStatus::Planned),
                    ..Default::default()
                })
                .unwrap_or_default();

            for issue in planned {
                if !issue.children().is_empty() {
                    continue;
                }
                if !(labels.is_empty() || issue.auto(labels)) {
                    continue;
                }
                if !provider.may_spawn(issue.id()) {
                    continue;
                }
                if existing_branches.iter().any(|b| b == issue.branch()) {
                    continue;
                }

                let branch = issue.branch().clone();
                spawning.push(branch.clone());

                let mux = mux::for_repo(global.mux, &repo_name);
                match spawn_worker_for_issue(&req.ctx, &issue, &cfg, &provider, &mux) {
                    Ok(_worker) => {
                        tracing::info!(worker = %branch, "auto-spawned worker");
                    }
                    Err(msg) => {
                        tracing::warn!(worker = %branch, "auto-spawn failed: {}", msg);
                    }
                }
                spawning.retain(|b| b != &branch);
            }
        }

        *self.spawning_workers.lock().unwrap() = spawning;
    }
}

fn spawn_worker_for_issue(
    ctx: &TickContext,
    issue: &Issue,
    cfg: &RepoCtx,
    provider: &IssueProvider,
    mux: &dyn Mux,
) -> std::result::Result<Worker, String> {
    let branch = issue.branch();
    if worktree_path(&cfg.paths.repo_root, branch).exists() {
        tracing::debug!(worker = %branch, "worktree already exists, skipping");
        return Ok(Worker::from_branch(&cfg.paths.repo_root, branch.clone()));
    }

    let worker = Worker::spawn(
        &ctx.app.paths,
        cfg,
        branch,
        &cfg.base_branch(&ctx.app.config),
        spawn_task(issue, provider),
        true,
        Some(issue.id().clone()),
        mux,
    )
    .map_err(|e| e.to_string())?;

    if let Err(e) = provider.update_status(issue.id(), &IssueStatus::InProgress) {
        tracing::warn!(
            issue = %issue.id(),
            error = %e,
            "failed to update issue status to InProgress"
        );
    }

    Ok(worker)
}
