//! `jig fleet join` — enroll this machine on the tailnet as a jig peer.

use clap::Args;
use jig_core::net::tailscale::JIG_TAG;
use jig_core::net::{NetError, NetKind, TailscaleNet};

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::{AppConfig, AppCtx};

/// Join the tailnet as a jig peer
///
/// Runs `tailscale up --authkey <key> --advertise-tags=tag:jig`. Use a
/// reusable, pre-authorized key that is allowed to apply tag:jig, and the same
/// command enrolls every machine.
#[derive(Args, Debug, Clone)]
pub struct Join {
    /// Tailscale (or Headscale) auth key
    authkey: String,

    /// Control server URL, for a self-hosted Headscale
    #[arg(long)]
    login_server: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum JoinError {
    #[error("`jig fleet join` enrolls on tailscale, but this machine has [fleet] net = \"{0}\"")]
    WrongNet(NetKind),
    #[error(transparent)]
    Net(#[from] NetError),
}

impl Op for Join {
    type Context = AppConfig;
    type Error = JoinError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<AppConfig, JoinError> {
        Ok(app.config)
    }

    fn run(&self, config: AppConfig) -> Result<Self::Output, Self::Error> {
        match config.fleet.net {
            Some(NetKind::Tailscale) | None => {}
            Some(other) => return Err(JoinError::WrongNet(other)),
        }

        ui::progress(&format!(
            "joining the tailnet as {}",
            ui::highlight(JIG_TAG)
        ));
        TailscaleNet::join(&self.authkey, self.login_server.as_deref())?;
        ui::success(&format!("joined as {JIG_TAG}"));

        if config.fleet.net.is_none() {
            ui::detail(&format!(
                "set {} in ~/.config/jig/config.toml to use it",
                ui::highlight("[fleet] net = \"tailscale\"")
            ));
        }
        Ok(NoOutput)
    }
}
