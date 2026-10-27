//! Model command — switch a worker's agent to another model

use clap::Args;

use super::verb::{self, VerbError};
use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::{AppCtx, Ctx};
use crate::daemon::commands::Command;

/// Switch a running worker's agent to another model
#[derive(Args, Debug, Clone)]
pub struct Model {
    /// Branch name
    pub branch: String,

    /// Model to switch to, as the agent's /model command takes it
    pub model: String,

    /// Operate on all tracked repos
    #[arg(short = 'g', long)]
    global: bool,
}

impl Op for Model {
    type Context = Ctx;
    type Error = VerbError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, VerbError> {
        Ok(Ctx::scoped(app, self.global)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        let model = self.model.clone();
        verb::run(&ctx, &self.branch, Command::Model { model })?;
        ui::success(&format!(
            "Switched '{}' to {}",
            ui::highlight(&self.branch),
            ui::bold(&self.model)
        ));
        Ok(NoOutput)
    }
}
