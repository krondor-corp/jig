//! Resume command — relaunch a dead worker's agent session

use clap::Args;

use crate::context::ContextError;
use crate::prompts::resume_task;
use crate::worker::{Worker, WorkerError};
use jig_core::git::Branch;
use jig_core::mux;

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::AppCtx;
use crate::context::Ctx;

/// Resume a dead worker by relaunching its agent session
#[derive(Args, Debug, Clone)]
pub struct Resume {
    /// Branch name to resume
    pub branch: String,

    /// Override the task context for the resumed session
    #[arg(long, short)]
    pub context: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ResumeError {
    #[error(transparent)]
    Ctx(#[from] ContextError),
    #[error(transparent)]
    Worker(#[from] WorkerError),
    #[error("{0}")]
    Usage(String),
}

impl Op for Resume {
    type Context = Ctx;
    type Error = ResumeError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, ResumeError> {
        Ok(Ctx::here(app)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        let repo = ctx.repo()?;
        let mux = mux::for_repo(ctx.config.mux, &repo.name());
        let worker = Worker::open(repo, &Branch::new(&self.branch))?;

        // Error if mux window already exists
        if worker.has_mux_window(&mux) {
            ui::failure(&format!(
                "Worker '{}' already has a window. Use '{}' to attach.",
                ui::highlight(&self.branch),
                ui::highlight(format!("jig attach {}", self.branch))
            ));
            return Err(ResumeError::Usage(format!(
                "Worker '{}' already running — use `jig attach` instead",
                self.branch
            )));
        }

        let effective_context = self
            .context
            .clone()
            .unwrap_or_else(|| "You were interrupted. Resume your previous task.".to_string());

        worker.resume(repo, resume_task(&effective_context), &mux)?;

        ui::success(&format!("Resumed worker '{}'", ui::highlight(&self.branch)));

        eprintln!();
        eprintln!(
            "  Use '{}' to attach",
            ui::highlight(format!("jig attach {}", self.branch))
        );
        eprintln!("  Use '{}' to check status", ui::highlight("jig ps"));

        Ok(NoOutput)
    }
}
