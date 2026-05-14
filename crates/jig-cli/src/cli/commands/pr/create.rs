//! Create subcommand — push current branch and create a draft PR

use std::fmt;

use clap::Args;

use crate::cli::op::Op;
use crate::cli::ui;
use crate::context::{RepoConfig, RepoCtx};
use crate::worker::events::{self, WorkerState};
use jig_core::git::{Branch, Repo};
use jig_core::github::GitHubClient;
use jig_core::Worktree;

/// Push current branch and create a draft PR
#[derive(Args, Debug, Clone)]
pub struct Create {
    /// PR title (defaults to --fill behavior)
    #[arg(short, long)]
    pub title: Option<String>,

    /// PR body/description
    #[arg(short, long)]
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct CreateOutput(pub String);

impl fmt::Display for CreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error(transparent)]
    Context(#[from] crate::context::ContextError),
    #[error(transparent)]
    Git(#[from] jig_core::GitError),
    #[error(transparent)]
    GitHub(#[from] jig_core::github::GitHubError),
    #[error(transparent)]
    Linear(#[from] jig_core::issues::providers::linear::client::LinearError),
    #[error("could not determine current branch")]
    NoBranch,
}

impl Op for Create {
    type Context = RepoCtx;
    type Error = CreateError;
    type Output = CreateOutput;

    fn build_context(&self) -> Result<RepoCtx, CreateError> {
        Ok(RepoCtx::from_cwd()?)
    }

    fn run(&self, ctx: RepoCtx) -> Result<Self::Output, Self::Error> {
        let git_repo = Repo::discover()?;
        let branch = git_repo
            .current_branch()
            .map_err(|_| CreateError::NoBranch)?;

        let base = resolve_base(&ctx.repo, &ctx.config)?;
        let base_str: &str = &base;
        let base_for_gh = base_str.strip_prefix("origin/").unwrap_or(base_str);

        ui::detail(&format!(
            "Base: {} → {}",
            ui::highlight(&branch.to_string()),
            ui::highlight(base_for_gh)
        ));

        ui::detail("Pushing...");
        git_repo.push_branch(&branch)?;

        let branch_str = branch.to_string();
        let gh = GitHubClient::from_remote()?;
        let url = gh.create_pr(
            base_for_gh,
            Some(&branch_str),
            self.title.as_deref(),
            self.body.as_deref(),
        )?;

        ui::success(&format!("Draft PR created: {}", ui::highlight(&url)));

        Ok(CreateOutput(url))
    }
}

fn resolve_base(repo: &RepoConfig, global: &crate::context::Config) -> Result<Branch, CreateError> {
    let worktree_name = match Worktree::current() {
        Ok(wt) => wt.branch_name(),
        Err(_) => return Ok(repo.base_branch(global)),
    };

    let repo_name = repo.name();

    let issue_ref = match events::event_log_for_worker(&repo_name, &worktree_name) {
        Ok(log) => {
            let state: WorkerState = match log.reduce() {
                Ok(s) => s,
                Err(_) => return Ok(repo.base_branch(global)),
            };
            match state.issue_ref {
                Some(r) => r,
                None => return Ok(repo.base_branch(global)),
            }
        }
        Err(_) => return Ok(repo.base_branch(global)),
    };

    let provider = repo.issue_provider(global)?;
    let issue = match provider.get(&issue_ref)? {
        Some(i) => i,
        None => return Ok(repo.base_branch(global)),
    };

    if let Some(parent_ref) = &issue.parent() {
        if let Ok(Some(parent_issue)) = provider.get(parent_ref) {
            return Ok(parent_issue.branch().clone());
        }
    }

    Ok(repo.base_branch(global))
}
