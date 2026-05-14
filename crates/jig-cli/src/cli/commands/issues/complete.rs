use std::fmt;

use clap::Args;

use jig_core::issues::IssueStatus;

use crate::cli::op::Op;
use crate::context::{RepoConfig, RepoCtx};

/// Mark an issue as complete
#[derive(Args, Debug, Clone)]
pub struct Complete {
    /// Issue ID (e.g. "features/my-feature")
    pub id: String,

    /// Delete the issue file after marking complete
    #[arg(long)]
    pub delete: bool,
}

#[derive(Debug)]
pub struct CompleteOutput {
    pub id: String,
    pub deleted: bool,
}

impl fmt::Display for CompleteOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.deleted {
            write!(f, "Completed and deleted: {}", self.id)
        } else {
            write!(f, "Completed: {}", self.id)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompleteError {
    #[error(transparent)]
    Context(#[from] crate::context::ContextError),
    #[error(transparent)]
    Linear(#[from] jig_core::issues::providers::linear::client::LinearError),
}

fn run(
    repo: &RepoConfig,
    global: &crate::context::Config,
    cmd: &Complete,
) -> Result<CompleteOutput, CompleteError> {
    let linear_provider = repo.linear_provider(global)?;
    linear_provider.update_status(&cmd.id, &IssueStatus::Complete)?;

    Ok(CompleteOutput {
        id: cmd.id.clone(),
        deleted: cmd.delete,
    })
}

impl Op for Complete {
    type Context = RepoCtx;
    type Error = CompleteError;
    type Output = CompleteOutput;

    fn build_context(&self) -> Result<RepoCtx, CompleteError> {
        Ok(RepoCtx::from_cwd()?)
    }

    fn run(&self, ctx: RepoCtx) -> Result<Self::Output, Self::Error> {
        run(&ctx.repo, &ctx.config, self)
    }
}
