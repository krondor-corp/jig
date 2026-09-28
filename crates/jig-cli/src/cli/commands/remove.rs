//! Remove worktree command

use clap::Args;
use glob::Pattern;

use crate::context::{AppCtx, Ctx, RepoCtx};
use jig_core::git::Repo;
use jig_core::Worktree;

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;

/// Remove worktree(s)
#[derive(Args, Debug, Clone)]
pub struct Remove {
    /// Worktree name or glob pattern
    pub pattern: String,

    /// Force removal even with uncommitted changes
    #[arg(long, short)]
    pub force: bool,

    /// Operate on all tracked repos
    #[arg(short = 'g', long)]
    global: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveError {
    #[error(transparent)]
    Ctx(#[from] crate::context::ContextError),
    #[error("{0}")]
    NotFound(String),
    #[error("Invalid pattern: {0}")]
    InvalidPattern(#[from] glob::PatternError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Git(#[from] jig_core::GitError),
}

impl Op for Remove {
    type Context = Ctx;
    type Error = RemoveError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, RemoveError> {
        Ok(Ctx::scoped(app, self.global)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        // One repo or every tracked one: remove from the first that has a
        // match, and report only if none did.
        for repo in ctx.repos() {
            match self.remove_from_repo(repo) {
                Err(RemoveError::NotFound(_)) => continue,
                result => return result,
            }
        }
        Err(RemoveError::NotFound(format!(
            "no worktrees matching '{}'",
            self.pattern
        )))
    }
}

impl Remove {
    fn remove_from_repo(&self, repo: &RepoCtx) -> Result<NoOutput, RemoveError> {
        let git_repo = Repo::open(&repo.paths.repo_root)?;
        let worktrees = git_repo.list_worktrees()?;
        let names: Vec<String> = worktrees
            .iter()
            .map(|wt| wt.branch_name().to_string())
            .collect();

        // Find matching worktrees
        let pattern = Pattern::new(&self.pattern)?;

        let matching: Vec<_> = names
            .iter()
            .filter(|name| pattern.matches(name.as_str()) || name.as_str() == pattern.as_str())
            .cloned()
            .collect();

        if matching.is_empty() {
            // If not a pattern match, try exact match
            let exact_path = repo.paths.worktrees_path.join(pattern.as_str());
            if exact_path.exists() {
                Worktree::open(&exact_path)?.remove(self.force)?;
                ui::success(&format!(
                    "Removed worktree '{}'",
                    ui::highlight(pattern.as_str())
                ));
                return Ok(NoOutput);
            }
            return Err(RemoveError::NotFound(format!(
                "no worktrees matching '{}'",
                pattern.as_str()
            )));
        }

        // Remove each matching worktree
        for name in matching {
            let path = repo.paths.worktrees_path.join(&name);
            Worktree::open(&path)?.remove(self.force)?;
            ui::success(&format!("Removed worktree '{}'", ui::highlight(&name)));
        }

        Ok(NoOutput)
    }
}
