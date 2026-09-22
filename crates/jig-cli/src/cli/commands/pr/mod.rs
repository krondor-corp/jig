//! Pr command — push, create, and inspect pull requests.

mod comments;
mod create;

use clap::Args;

use crate::cli::op::Op;

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
    type Context = ();
    type Output = OpOutput;
    type Error = OpError;

    fn build_context(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
        match &self.command {
            Some(cmd) => cmd.run(()),
            None => Command::Create(self.create.clone()).run(()),
        }
    }
}
