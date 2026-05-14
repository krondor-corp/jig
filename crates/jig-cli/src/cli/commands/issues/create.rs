use std::fmt;
use std::io;

use clap::Args;

use jig_core::issues::{IssuePriority, IssueStatus};

use crate::cli::op::Op;
use crate::context::{RepoConfig, RepoCtx};

/// Create a new issue
#[derive(Args, Debug, Clone)]
pub struct Create {
    /// Issue title
    pub title: String,

    /// Template to use (standalone, ticket, epic-index) — file provider only
    #[arg(short, long, default_value = "standalone")]
    pub template: String,

    /// Issue priority (urgent, high, medium, low)
    #[arg(short, long)]
    pub priority: Option<String>,

    /// Category/directory (file) or project name (Linear)
    #[arg(short, long)]
    pub category: Option<String>,

    /// Labels to attach (can specify multiple -l flags)
    #[arg(short, long)]
    pub label: Vec<String>,

    /// Issue body/description (use "-" to read from stdin)
    #[arg(short, long)]
    pub body: Option<String>,

    /// Parent issue ID (e.g. "JIG-19") to create this as a sub-issue
    #[arg(short = 'P', long)]
    pub parent: Option<String>,

    /// Initial status (triage, backlog, planned, in-progress, complete, blocked)
    #[arg(short = 's', long, default_value = "backlog")]
    pub status: String,
}

#[derive(Debug)]
pub struct CreateOutput(pub String);

impl fmt::Display for CreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Created issue: {}", self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error(transparent)]
    Context(#[from] crate::context::ContextError),
    #[error(transparent)]
    Linear(#[from] jig_core::issues::providers::linear::client::LinearError),
    #[error("{0}")]
    Usage(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

fn read_body(body: Option<&str>) -> Result<Option<String>, CreateError> {
    match body {
        Some("-") => {
            let mut buf = String::new();
            io::Read::read_to_string(&mut io::stdin(), &mut buf)?;
            Ok(Some(buf))
        }
        Some(text) => Ok(Some(text.to_string())),
        None => Ok(None),
    }
}

fn run(
    repo: &RepoConfig,
    global: &crate::context::Config,
    cmd: &Create,
) -> Result<CreateOutput, CreateError> {
    let pri: Option<IssuePriority> = cmd.priority.as_deref().and_then(|s| s.parse().ok());
    let initial_status: IssueStatus = cmd
        .status
        .parse()
        .map_err(|_| CreateError::Usage(format!("unknown status: {}", cmd.status)))?;
    let body_text = read_body(cmd.body.as_deref())?;

    let linear_provider = repo.linear_provider(global)?;
    let id = linear_provider.create_issue(
        &cmd.title,
        body_text.as_deref(),
        pri.as_ref(),
        &cmd.label,
        cmd.category.as_deref(),
        cmd.parent.as_deref(),
        Some(&initial_status),
    )?;

    Ok(CreateOutput(id))
}

impl Op for Create {
    type Context = RepoCtx;
    type Error = CreateError;
    type Output = CreateOutput;

    fn build_context(&self) -> Result<RepoCtx, CreateError> {
        Ok(RepoCtx::from_cwd()?)
    }

    fn run(&self, ctx: RepoCtx) -> Result<Self::Output, Self::Error> {
        run(&ctx.repo, &ctx.config, self)
    }
}
