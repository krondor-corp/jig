//! Attach command — focus a worker (or repo group) and connect a client.
//!
//! Resolution rules:
//! - In a repo, no branch: focus this repo's group.
//! - Branch given: resolve repo-first, then fall back to a registry-wide
//!   search. A cross-repo collision errors and asks for `--repo`.
//! - `-g/--global`: skip the repo-first step (the worker is elsewhere).
//! - Outside a repo with no branch and no `--repo`: nothing to target.

use clap::Args;

use crate::context::RepoConfig;
use crate::context::RepoRegistry;
use crate::worker::Worker;
use jig_core::git::Branch;
use jig_core::mux::Mux;

use crate::cli::op::{NoOutput, Op};

/// Attach to a worker session
#[derive(Args, Debug, Clone)]
pub struct Attach {
    /// Branch name to switch to
    pub branch: Option<String>,

    /// Search all registered repos even when inside one
    #[arg(long, short)]
    pub global: bool,

    /// Target a registered repo by name
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AttachError {
    #[error(transparent)]
    Context(#[from] crate::context::ContextError),
    #[error(transparent)]
    Worker(#[from] crate::worker::WorkerError),
    #[error(transparent)]
    Mux(#[from] jig_core::mux::MuxError),
    #[error("{0}")]
    Usage(String),
}

impl Op for Attach {
    type Context = ();
    type Error = AttachError;
    type Output = NoOutput;

    fn build_context(&self) -> Result<(), AttachError> {
        Ok(())
    }

    fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
        let kind = mux_kind();
        let branch = self.branch.as_deref();

        if let Some(repo_name) = self.repo.as_deref() {
            let cfg = find_repo_by_name(repo_name)?;
            attach(&cfg, kind, branch.map(Branch::new).as_ref())?;
            return Ok(NoOutput);
        }

        let local = if self.global {
            None
        } else {
            RepoConfig::from_cwd().ok()
        };

        match (&local, branch) {
            (Some(cfg), None) => attach(cfg, kind, None)?,
            (Some(cfg), Some(b)) if cfg.worktrees_path.join(b).exists() => {
                attach(cfg, kind, Some(&Branch::new(b)))?
            }
            (_, Some(b)) => {
                let cfg = find_repo_by_branch(b)?;
                attach(&cfg, kind, Some(&Branch::new(b)))?
            }
            (None, None) => {
                return Err(AttachError::Usage(
                    "nothing to target — run inside a repo, or pass a branch or --repo".into(),
                ))
            }
        }
        Ok(NoOutput)
    }
}

/// Attach has no Op context (it resolves the repo itself), so the mux
/// choice comes straight from global config.
fn mux_kind() -> jig_core::mux::MuxKind {
    crate::context::Config::load().unwrap_or_default().mux
}

fn registered_repos() -> Vec<RepoConfig> {
    let registry = RepoRegistry::load().unwrap_or_default();
    registry
        .repos()
        .iter()
        .filter(|e| e.path.exists())
        .filter_map(|e| RepoConfig::from_path(&e.path).ok())
        .collect()
}

fn find_repo_by_name(name: &str) -> Result<RepoConfig, AttachError> {
    registered_repos()
        .into_iter()
        .find(|c| c.name() == name)
        .ok_or_else(|| AttachError::Usage(format!("no registered repo named '{name}'")))
}

/// Registry-wide search for the repo holding a worker branch.
/// Ambiguity is an error: the same branch name in two repos needs `--repo`.
fn find_repo_by_branch(branch: &str) -> Result<RepoConfig, AttachError> {
    let mut matches: Vec<RepoConfig> = registered_repos()
        .into_iter()
        .filter(|c| c.worktrees_path.join(branch).exists())
        .collect();
    match matches.len() {
        0 => Err(AttachError::Worker(crate::worker::WorkerError::NotFound(
            branch.to_string(),
        ))),
        1 => Ok(matches.remove(0)),
        _ => {
            let names: Vec<String> = matches.iter().map(|c| c.name()).collect();
            Err(AttachError::Usage(format!(
                "branch '{}' exists in multiple repos ({}) — pass --repo",
                branch,
                names.join(", ")
            )))
        }
    }
}

fn attach(
    cfg: &RepoConfig,
    kind: jig_core::mux::MuxKind,
    branch: Option<&Branch>,
) -> Result<(), AttachError> {
    let mux = jig_core::mux::from_group_name(kind, cfg.session_name());
    match branch {
        Some(branch) => {
            let workers = Worker::discover(&jig_core::git::Repo::open(&cfg.repo_root).unwrap());
            let worker = workers
                .iter()
                .find(|w| w.branch() == branch)
                .ok_or_else(|| {
                    AttachError::Worker(crate::worker::WorkerError::NotFound(branch.to_string()))
                })?;
            worker.attach(&mux)?;
            Ok(())
        }
        None => {
            mux.attach()?;
            Ok(())
        }
    }
}
