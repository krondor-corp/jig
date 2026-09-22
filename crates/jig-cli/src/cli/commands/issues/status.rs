use std::fmt;

use clap::Args;

use jig_core::issues::IssueStatus;

use crate::cli::op::Op;
use crate::context::{RepoConfig, RepoCtx};

/// Update issue status
#[derive(Args, Debug, Clone)]
pub struct Status {
    /// Issue ID (e.g. "features/my-feature")
    pub id: String,

    /// New status (triage, backlog, planned, in-progress, complete, blocked)
    #[arg(short, long)]
    pub status: String,
}

#[derive(Debug)]
pub struct StatusOutput {
    pub id: String,
    pub status: String,
}

impl fmt::Display for StatusOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Updated {} -> {}", self.id, self.status)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StatusError {
    #[error(transparent)]
    Context(#[from] crate::context::ContextError),
    #[error(transparent)]
    Linear(#[from] jig_core::issues::providers::linear::client::LinearError),
    #[error("{0}")]
    Usage(String),
}

fn run(
    repo: &RepoConfig,
    global: &crate::context::Config,
    cmd: &Status,
) -> Result<StatusOutput, StatusError> {
    let status: IssueStatus = cmd
        .status
        .parse()
        .map_err(|_| StatusError::Usage(format!("unknown status: {}", cmd.status)))?;

    let linear_provider = repo.linear_provider(global)?;
    linear_provider.update_status(&cmd.id, &status)?;

    Ok(StatusOutput {
        id: cmd.id.clone(),
        status: status.to_string(),
    })
}

impl Op for Status {
    type Context = RepoCtx;
    type Error = StatusError;
    type Output = StatusOutput;

    fn build_context(&self) -> Result<RepoCtx, StatusError> {
        Ok(RepoCtx::from_cwd()?)
    }

    fn run(&self, ctx: RepoCtx) -> Result<Self::Output, Self::Error> {
        run(&ctx.repo, &ctx.config, self)
    }
}
