//! Kill command - kill a running worker window

use clap::Args;

use super::verb::{self, VerbError};
use crate::context::{AppCtx, ContextError, Ctx, RepoCtx};
use crate::daemon::commands::{self, Command, CommandError};
use crate::worker::{Worker, WorkerError};

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;

/// Kill a running worker window
#[derive(Args, Debug, Clone)]
pub struct Kill {
    /// Branch name
    pub branch: Option<String>,

    /// Kill all workers
    #[arg(long, short)]
    pub all: bool,

    /// Operate on all tracked repos
    #[arg(short = 'g', long)]
    global: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum KillError {
    #[error(transparent)]
    Ctx(#[from] ContextError),
    #[error(transparent)]
    Worker(#[from] WorkerError),
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    Verb(#[from] VerbError),
    #[error("specify a branch or --all")]
    NoTarget,
}

impl Op for Kill {
    type Context = Ctx;
    type Error = KillError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, KillError> {
        Ok(Ctx::scoped(app, self.global)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        if self.all {
            let mut killed = 0;
            for repo in ctx.repos() {
                killed += kill_all_in_repo(&ctx, repo)?;
            }
            if killed == 0 {
                eprintln!("{}", ui::dim("No workers to kill."));
            }
            return Ok(NoOutput);
        }

        // One repo or every tracked one, the search is the same.
        let name = self.branch.as_deref().ok_or(KillError::NoTarget)?;
        verb::run(&ctx, name, Command::Kill)?;
        ui::success(&format!("Killed '{}'", ui::highlight(name)));
        Ok(NoOutput)
    }
}

fn kill_all_in_repo(ctx: &Ctx, repo: &RepoCtx) -> Result<usize, KillError> {
    let workers = Worker::discover(repo)?;
    for worker in &workers {
        commands::run(&ctx.paths, &ctx.config, repo, worker, Command::Kill)?;
        ui::success(&format!("Killed '{}'", ui::highlight(worker.branch())));
    }
    Ok(workers.len())
}
