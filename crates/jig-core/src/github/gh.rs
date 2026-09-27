//! The one place jig shells out to `gh`.
//!
//! Every call is bounded. The monitor actor polls GitHub on every tick for
//! every worker, and its queue holds one request — a `gh` that connects and
//! then goes quiet would stop PR tracking, nudges and pruning for the life of
//! the daemon, while `jig daemon status` kept reporting healthy.

use std::path::Path;
use std::process::Command;

use crate::exec::{Exec, Timeout};

use super::error::{GitHubError, Result};

/// Run `gh <args>`, returning stdout. Errors if it could not run, timed out,
/// or exited non-zero.
pub(crate) fn gh(args: &[&str], dir: Option<&Path>, timeout: Timeout) -> Result<String> {
    let mut command = Command::new("gh");
    command.args(args);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }

    let subcommand = args.first().copied().unwrap_or("gh");
    let output = Exec::command(command)
        .labeled(format!("gh {subcommand}"))
        .timeout(timeout)
        .capturing()
        .run()?;

    if !output.success() {
        return Err(GitHubError::Cli(format!(
            "gh {subcommand} failed: {}",
            output.failure()
        )));
    }
    Ok(output.stdout)
}
