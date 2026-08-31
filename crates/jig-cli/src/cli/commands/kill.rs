//! Kill command - kill a running worker window

use clap::Args;

use crate::context::{RepoConfig, ScopedCtx};
use crate::worker::Worker;

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
    Context(#[from] crate::context::ContextError),
    #[error(transparent)]
    Worker(#[from] crate::worker::WorkerError),
    #[error("specify a branch or --all")]
    NoTarget,
    #[error("{0}")]
    NotFound(String),
}

impl Op for Kill {
    type Context = ScopedCtx;
    type Error = KillError;
    type Output = NoOutput;

    fn build_context(&self) -> Result<ScopedCtx, KillError> {
        Ok(ScopedCtx::from_global(self.global)?)
    }

    fn run(&self, ctx: ScopedCtx) -> Result<Self::Output, Self::Error> {
        match ctx {
            ScopedCtx::Global(g) => {
                if self.all {
                    let mut killed = 0;
                    for repo in &g.repos {
                        killed += kill_all_in_repo(repo, g.config.mux)?;
                    }
                    if killed == 0 {
                        eprintln!("{}", ui::dim("No workers to kill."));
                    }
                    return Ok(NoOutput);
                }

                let name = self.branch.as_deref().ok_or(KillError::NoTarget)?;
                for repo in &g.repos {
                    let git_repo = jig_core::git::Repo::open(&repo.repo_root).unwrap();
                    let repo_name = repo.name();
                    let mux = jig_core::mux::for_repo(g.config.mux, &repo_name);
                    let workers = Worker::discover(&git_repo);
                    if let Some(worker) = workers.iter().find(|w| w.branch() == name) {
                        let _ = worker.kill(&mux);
                        worker.unregister()?;
                        ui::success(&format!("Killed '{}'", ui::highlight(name)));
                        return Ok(NoOutput);
                    }
                }
                Err(KillError::NotFound(format!("worker '{}' not found", name)))
            }
            ScopedCtx::Repo(r) => {
                let repo_name = r.repo.name();
                let mux = jig_core::mux::for_repo(r.config.mux, &repo_name);

                if self.all {
                    let killed = kill_all_in_repo(&r.repo, r.config.mux)?;
                    if killed == 0 {
                        eprintln!("{}", ui::dim("No workers to kill."));
                    }
                    return Ok(NoOutput);
                }

                let name = self.branch.as_deref().ok_or(KillError::NoTarget)?;
                let workers =
                    Worker::discover(&jig_core::git::Repo::open(&r.repo.repo_root).unwrap());
                let worker = workers
                    .iter()
                    .find(|w| w.branch() == name)
                    .ok_or_else(|| KillError::NotFound(format!("worker '{}' not found", name)))?;
                let _ = worker.kill(&mux);
                worker.unregister()?;
                ui::success(&format!("Killed '{}'", ui::highlight(name)));
                Ok(NoOutput)
            }
        }
    }
}

fn kill_all_in_repo(repo: &RepoConfig, kind: jig_core::mux::MuxKind) -> Result<usize, KillError> {
    let repo_name = repo.name();
    let mux = jig_core::mux::for_repo(kind, &repo_name);
    let workers = Worker::discover(&jig_core::git::Repo::open(&repo.repo_root).unwrap());
    for worker in &workers {
        let _ = worker.kill(&mux);
        worker.unregister()?;
        ui::success(&format!("Killed '{}'", ui::highlight(worker.branch())));
    }
    Ok(workers.len())
}
