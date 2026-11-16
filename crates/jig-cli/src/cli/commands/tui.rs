//! Tui command — jig's terminal control hub. Plain `jig` runs it too.

use clap::Args;

use crate::cli::op::{LogSink, NoOutput, Op};
use crate::cli::ui;
use crate::context::{AppCtx, Ctx};

/// Open jig's TUI: every worker on this machine and the fleet (also: plain `jig`)
#[derive(Args, Debug, Clone, Default)]
pub struct Tui;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error(transparent)]
    Ctx(#[from] crate::context::ContextError),
    #[error("terminal: {0}")]
    Terminal(#[from] std::io::Error),
}

impl Op for Tui {
    type Context = Ctx;
    type Error = TuiError;
    type Output = NoOutput;

    /// Always global: the hub shows every tracked repo.
    fn build_context(&self, app: AppCtx) -> Result<Ctx, TuiError> {
        Ok(Ctx::everywhere(app)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        let no_color =
            ui::is_plain() || std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        crate::tui::run(ctx, no_color)?;
        Ok(NoOutput)
    }

    /// It owns the terminal: a warning on stderr would tear the screen.
    fn log_sink(&self) -> LogSink {
        LogSink::File
    }
}
