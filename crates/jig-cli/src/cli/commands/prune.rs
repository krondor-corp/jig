//! Prune command — remove a stopped worker's worktree and logs

use clap::Args;

use super::verb::{self, VerbError};
use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::{AppCtx, Ctx};
use crate::daemon::commands::Command;

/// Remove a stopped worker's worktree and logs, uncommitted changes included
#[derive(Args, Debug, Clone)]
pub struct Prune {
    /// Branch name
    pub branch: String,

    /// Operate on all tracked repos
    #[arg(short = 'g', long)]
    global: bool,
}

impl Op for Prune {
    type Context = Ctx;
    type Error = VerbError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, VerbError> {
        Ok(Ctx::scoped(app, self.global)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        verb::run(&ctx, &self.branch, Command::Prune)?;
        ui::success(&format!("Pruned '{}'", ui::highlight(&self.branch)));
        Ok(NoOutput)
    }
}
