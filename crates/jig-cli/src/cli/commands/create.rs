//! Create worktree command

use clap::Args;
use std::path::PathBuf;

use crate::worker::events::{self, Event, EventKind};
use jig_core::git::Branch;
use jig_core::Worktree;

use crate::cli::op::Op;
use crate::cli::ui;
use crate::context::RepoCtx;

/// Create a new worktree
#[derive(Args, Debug, Clone)]
pub struct Create {
    /// Branch name
    pub branch: String,

    /// Open/cd into worktree after creating
    #[arg(short = 'o')]
    pub open: bool,

    /// Base branch to create worktree from (overrides jig.toml default)
    #[arg(long, short = 'b')]
    pub base: Option<String>,

    /// Skip on-create hook execution
    #[arg(long = "no-hooks")]
    pub no_hooks: bool,
}

/// Output containing optional cd command
#[derive(Debug)]
pub enum CreateOutput {
    /// No output (created without -o flag)
    None,
    /// cd command to stdout
    Cd(PathBuf),
}

impl std::fmt::Display for CreateOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateOutput::None => Ok(()),
            CreateOutput::Cd(path) => write!(f, "cd '{}'", path.display()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error(transparent)]
    Context(#[from] crate::context::ContextError),
    #[error(transparent)]
    Git(#[from] jig_core::GitError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("configured base branch `{base}` requires a remote named `{remote}`, but no such remote is configured in this repository\n  help: add a remote (`git remote add {remote} <url>`) or set `[worktree] base = \"main\"` in jig.toml")]
    MissingRemote { remote: String, base: String },
}

impl Op for Create {
    type Context = RepoCtx;
    type Error = CreateError;
    type Output = CreateOutput;

    fn build_context(&self) -> Result<RepoCtx, CreateError> {
        Ok(RepoCtx::from_cwd()?)
    }

    fn run(&self, ctx: RepoCtx) -> Result<Self::Output, Self::Error> {
        let repo = &ctx.repo;
        let base_branch = match &self.base {
            Some(b) => Branch::new(b),
            None => repo.base_branch(&ctx.config),
        };

        let git_repo = jig_core::Repo::open(&repo.repo_root)?;

        if let Some(remote) = base_branch.remote_prefix() {
            if !git_repo.has_remote(remote) {
                return Err(CreateError::MissingRemote {
                    remote: remote.to_string(),
                    base: base_branch.to_string(),
                });
            }
        }

        let branch = Branch::new(&self.branch);
        let copy_files: Vec<std::path::PathBuf> = repo
            .repo
            .worktree
            .copy
            .iter()
            .map(std::path::PathBuf::from)
            .collect();
        let on_create = repo.repo.worktree.on_create.as_ref().map(|cmd| {
            let mut c = std::process::Command::new("sh");
            c.args(["-c", cmd]);
            c
        });
        let wt = Worktree::create(&git_repo, &branch, &base_branch, &copy_files, on_create)?;

        let repo_name = repo.name();
        if let Ok(event_log) = events::event_log_for_worker(&repo_name, &self.branch) {
            if let Err(e) = event_log.append(&Event::now(EventKind::Create {
                branch: branch.to_string(),
            })) {
                tracing::warn!(branch = %self.branch, error = %e, "failed to emit Create event");
            }
        }

        ui::success(&format!(
            "Created worktree on branch '{}'",
            ui::highlight(&branch)
        ));

        // Output cd command if -o flag
        if self.open {
            let canonical = wt.path().canonicalize()?;
            Ok(CreateOutput::Cd(canonical))
        } else {
            Ok(CreateOutput::None)
        }
    }
}
