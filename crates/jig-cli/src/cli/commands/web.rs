//! Web command — open the daemon's web view in a browser.
//!
//! The URL comes from the running daemon (`DaemonInfo.web`), not from
//! config: only the daemon knows whether it actually bound the port.

use std::process::Command;

use clap::Args;

use crate::cli::op::Op;
use crate::cli::ui;
use crate::context::{AppCtx, AppPaths};
use crate::daemon::ipc::{self, IpcError};

/// Open the daemon's web view (needs `[fleet] web = true`)
#[derive(Args, Debug, Clone)]
pub struct Web {
    /// Print the URL without opening a browser
    #[arg(long)]
    no_open: bool,
}

/// The web view's URL, printed so scripts can use it.
#[derive(Debug)]
pub struct WebOutput(String);

impl std::fmt::Display for WebOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error("daemon is not running — start it with `jig daemon start`")]
    NotRunning,
    #[error("the daemon is not serving a web view — set `web = true` under [fleet] in ~/.config/jig/config.toml and restart it")]
    Off,
    #[error(transparent)]
    Ipc(#[from] IpcError),
}

impl Op for Web {
    type Context = AppPaths;
    type Error = WebError;
    type Output = WebOutput;

    fn build_context(&self, app: AppCtx) -> Result<AppPaths, WebError> {
        Ok(app.paths)
    }

    fn run(&self, paths: AppPaths) -> Result<Self::Output, Self::Error> {
        let info = ipc::ping(&paths)?.ok_or(WebError::NotRunning)?;
        let url = info.web.ok_or(WebError::Off)?;
        if !self.no_open {
            if let Err(e) = open_browser(&url) {
                ui::warning(&format!("could not open a browser: {e}"));
            }
        }
        Ok(WebOutput(url))
    }
}

/// Hand `url` to the desktop's opener.
fn open_browser(url: &str) -> std::io::Result<()> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(opener)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(drop)
}
