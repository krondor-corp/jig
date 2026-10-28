//! What the control-verb commands share: find the named worker in the
//! command's repos, then run the verb through the daemon if one is
//! listening, in-process otherwise — see [`crate::daemon::commands`].

use crate::context::{ContextError, Ctx};
use crate::daemon::commands::{self, Command, CommandError, Via};
use crate::worker::{Worker, WorkerError};
use jig_core::git::Branch;

#[derive(Debug, thiserror::Error)]
pub enum VerbError {
    #[error(transparent)]
    Ctx(#[from] ContextError),
    #[error(transparent)]
    Worker(#[from] WorkerError),
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error("worker '{0}' not found")]
    NotFound(String),
}

/// Run `cmd` on the worker for branch `name`, searching the context's repos
/// in order — the current one, or every tracked one under `-g`.
pub fn run(ctx: &Ctx, name: &str, cmd: Command) -> Result<Via, VerbError> {
    let branch = Branch::new(name);
    for repo in ctx.repos() {
        let worker = match Worker::open(repo, &branch) {
            Ok(worker) => worker,
            Err(WorkerError::NotFound(_)) => continue,
            Err(e) => return Err(e.into()),
        };
        return Ok(commands::run(&ctx.paths, &ctx.config, repo, &worker, cmd)?);
    }
    Err(VerbError::NotFound(name.to_string()))
}
