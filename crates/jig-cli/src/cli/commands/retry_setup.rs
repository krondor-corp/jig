//! Retry-setup command — re-run a failed worker's setup and start its agent

use clap::Args;

use super::verb::{self, VerbError};
use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::{AppCtx, Ctx};
use crate::daemon::commands::Command;

/// Re-run on_create for a worker whose setup failed, then start its agent
#[derive(Args, Debug, Clone)]
pub struct RetrySetup {
    /// Branch name
    pub branch: String,

    /// Operate on all tracked repos
    #[arg(short = 'g', long)]
    global: bool,
}

impl Op for RetrySetup {
    type Context = Ctx;
    type Error = VerbError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, VerbError> {
        Ok(Ctx::scoped(app, self.global)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        ui::progress(&format!(
            "Retrying setup for '{}'",
            ui::highlight(&self.branch)
        ));
        verb::run(&ctx, &self.branch, Command::RetrySetup)?;
        ui::success(&format!(
            "Setup succeeded — started '{}'",
            ui::highlight(&self.branch)
        ));
        Ok(NoOutput)
    }
}
