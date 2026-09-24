//! `jig daemon install` / `uninstall` — run the daemon under the OS.
//!
//! The daemon should have exactly the permissions of the person who
//! installed it, because its hooks run their tools (docker, ssh, the mux).
//! That decides the service level:
//!
//! * **Linux** installs a *system* unit with `User=<you>`. systemd runs
//!   `initgroups` for it, so the daemon gets every group you have. A
//!   `systemd --user` service would get your primary group only — the user
//!   manager is started by PID 1 without supplementary groups and cannot
//!   add them — which surfaces later as "permission denied … docker.sock".
//!   `--user` installs one anyway, for when sudo isn't available.
//! * **macOS** installs a launchd user agent, which already runs with your
//!   full group membership.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::Args;
use service_manager::{
    RestartPolicy, ServiceInstallCtx, ServiceLabel, ServiceLevel, ServiceManager, ServiceStartCtx,
    ServiceStatusCtx, ServiceStopCtx, ServiceUninstallCtx,
};

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;
use crate::context::AppPaths;

/// What the service is called to launchd/systemd.
const LABEL: &str = "org.jig.daemon";

/// Variables worth carrying from the installing shell, when set.
///
/// `PATH`, because jig shells out to `git`, `gh` and the mux;
/// `XDG_CONFIG_HOME`, or the service would read a different config
/// directory; `SSH_AUTH_SOCK`, because `git fetch` over SSH asks an agent
/// for the key.
const INHERITED: [&str; 3] = ["PATH", "XDG_CONFIG_HOME", "SSH_AUTH_SOCK"];

/// Where the service runs from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// A system unit running as the installing user: their groups, starts
    /// at boot, needs root to install.
    System,
    /// The user's own service manager: no root, but on Linux only the
    /// primary group.
    User,
}

/// Install the daemon as an OS service (systemd or launchd)
#[derive(Args, Debug, Clone)]
pub struct Install {
    /// Install a user service instead of a system one (no sudo needed)
    ///
    /// On Linux the daemon then runs with your primary group only, so
    /// hooks that need another group — docker, say — will fail.
    #[arg(long)]
    user: bool,

    /// Install the service but don't start it now
    #[arg(long)]
    no_start: bool,
}

/// Remove the daemon's OS service
#[derive(Args, Debug, Clone)]
pub struct Uninstall {
    /// Remove the user service rather than the system one
    #[arg(long)]
    user: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("no service manager on this system (expected systemd or launchd)")]
    Unsupported,
    #[error("a system service has to be installed as root")]
    NeedsRoot,
    #[error("could not work out who to install the service for: {0}")]
    NoUser(String),
    #[error("could not find the jig binary: {0}")]
    Exe(std::io::Error),
    #[error("{action} the service failed: {source}")]
    Manager {
        action: &'static str,
        source: std::io::Error,
    },
}

/// The person the daemon runs as: the caller, or whoever invoked `sudo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetUser {
    pub name: String,
    pub uid: u32,
    pub home: PathBuf,
}

impl Op for Install {
    type Context = AppPaths;
    type Error = ServiceError;
    type Output = NoOutput;

    fn build_context(&self, paths: &AppPaths) -> Result<AppPaths, ServiceError> {
        Ok(paths.clone())
    }

    fn run(&self, _: AppPaths) -> Result<Self::Output, Self::Error> {
        let level = level(self.user);
        ensure_privileged(level)?;

        let target = target_user()?;
        let exe = std::env::current_exe().map_err(ServiceError::Exe)?;
        let env = inherited_env(|name| std::env::var(name));

        let manager = manager(level)?;
        manager
            .install(install_ctx(level, &exe, &target, env))
            .map_err(|source| ServiceError::Manager {
                action: "installing",
                source,
            })?;

        ui::success(&format!(
            "installed {} {}",
            ui::highlight(LABEL),
            ui::dim(&format!("runs as {}", target.name))
        ));
        match level {
            Level::System => ui::detail("starts at boot, with all of your groups"),
            Level::User => {
                ui::detail("starts when you log in");
                if cfg!(target_os = "linux") {
                    ui::warning(
                        "a user service gets your primary group only — hooks needing \
                         docker or similar will fail; reinstall without --user for those",
                    );
                }
            }
        }

        if self.no_start {
            return Ok(NoOutput);
        }

        manager
            .start(ServiceStartCtx { label: label() })
            .map_err(|source| ServiceError::Manager {
                action: "starting",
                source,
            })?;
        ui::success("daemon started");
        ui::detail(&format!(
            "check it with {}",
            ui::highlight("jig daemon status")
        ));
        Ok(NoOutput)
    }
}

impl Op for Uninstall {
    type Context = AppPaths;
    type Error = ServiceError;
    type Output = NoOutput;

    fn build_context(&self, paths: &AppPaths) -> Result<AppPaths, ServiceError> {
        Ok(paths.clone())
    }

    fn run(&self, _: AppPaths) -> Result<Self::Output, Self::Error> {
        let level = level(self.user);
        ensure_privileged(level)?;
        let manager = manager(level)?;

        // Stopping a service that isn't running is not a failure worth
        // reporting; removing it is what was asked for.
        if matches!(
            manager.status(ServiceStatusCtx { label: label() }),
            Ok(service_manager::ServiceStatus::Running)
        ) {
            let _ = manager.stop(ServiceStopCtx { label: label() });
        }

        manager
            .uninstall(ServiceUninstallCtx { label: label() })
            .map_err(|source| ServiceError::Manager {
                action: "removing",
                source,
            })?;

        ui::success(&format!("removed {}", ui::highlight(LABEL)));
        Ok(NoOutput)
    }
}

/// System on Linux unless `--user` was passed; launchd agents are always
/// per-user, and already carry the user's groups.
fn level(user_flag: bool) -> Level {
    if cfg!(target_os = "linux") && !user_flag {
        Level::System
    } else {
        Level::User
    }
}

/// Installing a system unit writes under `/etc` and talks to PID 1.
fn ensure_privileged(level: Level) -> Result<(), ServiceError> {
    if level == Level::System && !is_root() {
        ui::failure("installing a system service needs root");
        ui::detail(&format!("run {}", ui::highlight("sudo jig daemon install")));
        ui::detail(&format!(
            "or {} to install without sudo (Linux: primary group only)",
            ui::highlight("jig daemon install --user")
        ));
        return Err(ServiceError::NeedsRoot);
    }
    Ok(())
}

#[cfg(unix)]
fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

#[cfg(not(unix))]
fn is_root() -> bool {
    true
}

/// Who the daemon should run as: whoever invoked `sudo`, else the caller.
#[cfg(unix)]
fn target_user() -> Result<TargetUser, ServiceError> {
    let user = match std::env::var("SUDO_USER").ok() {
        Some(name) => nix::unistd::User::from_name(&name)
            .ok()
            .flatten()
            .ok_or_else(|| ServiceError::NoUser(format!("no such user: {name}")))?,
        None => nix::unistd::User::from_uid(nix::unistd::Uid::current())
            .ok()
            .flatten()
            .ok_or_else(|| ServiceError::NoUser("the current uid has no passwd entry".into()))?,
    };
    Ok(TargetUser {
        name: user.name,
        uid: user.uid.as_raw(),
        home: user.dir,
    })
}

#[cfg(not(unix))]
fn target_user() -> Result<TargetUser, ServiceError> {
    Err(ServiceError::Unsupported)
}

fn label() -> ServiceLabel {
    LABEL.parse().expect("a valid service label")
}

fn manager(level: Level) -> Result<Box<dyn ServiceManager>, ServiceError> {
    let mut manager = <dyn ServiceManager>::native().map_err(|_| ServiceError::Unsupported)?;
    if !manager.available().unwrap_or(false) {
        return Err(ServiceError::Unsupported);
    }
    let service_level = match level {
        Level::System => ServiceLevel::System,
        Level::User => ServiceLevel::User,
    };
    manager
        .set_level(service_level)
        .map_err(|source| ServiceError::Manager {
            action: "preparing",
            source,
        })?;
    Ok(manager)
}

fn install_ctx(
    level: Level,
    exe: &Path,
    target: &TargetUser,
    env: Vec<(String, String)>,
) -> ServiceInstallCtx {
    ServiceInstallCtx {
        label: label(),
        program: exe.to_path_buf(),
        args: vec![OsString::from("daemon"), OsString::from("start")],
        // The Linux system unit is written by hand: the generated template
        // has no way to say "after the user's runtime directory exists",
        // and that is where the daemon socket lives.
        contents: match level {
            Level::System if cfg!(target_os = "linux") => Some(system_unit(exe, target, &env)),
            _ => None,
        },
        username: Some(target.name.clone()),
        working_directory: None,
        environment: Some(env),
        autostart: true,
        restart_policy: RestartPolicy::OnFailure {
            delay_secs: Some(5),
            // Keep trying: a daemon that cannot start yet (no network at
            // boot, say) should still be running once the machine settles.
            max_retries: None,
            reset_after_secs: None,
        },
    }
}

/// The systemd system unit: runs as `target`, with their groups, after the
/// runtime directory that holds the daemon socket exists.
fn system_unit(exe: &Path, target: &TargetUser, env: &[(String, String)]) -> String {
    let TargetUser { name, uid, home } = target;
    let mut unit = String::new();
    unit.push_str("[Unit]\n");
    unit.push_str("Description=jig daemon\n");
    // `/run/user/<uid>` holds the socket and PID file, and logind owns it —
    // pull it up first, or a start at boot finds nothing there.
    unit.push_str(&format!("Requires=user-runtime-dir@{uid}.service\n"));
    unit.push_str(&format!(
        "After=user-runtime-dir@{uid}.service network-online.target\n"
    ));
    unit.push_str("\n[Service]\n");
    unit.push_str(&format!("User={name}\n"));
    unit.push_str(&format!("ExecStart={} daemon start\n", exe.display()));
    unit.push_str("Restart=on-failure\n");
    unit.push_str("RestartSec=5\n");
    unit.push_str(&format!("Environment=\"HOME={}\"\n", home.display()));
    unit.push_str(&format!(
        "Environment=\"XDG_RUNTIME_DIR=/run/user/{uid}\"\n"
    ));
    for (var, val) in env {
        unit.push_str(&format!("Environment=\"{var}={val}\"\n"));
    }
    unit.push_str("\n[Install]\n");
    unit.push_str("WantedBy=multi-user.target\n");
    unit
}

/// [`INHERITED`] variables that are actually set, in order.
fn inherited_env(
    var: impl Fn(&str) -> Result<String, std::env::VarError>,
) -> Vec<(String, String)> {
    INHERITED
        .iter()
        .filter_map(|name| Some(((*name).to_string(), var(name).ok()?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::VarError;

    fn target() -> TargetUser {
        TargetUser {
            name: "bot".into(),
            uid: 1001,
            home: PathBuf::from("/home/bot"),
        }
    }

    fn env(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn the_service_runs_the_daemon_in_the_foreground() {
        let ctx = install_ctx(
            Level::User,
            Path::new("/usr/local/bin/jig"),
            &target(),
            vec![],
        );
        assert_eq!(ctx.program, PathBuf::from("/usr/local/bin/jig"));
        assert_eq!(
            ctx.args,
            vec![OsString::from("daemon"), OsString::from("start")]
        );
        assert!(ctx.autostart, "the point is surviving a reboot");
        assert!(matches!(
            ctx.restart_policy,
            RestartPolicy::OnFailure { .. }
        ));
    }

    #[test]
    fn the_system_unit_runs_as_the_installing_user() {
        let unit = system_unit(Path::new("/home/bot/.local/bin/jig"), &target(), &[]);
        assert!(unit.contains("User=bot\n"), "{unit}");
        assert!(
            unit.contains("ExecStart=/home/bot/.local/bin/jig daemon start\n"),
            "{unit}"
        );
        assert!(unit.contains("WantedBy=multi-user.target"), "{unit}");
    }

    #[test]
    fn the_system_unit_waits_for_the_runtime_dir_that_holds_the_socket() {
        // Without this an early-boot start resolves XDG_RUNTIME_DIR to a
        // directory logind has not created yet.
        let unit = system_unit(Path::new("/usr/bin/jig"), &target(), &[]);
        assert!(
            unit.contains("Requires=user-runtime-dir@1001.service"),
            "{unit}"
        );
        assert!(
            unit.contains("Environment=\"XDG_RUNTIME_DIR=/run/user/1001\""),
            "{unit}"
        );
        assert!(unit.contains("Environment=\"HOME=/home/bot\""), "{unit}");
    }

    #[test]
    fn the_system_unit_carries_the_shell_environment() {
        let unit = system_unit(
            Path::new("/usr/bin/jig"),
            &target(),
            &env(&[("PATH", "/home/bot/.local/bin:/usr/bin")]),
        );
        assert!(
            unit.contains("Environment=\"PATH=/home/bot/.local/bin:/usr/bin\""),
            "{unit}"
        );
    }

    #[test]
    fn a_user_install_writes_no_unit_of_its_own() {
        let ctx = install_ctx(Level::User, Path::new("/usr/bin/jig"), &target(), vec![]);
        assert!(
            ctx.contents.is_none(),
            "the platform's own template is right for a user service"
        );
    }

    #[test]
    fn only_variables_that_are_set_are_carried_over() {
        let carried = inherited_env(|name| match name {
            "PATH" => Ok("/usr/bin".to_string()),
            _ => Err(VarError::NotPresent),
        });
        assert_eq!(carried, env(&[("PATH", "/usr/bin")]));
    }

    #[test]
    fn the_agent_socket_and_config_home_follow_the_daemon() {
        // Without these the daemon reads a different config directory, and
        // `git fetch` over SSH has no agent to ask for a key.
        let carried = inherited_env(|name| match name {
            "PATH" => Ok("/usr/bin".to_string()),
            "XDG_CONFIG_HOME" => Ok("/home/bot/cfg".to_string()),
            "SSH_AUTH_SOCK" => Ok("/run/user/1001/gnupg/S.gpg-agent.ssh".to_string()),
            _ => Err(VarError::NotPresent),
        });
        assert_eq!(
            carried,
            env(&[
                ("PATH", "/usr/bin"),
                ("XDG_CONFIG_HOME", "/home/bot/cfg"),
                ("SSH_AUTH_SOCK", "/run/user/1001/gnupg/S.gpg-agent.ssh"),
            ])
        );
    }

    #[test]
    fn the_label_parses() {
        assert_eq!(label().to_qualified_name(), LABEL);
    }
}
