//! Home command - navigate to base repo root

use clap::Args;
use std::path::PathBuf;

use crate::cli::op::Op;
use crate::context::RepoCtx;

/// Go to base repository root
#[derive(Args, Debug, Clone)]
pub struct Home;

#[derive(Debug)]
pub struct HomeOutput(PathBuf);

impl std::fmt::Display for HomeOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cd '{}'", self.0.display())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HomeError {
    #[error(transparent)]
    Context(#[from] crate::context::ContextError),
}

impl Op for Home {
    type Context = RepoCtx;
    type Error = HomeError;
    type Output = HomeOutput;

    fn build_context(&self) -> Result<RepoCtx, HomeError> {
        Ok(RepoCtx::from_cwd()?)
    }

    fn run(&self, ctx: RepoCtx) -> Result<Self::Output, Self::Error> {
        Ok(HomeOutput(ctx.repo.repo_root.clone()))
    }
}
