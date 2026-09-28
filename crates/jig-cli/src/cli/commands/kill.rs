//! Kill command - kill a running worker window

use clap::Args;

use crate::context::{AppCtx, Ctx, RepoCtx};
use crate::worker::Worker;

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::AppPaths;

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
    Ctx(#[from] crate::context::ContextError),
    #[error(transparent)]
    Worker(#[from] crate::worker::WorkerError),
    #[error(transparent)]
    Git(#[from] jig_core::git::GitError),
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
        let paths = ctx.paths.clone();
        let kind = ctx.config.mux;

        if self.all {
            let mut killed = 0;
            for repo in ctx.repos() {
                killed += kill_all_in_repo(&paths, repo, kind)?;
            }
            if killed == 0 {
                eprintln!("{}", ui::dim("No workers to kill."));
            }
            return Ok(NoOutput);
        }

        // One repo or every tracked one, the search is the same.
        let name = self.branch.as_deref().ok_or(KillError::NoTarget)?;
        for repo in ctx.repos() {
            let git_repo = jig_core::git::Repo::open(&repo.paths.repo_root)?;
            let mux = jig_core::mux::for_repo(kind, &repo.name());
            let workers = Worker::discover(&git_repo);
            if let Some(worker) = workers.iter().find(|w| w.branch() == name) {
                let _ = worker.kill(&mux);
                worker.unregister(&paths)?;
                ui::success(&format!("Killed '{}'", ui::highlight(name)));
                return Ok(NoOutput);
            }
        }
        Err(KillError::NotFound(format!("worker '{}' not found", name)))
    }
}

fn kill_all_in_repo(
    paths: &AppPaths,
    repo: &RepoCtx,
    kind: jig_core::mux::MuxKind,
) -> Result<usize, KillError> {
    let repo_name = repo.name();
    let mux = jig_core::mux::for_repo(kind, &repo_name);
    let workers = Worker::discover(&jig_core::git::Repo::open(&repo.paths.repo_root)?);
    for worker in &workers {
        let _ = worker.kill(&mux);
        worker.unregister(paths)?;
        ui::success(&format!("Killed '{}'", ui::highlight(worker.branch())));
    }
    Ok(workers.len())
}
