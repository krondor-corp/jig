//! Home command - navigate to base repo root

use clap::Args;
use std::path::PathBuf;

use crate::cli::op::Op;
use crate::context::AppCtx;
use crate::context::Ctx;

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
    Ctx(#[from] crate::context::ContextError),
}

impl Op for Home {
    type Context = Ctx;
    type Error = HomeError;
    type Output = HomeOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, HomeError> {
        Ok(Ctx::here(app)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        Ok(HomeOutput(ctx.repo()?.paths.repo_root.clone()))
    }
}
