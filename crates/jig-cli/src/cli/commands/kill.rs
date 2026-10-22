//! Kill command - kill a running worker window

use clap::Args;

use crate::context::{AppCtx, ContextError, Ctx, RepoCtx};
use crate::worker::{Worker, WorkerError};
use jig_core::git::Branch;
use jig_core::mux::{self, MuxKind};

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
    #[error("specify a branch or --all")]
    NoTarget,
    #[error("{0}")]
    NotFound(String),
}

impl Op for Kill {
    type Context = Ctx;
    type Error = KillError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, KillError> {
        Ok(Ctx::scoped(app, self.global)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        let kind = ctx.config.mux;

        if self.all {
            let mut killed = 0;
            for repo in ctx.repos() {
                killed += kill_all_in_repo(repo, kind)?;
            }
            if killed == 0 {
                eprintln!("{}", ui::dim("No workers to kill."));
            }
            return Ok(NoOutput);
        }

        // One repo or every tracked one, the search is the same.
        let name = self.branch.as_deref().ok_or(KillError::NoTarget)?;
        let branch = Branch::new(name);
        for repo in ctx.repos() {
            let worker = match Worker::open(repo, &branch) {
                Ok(worker) => worker,
                Err(WorkerError::NotFound(_)) => continue,
                Err(e) => return Err(e.into()),
            };
            let _ = worker.kill(&mux::for_repo(kind, &repo.name()));
            worker.unregister()?;
            ui::success(&format!("Killed '{}'", ui::highlight(name)));
            return Ok(NoOutput);
        }
        Err(KillError::NotFound(format!("worker '{}' not found", name)))
    }
}

fn kill_all_in_repo(repo: &RepoCtx, kind: MuxKind) -> Result<usize, KillError> {
    let mux = mux::for_repo(kind, &repo.name());
    let workers = Worker::discover(repo)?;
    for worker in &workers {
        let _ = worker.kill(&mux);
        worker.unregister()?;
        ui::success(&format!("Killed '{}'", ui::highlight(worker.branch())));
    }
    Ok(workers.len())
}
