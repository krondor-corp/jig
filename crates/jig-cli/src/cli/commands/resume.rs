//! Resume command — relaunch a dead worker's agent session

use clap::Args;

use crate::worker::Worker;
use jig_core::agents;
use jig_core::Worktree;

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
    Ctx(#[from] crate::context::ContextError),
    #[error(transparent)]
    Worker(#[from] crate::worker::WorkerError),
    #[error(transparent)]
    Git(#[from] jig_core::GitError),
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
        let repo_name = ctx.repo()?.name();
        let repo = ctx.repo()?;
        let mux = jig_core::mux::for_repo(ctx.config.mux, &repo_name);

        // Open existing worktree
        let wt_path = repo.paths.worktrees_path.join(&self.branch);
        if !wt_path.exists() {
            return Err(ResumeError::Usage(format!(
                "worktree '{}' not found",
                self.branch
            )));
        }
        let wt = Worktree::open(&wt_path)?;

        // Error if mux window already exists
        let pre = Worker::from(&wt);
        if pre.has_mux_window(&mux) {
            ui::failure(&format!(
                "Worker '{}' already has a window. Use '{}' to attach.",
                ui::highlight(&self.branch),
                ui::highlight(&format!("jig attach {}", self.branch))
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

        let agent = agents::Agent::from_config(
            &repo.config.agent.agent_type,
            repo.config.agent.model.as_deref(),
            &repo.config.agent.disallowed_tools,
        )
        .unwrap_or_else(|| agents::Agent::from_config("claude", None, &[]).unwrap());

        let prompt = crate::prompts::resume_task(&effective_context);
        Worker::resume(&ctx.paths, &wt, &agent, prompt, &mux)?;

        ui::success(&format!("Resumed worker '{}'", ui::highlight(&self.branch)));

        eprintln!();
        eprintln!(
            "  Use '{}' to attach",
            ui::highlight(&format!("jig attach {}", self.branch))
        );
        eprintln!("  Use '{}' to check status", ui::highlight("jig ps"));

        Ok(NoOutput)
    }
}
