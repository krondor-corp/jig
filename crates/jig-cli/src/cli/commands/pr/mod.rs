//! Pr command — push, create, and inspect pull requests.

mod comments;
mod create;

use clap::Args;

use crate::cli::op::Op;
use crate::context::JigDirs;

pub use create::Create;

/// Push, create, and inspect pull requests
#[derive(Args, Debug, Clone)]
pub struct Pr {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub create: Create,
}

crate::command_enum! {
    /// Push current branch and create a draft PR (default)
    (Create, Create),
    /// Show review comments and feedback on the current PR
    (Comments, comments::Comments),
}

impl Op for Pr {
    type Context = JigDirs;
    type Output = OpOutput;
    type Error = OpError;

    fn build_context(&self, dirs: &JigDirs) -> Result<JigDirs, Self::Error> {
        Ok(dirs.clone())
    }

    fn run(&self, dirs: JigDirs) -> Result<Self::Output, Self::Error> {
        match &self.command {
            Some(cmd) => cmd.run(dirs),
            None => Command::Create(self.create.clone()).run(dirs),
        }
    }

    fn log_sink(&self) -> crate::cli::op::LogSink {
        match &self.command {
            Some(cmd) => cmd.log_sink(),
            None => Command::Create(self.create.clone()).log_sink(),
        }
    }
}
