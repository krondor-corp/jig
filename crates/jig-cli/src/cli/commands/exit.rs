//! Exit worktree command

use clap::Args;
use std::path::PathBuf;

use jig_core::Worktree;

use crate::cli::op::Op;
use crate::cli::ui;
use crate::context::RepoCtx;

/// Exit current worktree and remove it
#[derive(Args, Debug, Clone)]
pub struct Exit {
    /// Force removal even with uncommitted changes
    #[arg(long, short)]
    pub force: bool,
}

/// Output containing cd command to base repo
#[derive(Debug)]
pub struct ExitOutput(PathBuf);

impl std::fmt::Display for ExitOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cd '{}'", self.0.display())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExitError {
    #[error(transparent)]
    Context(#[from] crate::context::ContextError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Git(#[from] jig_core::GitError),
}

impl Op for Exit {
    type Context = RepoCtx;
    type Error = ExitError;
    type Output = ExitOutput;

    fn build_context(&self) -> Result<RepoCtx, ExitError> {
        Ok(RepoCtx::from_cwd()?)
    }

    fn run(&self, ctx: RepoCtx) -> Result<Self::Output, Self::Error> {
        let wt = Worktree::current()?;
        let name = wt.branch_name();

        wt.remove(self.force)?;

        ui::success(&format!("Exited worktree '{}'", ui::highlight(&name)));

        let canonical = ctx.repo.repo_root.canonicalize()?;
        Ok(ExitOutput(canonical))
    }
}
