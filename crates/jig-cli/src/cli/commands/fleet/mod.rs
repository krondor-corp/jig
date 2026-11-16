//! Fleet command — this machine's place on the fleet's private network.
//!
//! See `docs/fleet-spec.md`. Nothing here opens a listener.

mod join;

use clap::Args;

use crate::cli::op::Op;
use crate::context::AppCtx;

/// Enroll this machine in the fleet's private network
#[derive(Args, Debug, Clone)]
pub struct Fleet {
    #[command(subcommand)]
    pub command: Command,
}

crate::command_enum! {
    /// Join the tailnet as a jig peer (`tailscale up` with a tag:jig key)
    (Join, join::Join),
}

impl Op for Fleet {
    type Context = AppCtx;
    type Output = OpOutput;
    type Error = OpError;

    fn build_context(&self, app: AppCtx) -> Result<AppCtx, Self::Error> {
        Ok(app)
    }

    fn run(&self, app: AppCtx) -> Result<Self::Output, Self::Error> {
        self.command.run(app)
    }
}
