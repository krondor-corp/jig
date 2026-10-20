use std::fmt;

use clap::Args;

use jig_core::issues::IssueStatus;

use crate::cli::op::Op;
use crate::context::{AppCtx, Ctx, RepoCtx};

/// Mark an issue as complete
#[derive(Args, Debug, Clone)]
pub struct Complete {
    /// Issue ID (e.g. "features/my-feature")
    pub id: String,
}

#[derive(Debug)]
pub struct CompleteOutput {
    pub id: String,
}

impl fmt::Display for CompleteOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Completed: {}", self.id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompleteError {
    #[error(transparent)]
    Ctx(#[from] crate::context::ContextError),
    #[error(transparent)]
    Linear(#[from] jig_core::issues::providers::linear::client::LinearError),
}

fn run(
    repo: &RepoCtx,
    global: &crate::context::AppConfig,
    cmd: &Complete,
) -> Result<CompleteOutput, CompleteError> {
    let linear_provider = repo.linear_provider(global)?;
    linear_provider.update_status(&cmd.id, &IssueStatus::Complete)?;

    Ok(CompleteOutput { id: cmd.id.clone() })
}

impl Op for Complete {
    type Context = Ctx;
    type Error = CompleteError;
    type Output = CompleteOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, CompleteError> {
        Ok(Ctx::here(app)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        run(ctx.repo()?, &ctx.config, self)
    }
}
