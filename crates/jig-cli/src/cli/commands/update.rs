//! Update command - update jig to latest version

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::Args;

use jig_core::exec::{Exec, Timeout};

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::{AppCtx, AppPaths};
use crate::daemon::ipc;

const GITHUB_REPO: &str = "krondor-corp/jig";
const INSTALL_SCRIPT_URL: &str =
    "https://raw.githubusercontent.com/krondor-corp/jig/main/install.sh";

/// Update jig to latest version
#[derive(Args, Debug, Clone)]
pub struct Update {
    /// Force update, discarding local changes
    #[arg(long, short)]
    pub force: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("{0}")]
    Failed(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Installation method detection
#[derive(Debug, Clone, PartialEq, Eq)]
enum InstallMethod {
    /// A released binary we can replace where it stands, wherever that is.
    Binary(PathBuf),
    /// A released binary in a directory this user cannot write to —
    /// `/usr/local/bin` owned by root, typically.
    Protected(PathBuf),
    /// Installed via cargo install
    Cargo(PathBuf),
    /// Running from source/target directory
    Source(PathBuf),
}

impl InstallMethod {
    fn description(&self) -> &str {
        match self {
            InstallMethod::Binary(_) => "released binary",
            InstallMethod::Protected(_) => "released binary (not writable by you)",
            InstallMethod::Cargo(_) => "cargo install (~/.cargo/bin)",
            InstallMethod::Source(_) => "source build (target/)",
        }
    }
}

impl Op for Update {
    type Context = AppPaths;
    type Error = UpdateError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<AppPaths, UpdateError> {
        Ok(app.paths)
    }

    fn run(&self, paths: AppPaths) -> Result<Self::Output, Self::Error> {
        let install_method = detect_installation()?;
        let current_version = env!("CARGO_PKG_VERSION");

        // Print header
        ui::header("Update");
        eprintln!("  Current version: {}", ui::highlight(current_version));
        eprintln!("  Installation: {}", ui::dim(install_method.description()));
        eprintln!();

        // Check for latest version
        ui::progress("Checking for updates...");
        let latest_version = get_latest_version()?;
        eprintln!("  Latest version: {}", ui::highlight(&latest_version));
        eprintln!();

        // Compare versions
        let needs_update = is_newer_version(current_version, &latest_version);

        if !needs_update && !self.force {
            ui::success("Already up to date!");
            return Ok(NoOutput);
        }

        if needs_update {
            ui::progress(&format!(
                "New version available: {} → {}",
                ui::dim(current_version),
                ui::highlight(&latest_version)
            ));
        } else {
            ui::progress("Forcing update...");
        }

        match install_method {
            InstallMethod::Binary(ref path) => {
                run_install_script(path.parent().unwrap_or(Path::new(".")))?;
            }
            InstallMethod::Protected(ref path) => {
                let dir = path.parent().unwrap_or(Path::new("."));
                eprintln!();
                eprintln!(
                    "{} is not writable by you.",
                    ui::highlight(&dir.display().to_string())
                );
                eprintln!();
                eprintln!("Re-run the installer as root:");
                ui::detail(&format!(
                    "curl -fsSL {} | sudo INSTALL_DIR={} bash",
                    INSTALL_SCRIPT_URL,
                    shell_quote(dir)
                ));
                return Ok(NoOutput);
            }
            InstallMethod::Cargo(_) | InstallMethod::Source(_) => {
                // Prompt for dev builds
                eprintln!();
                eprintln!("You're running a development build.");

                let home_bin = dirs::home_dir()
                    .unwrap_or_default()
                    .join(".local")
                    .join("bin");
                if prompt_confirm("Install latest release to ~/.local/bin?", true)? {
                    run_install_script(&home_bin)?;

                    // Check for old cargo bin if this was a cargo install
                    if matches!(install_method, InstallMethod::Cargo(_)) {
                        // Don't remove the current binary - that would be confusing
                        // The user might still want to keep their dev setup
                    } else {
                        // For source builds, check if there's an old cargo bin to clean up
                        check_and_remove_old_cargo_bin()?;
                    }
                } else {
                    eprintln!();
                    eprintln!("To update manually, run:");
                    ui::detail(&format!(
                        "cargo install --git https://github.com/{}",
                        GITHUB_REPO
                    ));
                    return Ok(NoOutput);
                }
            }
        }

        eprintln!();
        ui::success("Updated successfully!");

        // The running daemon is still executing the binary it started with;
        // replacing the file on disk changed nothing about that process.
        if let Ok(Some(info)) = ipc::ping(&paths) {
            eprintln!();
            ui::warning(&format!(
                "the daemon (pid {}) is still running the old version",
                info.pid
            ));
            ui::detail(&format!(
                "restart it with {}",
                ui::highlight("jig daemon restart")
            ));
        }

        Ok(NoOutput)
    }
}

/// Detect how jig was installed
/// How this jig got here, and whether we can replace it.
///
/// What matters is not which directory it sits in but whether we can write
/// there. `install.sh` honours `INSTALL_DIR`, so a released binary is just as
/// likely to be in `/usr/local/bin` as `~/.local/bin`, and refusing to update
/// the former because it is not on a hardcoded list helps nobody.
fn detect_installation() -> Result<InstallMethod, UpdateError> {
    let exe_path = std::env::current_exe()?;
    let path_str = exe_path.to_string_lossy();

    if path_str.contains("/.cargo/bin/") {
        return Ok(InstallMethod::Cargo(exe_path));
    }
    if path_str.contains("/target/") {
        return Ok(InstallMethod::Source(exe_path));
    }
    Ok(match exe_path.parent().is_some_and(writable) {
        true => InstallMethod::Binary(exe_path),
        false => InstallMethod::Protected(exe_path),
    })
}

/// Whether this process could put a new file in `dir`.
///
/// Asked by trying, rather than reading permission bits: ownership, groups,
/// ACLs and read-only mounts all decide this, and only the kernel knows.
fn writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".jig-update-probe-{}", std::process::id()));
    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Fetch the latest version from GitHub releases
fn get_latest_version() -> Result<String, UpdateError> {
    let mut curl = Command::new("curl");
    curl.args([
        "-fsSL",
        &format!(
            "https://api.github.com/repos/{}/releases/latest",
            GITHUB_REPO
        ),
    ]);
    let output = Exec::command(curl)
        .timeout(Timeout::NETWORK)
        .capturing()
        .run()
        .map_err(|e| UpdateError::Failed(e.to_string()))?;

    if !output.success() {
        let stderr = output.failure();
        return Err(UpdateError::Failed(format!(
            "Failed to fetch latest version: {}",
            stderr
        )));
    }

    let body = &output.stdout;

    // Parse tag_name from JSON response
    // Looking for: "tag_name": "v0.5.1",
    for line in body.lines() {
        if line.contains("\"tag_name\"") {
            if let Some(start) = line.find(':') {
                let value = &line[start + 1..];
                // Strip whitespace, trailing comma, then quotes
                let value = value.trim().trim_end_matches(',').trim_matches('"');
                // Remove leading 'v' if present
                let version = value.trim_start_matches('v');
                return Ok(version.to_string());
            }
        }
    }

    Err(UpdateError::Failed(
        "Could not parse version from GitHub response".to_string(),
    ))
}

/// Prompt user for yes/no confirmation
fn prompt_confirm(message: &str, default_yes: bool) -> Result<bool, UpdateError> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    eprint!("{} {} ", message, suffix);
    io::stderr().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    let answer = line.trim().to_lowercase();
    if answer.is_empty() {
        Ok(default_yes)
    } else {
        Ok(answer == "y" || answer == "yes")
    }
}

/// Run the install script
/// Re-run the installer, targeting the directory we are replacing.
///
/// Without `INSTALL_DIR` the script defaults to `~/.local/bin`, which for a
/// binary installed anywhere else would leave a second jig on the system and
/// update whichever one PATH happened to prefer.
/// Single-quote a path for the shell, so a directory with a space in it
/// does not silently become two arguments.
fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', r"'\''"))
}

fn run_install_script(dir: &Path) -> Result<(), UpdateError> {
    eprintln!();
    ui::progress(&format!("Installing to {}...", dir.display()));
    eprintln!();

    // Not an `Exec`: the installer streams its progress straight to the
    // user's terminal, and there is a person here who can ctrl-c it.
    let status = Command::new("bash")
        .args([
            "-c",
            &format!(
                "curl -fsSL {} | INSTALL_DIR={} bash",
                INSTALL_SCRIPT_URL,
                shell_quote(dir)
            ),
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(UpdateError::Failed("Install script failed".to_string()));
    }

    Ok(())
}

/// Check if cargo bin jig exists and prompt for removal
fn check_and_remove_old_cargo_bin() -> Result<(), UpdateError> {
    let home = std::env::var("HOME").unwrap_or_default();
    let cargo_bin = PathBuf::from(&home).join(".cargo/bin/jig");

    if cargo_bin.exists() {
        eprintln!();
        eprintln!(
            "Found old build at {}",
            ui::dim(&cargo_bin.display().to_string())
        );

        if prompt_confirm("Remove it?", true)? {
            std::fs::remove_file(&cargo_bin)?;
            ui::success(&format!("Removed {}", cargo_bin.display()));
        }
    }

    Ok(())
}

/// Compare semver versions (simple implementation)
fn is_newer_version(current: &str, latest: &str) -> bool {
    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v
            .trim_start_matches('v')
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect();
        (
            *parts.first().unwrap_or(&0),
            *parts.get(1).unwrap_or(&0),
            *parts.get(2).unwrap_or(&0),
        )
    };

    let current = parse_version(current);
    let latest = parse_version(latest);

    latest > current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_writable_directory_is_writable() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(writable(tmp.path()));
        // And nothing is left behind by asking.
        assert_eq!(std::fs::read_dir(tmp.path()).unwrap().count(), 0);
    }

    #[test]
    fn a_directory_that_is_not_there_is_not_writable() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(!writable(&tmp.path().join("nope")));
    }

    /// `/usr/local/bin` on a machine where jig was installed with
    /// `INSTALL_DIR=/usr/local/bin` — a normal install that the old
    /// path-substring check called "unknown" and refused to update.
    #[test]
    fn a_root_owned_bin_is_protected_not_unknown() {
        let dir = Path::new("/usr/local/bin");
        if !dir.exists() || nix::unistd::Uid::effective().is_root() {
            return; // nothing to prove here
        }
        assert!(
            !writable(dir),
            "this test assumes /usr/local/bin is not writable by the test user"
        );
    }

    #[test]
    fn paths_with_spaces_survive_the_shell() {
        let quoted = shell_quote(Path::new("/opt/my tools/bin"));
        assert_eq!(quoted, "'/opt/my tools/bin'");
    }
}
