//! `jig daemon install` / `uninstall` — run the daemon under the OS.
//!
//! systemd (`--user`) on Linux, a launchd user agent on macOS, via
//! `service-manager`. The service runs `jig daemon start`, so the daemon
//! itself is unchanged; the OS just keeps it alive and starts it at login.

use std::ffi::OsString;
use std::path::PathBuf;

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

/// Environment the daemon needs that a login shell would otherwise supply.
///
/// `PATH`, because jig shells out to `git`, `gh` and the mux; and
/// `XDG_CONFIG_HOME` when it is set, or the service would read a different
/// config directory than the shell that installed it.
const INHERITED: [&str; 2] = ["PATH", "XDG_CONFIG_HOME"];

/// Install the daemon as a user service (systemd or launchd)
#[derive(Args, Debug, Clone)]
pub struct Install {
    /// Install the service but don't start it now
    #[arg(long)]
    no_start: bool,
}

/// Remove the daemon's user service
#[derive(Args, Debug, Clone)]
pub struct Uninstall;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("no service manager on this system (expected systemd or launchd)")]
    Unsupported,
    #[error("could not find the jig binary: {0}")]
    Exe(std::io::Error),
    #[error("{action} the service failed: {source}")]
    Manager {
        action: &'static str,
        source: std::io::Error,
    },
}

impl Op for Install {
    type Context = AppPaths;
    type Error = ServiceError;
    type Output = NoOutput;

    fn build_context(&self, paths: &AppPaths) -> Result<AppPaths, ServiceError> {
        Ok(paths.clone())
    }

    fn run(&self, _: AppPaths) -> Result<Self::Output, Self::Error> {
        let manager = user_manager()?;
        let exe = std::env::current_exe().map_err(ServiceError::Exe)?;
        manager
            .install(install_ctx(
                exe.clone(),
                inherited_env(|name| std::env::var(name)),
            ))
            .map_err(|source| ServiceError::Manager {
                action: "installing",
                source,
            })?;

        ui::success(&format!(
            "installed {} {}",
            ui::highlight(LABEL),
            ui::dim(&format!("({})", exe.display()))
        ));

        if self.no_start {
            ui::detail("it will start at your next login");
            linger_hint();
            return Ok(NoOutput);
        }

        manager
            .start(ServiceStartCtx { label: label() })
            .map_err(|source| ServiceError::Manager {
                action: "starting",
                source,
            })?;
        ui::success("daemon started, and will start again at login");
        ui::detail(&format!(
            "check it with {}",
            ui::highlight("jig daemon status")
        ));
        linger_hint();
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
        let manager = user_manager()?;

        // A service that is already stopped is not an error worth failing on;
        // uninstalling is what was asked for.
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

/// A systemd user service stops when its user logs out, unless lingering is
/// on — which matters on a box you only ssh into.
#[cfg(target_os = "linux")]
fn linger_hint() {
    let user = std::env::var("USER").unwrap_or_else(|_| "$USER".to_string());
    ui::detail(&format!(
        "to keep it running while logged out: {}",
        ui::highlight(&format!("loginctl enable-linger {user}"))
    ));
}

#[cfg(not(target_os = "linux"))]
fn linger_hint() {}

fn label() -> ServiceLabel {
    LABEL.parse().expect("a valid service label")
}

/// The native service manager, set to install for this user rather than
/// system-wide — the daemon watches one person's repos.
fn user_manager() -> Result<Box<dyn ServiceManager>, ServiceError> {
    let mut manager = <dyn ServiceManager>::native().map_err(|_| ServiceError::Unsupported)?;
    if !manager.available().unwrap_or(false) {
        return Err(ServiceError::Unsupported);
    }
    manager
        .set_level(ServiceLevel::User)
        .map_err(|source| ServiceError::Manager {
            action: "preparing",
            source,
        })?;
    Ok(manager)
}

/// The service definition: run `jig daemon start`, restart it if it dies,
/// and start it at login.
fn install_ctx(program: PathBuf, environment: Vec<(String, String)>) -> ServiceInstallCtx {
    ServiceInstallCtx {
        label: label(),
        program,
        args: vec![OsString::from("daemon"), OsString::from("start")],
        contents: None,
        username: None,
        working_directory: None,
        environment: Some(environment),
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

    #[test]
    fn the_service_runs_the_daemon_in_the_foreground() {
        let ctx = install_ctx(PathBuf::from("/usr/local/bin/jig"), Vec::new());
        assert_eq!(ctx.program, PathBuf::from("/usr/local/bin/jig"));
        assert_eq!(
            ctx.args,
            vec![OsString::from("daemon"), OsString::from("start")]
        );
        assert!(ctx.autostart, "the point is surviving a reboot");
        assert!(
            matches!(ctx.restart_policy, RestartPolicy::OnFailure { .. }),
            "a daemon that dies should come back"
        );
    }

    #[test]
    fn only_variables_that_are_set_are_carried_over() {
        let env = inherited_env(|name| match name {
            "PATH" => Ok("/usr/bin".to_string()),
            _ => Err(VarError::NotPresent),
        });
        assert_eq!(env, vec![("PATH".to_string(), "/usr/bin".to_string())]);
    }

    #[test]
    fn a_custom_config_home_follows_the_daemon() {
        // Without this the service would read ~/.config/jig while the shell
        // that installed it reads somewhere else.
        let env = inherited_env(|name| match name {
            "PATH" => Ok("/usr/bin".to_string()),
            "XDG_CONFIG_HOME" => Ok("/home/bot/cfg".to_string()),
            _ => Err(VarError::NotPresent),
        });
        assert_eq!(
            env,
            vec![
                ("PATH".to_string(), "/usr/bin".to_string()),
                ("XDG_CONFIG_HOME".to_string(), "/home/bot/cfg".to_string()),
            ]
        );
    }

    #[test]
    fn the_label_parses() {
        assert_eq!(label().to_qualified_name(), LABEL);
    }
}
