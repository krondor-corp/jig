//! Nudge command — type a message into a worker's agent session

use clap::Args;

use super::verb::{self, VerbError};
use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::{AppCtx, Ctx};
use crate::daemon::commands::Command;

/// Send a message to a worker's agent
#[derive(Args, Debug, Clone)]
pub struct Nudge {
    /// Branch name
    pub branch: String,

    /// What to say — any text, typed into the session as-is
    #[arg(required = true, trailing_var_arg = true)]
    pub text: Vec<String>,

    /// Operate on all tracked repos
    #[arg(short = 'g', long)]
    global: bool,
}

impl Op for Nudge {
    type Context = Ctx;
    type Error = VerbError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, VerbError> {
        Ok(Ctx::scoped(app, self.global)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        let text = self.text.join(" ");
        verb::run(&ctx, &self.branch, Command::Nudge { text })?;
        ui::success(&format!("Nudged '{}'", ui::highlight(&self.branch)));
        Ok(NoOutput)
    }
}
