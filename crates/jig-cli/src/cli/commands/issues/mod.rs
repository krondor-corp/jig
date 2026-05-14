//! Issues command — discover, browse, and manage issues.

mod complete;
mod create;
pub mod list;
mod stats;
mod status;
mod update;

use clap::Args;

use crate::cli::op::Op;

pub use list::List;

/// Discover and manage issues
#[derive(Args, Debug, Clone)]
pub struct Issues {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub list: List,
}

crate::command_enum! {
    /// List and browse issues (default)
    (List, List),
    /// Create a new issue
    (Create, create::Create),
    /// Update an existing issue's fields
    (Update, update::Update),
    /// Update issue status
    (Status, status::Status),
    /// Mark an issue as complete
    (Complete, complete::Complete),
    /// Show issue statistics
    (Stats, stats::Stats),
}

impl Op for Issues {
    type Context = ();
    type Output = OpOutput;
    type Error = OpError;

    fn build_context(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
        match &self.command {
            Some(cmd) => cmd.run(()),
            None => Command::List(self.list.clone()).run(()),
        }
    }
}
