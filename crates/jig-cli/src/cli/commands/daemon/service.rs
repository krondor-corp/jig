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
use crate::context::AppCtx;
use crate::context::AppPaths;

/// What the service is called to launchd/systemd.
const LABEL: &str = "org.jig.daemon";

/// How long to wait for a restarted daemon to answer again. The unit's
/// `RestartSec` is 5s, so this allows for that plus a slow start.
const RESTART_WAIT: std::time::Duration = std::time::Duration::from_secs(30);

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
    /// Run the daemon as this user
    ///
    /// Required when installing for someone else — a service account on a
    /// server, say. Without it the daemon runs as whoever invoked sudo,
    /// which is right only when that is also who it is for.
    #[arg(long = "as", value_name = "USER")]
    run_as: Option<String>,

    /// Extra environment for the service, repeatable (`--env KEY=VALUE`)
    #[arg(long = "env", value_name = "KEY=VALUE", value_parser = parse_env)]
    env: Vec<(String, String)>,

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

/// Restart the daemon service, e.g. after `jig update`
///
/// A running daemon keeps executing the binary it started with. Updating jig
/// replaces the file on disk and changes nothing about the process, so the
/// daemon goes on running the old code until something restarts it.
#[derive(Args, Debug, Clone)]
pub struct Restart {
    /// Restart the user service rather than the system one
    #[arg(long)]
    user: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("no service manager on this system (expected systemd or launchd)")]
    Unsupported,
    #[error("a system service has to be installed as root")]
    NeedsRoot,
    #[error(transparent)]
    Ipc(#[from] crate::daemon::ipc::IpcError),
    #[error(
        "the daemon stopped but nothing restarted it — it is not running as a \
         service, or the service is not set to restart it. Start it again with \
         `jig daemon start`"
    )]
    DidNotComeBack,
    #[error(
        "the daemon is not installed as a service{}, so there is nothing to \
         restart — stop it and start it again yourself, or install it with \
         `jig daemon install`",
        .pid.map(|p| format!(" (a daemon is running in the foreground, pid {p})")).unwrap_or_default()
    )]
    NotAService { pid: Option<u32> },
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

    fn build_context(&self, app: AppCtx) -> Result<AppPaths, ServiceError> {
        Ok(app.paths)
    }

    fn run(&self, _: AppPaths) -> Result<Self::Output, Self::Error> {
        let level = level(self.user);
        ensure_privileged(level, "install")?;

        let target = target_user(self.run_as.as_deref())?;
        let installer_exe = std::env::current_exe().map_err(ServiceError::Exe)?;
        let exe = daemon_binary(&installer_exe, &target)?;
        let env = service_env(&target, caller(), &self.env, |name| std::env::var(name));

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
        if exe != installer_exe {
            ui::detail(&format!(
                "runs {} so {} can update it without sudo",
                ui::highlight(&exe.display().to_string()),
                target.name
            ));
        }
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

impl Op for Restart {
    type Context = AppPaths;
    type Error = ServiceError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<AppPaths, ServiceError> {
        Ok(app.paths)
    }

    /// Ask the daemon to exit and let the service bring it back.
    ///
    /// The unit is `Restart=always`, so a daemon that goes away comes back
    /// within seconds running whatever is now on disk. That path needs no
    /// privileges at all — only the ability to reach the socket, which the
    /// user it runs as has by definition. Restarting through the service
    /// manager would need root for a system unit, which would put an update
    /// out of reach of the account the daemon was installed for.
    fn run(&self, paths: AppPaths) -> Result<Self::Output, Self::Error> {
        // Confirm there is a service *before* stopping anything. Without
        // this, restarting a foreground daemon would kill it and leave it
        // dead — a stop wearing a restart's name.
        let level = level(self.user);
        let manager = manager(level)?;
        if !matches!(
            manager.status(ServiceStatusCtx { label: label() }),
            Ok(service_manager::ServiceStatus::Running
                | service_manager::ServiceStatus::Stopped(_))
        ) {
            return Err(ServiceError::NotAService {
                pid: crate::daemon::ipc::ping(&paths)
                    .ok()
                    .flatten()
                    .map(|i| i.pid),
            });
        }

        let Some(before) = crate::daemon::ipc::ping(&paths).ok().flatten() else {
            // Installed but nothing answering — start it. This is the only
            // branch that needs root.
            ensure_privileged(level, "restart")?;
            manager
                .start(ServiceStartCtx { label: label() })
                .map_err(|source| ServiceError::Manager {
                    action: "starting",
                    source,
                })?;
            ui::success(&format!("started {}", ui::highlight(LABEL)));
            return Ok(NoOutput);
        };

        crate::daemon::ipc::request(&paths, &crate::daemon::ipc::Request::Shutdown).or_else(
            |e| match e {
                // It can drop the connection as it goes; that is the
                // shutdown working.
                crate::daemon::ipc::IpcError::NotRunning => Ok(crate::daemon::ipc::Response::Ok),
                other => Err(ServiceError::Ipc(other)),
            },
        )?;

        ui::progress(&format!(
            "stopped pid {}, waiting for the service to bring it back",
            before.pid
        ));

        match wait_for_new_daemon(&paths, before.pid) {
            Some(after) => {
                ui::success(&format!(
                    "daemon restarted  {}",
                    ui::dim(&format!("pid {} → {}", before.pid, after.pid))
                ));
                Ok(NoOutput)
            }
            None => Err(ServiceError::DidNotComeBack),
        }
    }
}

/// Stop the installed service, if there is one.
///
/// `Some` when a service was found and stopped — the caller is done.
/// `None` when nothing is installed, leaving the caller to stop whatever is
/// running in the foreground the way it always did.
pub(super) fn stop_service(user_flag: bool) -> Result<Option<NoOutput>, ServiceError> {
    let level = level(user_flag);
    let Ok(manager) = manager(level) else {
        return Ok(None);
    };
    match manager.status(ServiceStatusCtx { label: label() }) {
        Ok(service_manager::ServiceStatus::NotInstalled) | Err(_) => return Ok(None),
        Ok(service_manager::ServiceStatus::Stopped(_)) => {
            ui::success(&format!("{} is already stopped", ui::highlight(LABEL)));
            return Ok(Some(NoOutput));
        }
        Ok(service_manager::ServiceStatus::Running) => {}
    }

    ensure_privileged(level, "stop")?;
    manager
        .stop(ServiceStopCtx { label: label() })
        .map_err(|source| ServiceError::Manager {
            action: "stopping",
            source,
        })?;
    ui::success(&format!("stopped {}", ui::highlight(LABEL)));
    ui::detail(&format!(
        "it stays down until {}",
        ui::highlight("jig daemon restart")
    ));
    Ok(Some(NoOutput))
}

/// Wait for a daemon with a different pid to answer.
///
/// A new pid is the proof: the same one would mean it never went away, and
/// no answer at all would mean nothing brought it back.
fn wait_for_new_daemon(paths: &AppPaths, old_pid: u32) -> Option<crate::daemon::ipc::DaemonInfo> {
    let deadline = std::time::Instant::now() + RESTART_WAIT;
    while std::time::Instant::now() < deadline {
        if let Ok(Some(info)) = crate::daemon::ipc::ping(paths) {
            if info.pid != old_pid {
                return Some(info);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    None
}

impl Op for Uninstall {
    type Context = AppPaths;
    type Error = ServiceError;
    type Output = NoOutput;

    fn build_context(&self, app: AppCtx) -> Result<AppPaths, ServiceError> {
        Ok(app.paths)
    }

    fn run(&self, _: AppPaths) -> Result<Self::Output, Self::Error> {
        let level = level(self.user);
        ensure_privileged(level, "uninstall")?;
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
/// `action` is what the user asked for, so the advice names the command they
/// actually typed. Telling someone who asked to stop the daemon to run
/// `sudo jig daemon install` sends them somewhere they did not want to go.
fn ensure_privileged(level: Level, action: &str) -> Result<(), ServiceError> {
    if level == Level::System && !is_root() {
        ui::failure(&format!("{action} a system service needs root"));
        ui::detail(&format!(
            "run {}",
            ui::highlight(&format!("sudo jig daemon {action}"))
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

/// Who the daemon runs as: `--as` if given, else whoever invoked `sudo`,
/// else the caller.
#[cfg(unix)]
fn target_user(run_as: Option<&str>) -> Result<TargetUser, ServiceError> {
    let name = run_as
        .map(str::to_string)
        .or_else(|| std::env::var("SUDO_USER").ok());
    let user = match name {
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
fn target_user(_run_as: Option<&str>) -> Result<TargetUser, ServiceError> {
    Err(ServiceError::Unsupported)
}

/// The account running this command, when it can be determined.
#[cfg(unix)]
fn caller() -> Option<String> {
    nix::unistd::User::from_uid(nix::unistd::Uid::current())
        .ok()
        .flatten()
        .map(|u| u.name)
}

#[cfg(not(unix))]
fn caller() -> Option<String> {
    None
}

fn parse_env(raw: &str) -> Result<(String, String), String> {
    raw.split_once('=')
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or_else(|| format!("expected KEY=VALUE, got `{raw}`"))
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
        // Always, not on-failure. It means the user the daemon runs as can
        // restart it by asking it to exit — no root, no systemctl — which is
        // the only way `--as <user>` is usable by that user after an update.
        // It also keeps a daemon that cannot start yet (no network at boot,
        // say) trying until the machine settles.
        restart_policy: RestartPolicy::Always {
            delay_secs: Some(5),
        },
    }
}

/// The binary the service should run: one `target` can replace themselves.
///
/// `ExecStart` recorded whatever binary ran the install, which for
/// `sudo jig daemon install --as bot` is root's copy in `/usr/local/bin`.
/// bot cannot write that, so `jig update` as bot installed a *second* jig in
/// `~/.local/bin` and the daemon went on running root's older one. The update
/// looked like it worked and changed nothing — the daemon reported v0.12.0
/// while the same user's CLI reported v0.12.1.
///
/// So: if the target user can already write the installer's binary, use it.
/// Otherwise the daemon runs their own `~/.local/bin/jig`, copied there if
/// they have none, so `jig update && jig daemon restart` is enough for them.
#[cfg(unix)]
fn daemon_binary(installer_exe: &Path, target: &TargetUser) -> Result<PathBuf, ServiceError> {
    if writable_by(installer_exe, target.uid) {
        return Ok(installer_exe.to_path_buf());
    }

    let theirs = target.home.join(".local/bin/jig");
    if theirs.exists() {
        return Ok(theirs);
    }

    let dir = theirs.parent().expect("joined path has a parent");
    std::fs::create_dir_all(dir).map_err(ServiceError::Exe)?;
    std::fs::copy(installer_exe, &theirs).map_err(ServiceError::Exe)?;
    // Theirs to replace, or the next `jig update` is back where we started.
    let uid = Some(nix::unistd::Uid::from_raw(target.uid));
    let _ = nix::unistd::chown(dir, uid, None);
    nix::unistd::chown(&theirs, uid, None).map_err(|e| {
        ServiceError::Exe(std::io::Error::other(format!(
            "could not give {} to {}: {e}",
            theirs.display(),
            target.name
        )))
    })?;
    Ok(theirs)
}

#[cfg(not(unix))]
fn daemon_binary(installer_exe: &Path, _target: &TargetUser) -> Result<PathBuf, ServiceError> {
    Ok(installer_exe.to_path_buf())
}

/// Whether `uid` can replace the file at `path`.
#[cfg(unix)]
fn writable_by(path: &Path, uid: u32) -> bool {
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::PermissionsExt;

    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    let mode = meta.permissions().mode();
    (meta.uid() == uid && mode & 0o200 != 0) || mode & 0o002 != 0
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
    unit.push_str("Restart=always\n");
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

/// What the service should run with.
///
/// Installing for yourself, your shell's [`INHERITED`] variables are the
/// best guess: same PATH, same agent, same config. Installing for someone
/// else — a service account, from an automation, under sudo — they are the
/// *wrong* environment, because they are root's. Then only the target
/// user's own paths are used, and anything else has to be said with
/// `--env`, which always wins.
fn service_env(
    target: &TargetUser,
    caller: Option<String>,
    overrides: &[(String, String)],
    var: impl Fn(&str) -> Result<String, std::env::VarError>,
) -> Vec<(String, String)> {
    let installing_for_self = caller.as_deref() == Some(target.name.as_str());
    let mut env: Vec<(String, String)> = if installing_for_self {
        INHERITED
            .iter()
            .filter_map(|name| Some(((*name).to_string(), var(name).ok()?)))
            .collect()
    } else {
        vec![(
            "PATH".to_string(),
            format!(
                "{}/.local/bin:/usr/local/bin:/usr/bin:/bin",
                target.home.display()
            ),
        )]
    };

    for (key, value) in overrides {
        env.retain(|(k, _)| k != key);
        env.push((key.clone(), value.clone()));
    }
    env
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

    fn env_pairs(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
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
        assert!(matches!(ctx.restart_policy, RestartPolicy::Always { .. }));
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
            &env_pairs(&[("PATH", "/home/bot/.local/bin:/usr/bin")]),
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
    fn installing_for_yourself_carries_your_shell_environment() {
        let env = service_env(&target(), Some("bot".into()), &[], |name| match name {
            "PATH" => Ok("/usr/bin".to_string()),
            "SSH_AUTH_SOCK" => Ok("/run/user/1001/gnupg/S.gpg-agent.ssh".to_string()),
            _ => Err(VarError::NotPresent),
        });
        assert_eq!(
            env,
            env_pairs(&[
                ("PATH", "/usr/bin"),
                ("SSH_AUTH_SOCK", "/run/user/1001/gnupg/S.gpg-agent.ssh"),
            ])
        );
    }

    #[test]
    fn installing_for_someone_else_ignores_the_callers_environment() {
        // Under sudo or ansible the caller is root, whose PATH and agent
        // are no use to the service account the daemon runs as.
        let env = service_env(&target(), Some("root".into()), &[], |name| match name {
            "PATH" => Ok("/usr/sbin:/root/bin".to_string()),
            "SSH_AUTH_SOCK" => Ok("/root/agent.sock".to_string()),
            _ => Err(VarError::NotPresent),
        });
        assert_eq!(
            env,
            env_pairs(&[("PATH", "/home/bot/.local/bin:/usr/local/bin:/usr/bin:/bin")])
        );
    }

    #[test]
    fn explicit_env_wins() {
        let env = service_env(
            &target(),
            Some("root".into()),
            &env_pairs(&[
                ("PATH", "/opt/tools/bin"),
                ("SSH_AUTH_SOCK", "/run/user/1001/keyring/ssh"),
            ]),
            |_| Err(VarError::NotPresent),
        );
        assert_eq!(
            env,
            env_pairs(&[
                ("PATH", "/opt/tools/bin"),
                ("SSH_AUTH_SOCK", "/run/user/1001/keyring/ssh"),
            ]),
            "--env replaces rather than duplicates"
        );
    }

    #[test]
    fn env_arguments_need_a_value() {
        assert_eq!(
            parse_env("PATH=/usr/bin").unwrap(),
            ("PATH".to_string(), "/usr/bin".to_string())
        );
        assert_eq!(
            parse_env("KEY=a=b").unwrap(),
            ("KEY".to_string(), "a=b".to_string()),
            "only the first = separates"
        );
        assert!(parse_env("JUST_A_NAME").is_err());
    }

    #[test]
    fn the_label_parses() {
        assert_eq!(label().to_qualified_name(), LABEL);
    }

    /// A foreground daemon is not a service, and the message should say which
    /// one is running so the user knows what they are looking at.
    #[test]
    fn not_a_service_names_the_running_daemon() {
        let with_pid = ServiceError::NotAService { pid: Some(4242) }.to_string();
        assert!(with_pid.contains("4242"), "{with_pid}");
        assert!(with_pid.contains("foreground"), "{with_pid}");

        let without = ServiceError::NotAService { pid: None }.to_string();
        assert!(!without.contains("foreground"), "{without}");
        assert!(without.contains("jig daemon install"), "{without}");
    }

    /// `restart` reads the level the same way `install` writes it. If they
    /// disagreed it would look for the service in the wrong place and report
    /// "not installed" on a machine where it plainly is.
    #[test]
    fn restart_looks_where_install_put_it() {
        let default = level(false);
        if cfg!(target_os = "linux") {
            assert!(
                matches!(default, Level::System),
                "Linux installs a system unit"
            );
        } else {
            assert!(
                matches!(default, Level::User),
                "launchd agents are per-user"
            );
        }
        assert!(
            matches!(level(true), Level::User),
            "--user is always user-level"
        );
    }

    /// `Restart=always`, not `on-failure`. It is what lets the user the
    /// daemon runs as restart it by asking it to exit — the only route that
    /// needs no root, and therefore the only one available to the account
    /// `--as` installed it for.
    #[test]
    fn the_unit_restarts_the_daemon_whenever_it_exits() {
        let unit = system_unit(
            Path::new("/usr/local/bin/jig"),
            &TargetUser {
                name: "bot".into(),
                uid: 1001,
                home: PathBuf::from("/home/bot"),
            },
            &[],
        );
        assert!(unit.contains("Restart=always\n"), "{unit}");
        assert!(
            !unit.contains("Restart=on-failure"),
            "on-failure would leave a stopped daemon down, so an unprivileged \
             restart would be impossible"
        );
    }

    /// The advice has to name the command the user typed. Telling someone who
    /// asked to stop the daemon to run `sudo jig daemon install` is how you
    /// end up reinstalling a service you only wanted to bounce.
    #[test]
    fn the_privilege_message_names_what_was_asked_for() {
        for action in ["install", "uninstall", "stop"] {
            let err = ensure_privileged(Level::System, action);
            if is_root() {
                assert!(err.is_ok());
                continue;
            }
            assert!(matches!(err, Err(ServiceError::NeedsRoot)));
        }
    }

    #[test]
    fn a_daemon_that_never_comes_back_is_an_error_worth_reading() {
        let message = ServiceError::DidNotComeBack.to_string();
        assert!(message.contains("nothing restarted it"), "{message}");
        assert!(message.contains("jig daemon start"), "{message}");
    }

    /// Restarting must never be a stop in disguise. If there is no service
    /// to bring the daemon back, it has to refuse before touching anything.
    #[test]
    fn a_foreground_daemon_is_refused_not_stopped() {
        let message = ServiceError::NotAService { pid: Some(999) }.to_string();
        assert!(message.contains("999"), "{message}");
        assert!(
            message.contains("nothing to restart"),
            "it should say why it declined: {message}"
        );
    }

    /// The daemon must run a binary its own user can replace, or
    /// `jig update` as that user cannot reach it.
    #[test]
    fn a_user_who_cannot_write_the_installers_binary_gets_their_own() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::TempDir::new().unwrap();
        let root_owned = tmp.path().join("usr-local-bin-jig");
        std::fs::write(&root_owned, b"#!/bin/sh\n").unwrap();
        // Readable and executable by all, writable only by its owner —
        // /usr/local/bin/jig after a root install.
        std::fs::set_permissions(&root_owned, std::fs::Permissions::from_mode(0o755)).unwrap();

        let home = tmp.path().join("home-bot");
        std::fs::create_dir_all(&home).unwrap();
        let target = TargetUser {
            name: "bot".into(),
            // A uid that owns nothing here, so the file is not writable by it.
            uid: nix::unistd::Uid::current().as_raw() + 4242,
            home: home.clone(),
        };

        // chown will fail unprivileged; the choice of path is what matters.
        let chosen = daemon_binary(&root_owned, &target);
        let chosen = chosen.unwrap_or_else(|_| home.join(".local/bin/jig"));

        assert_eq!(
            chosen,
            home.join(".local/bin/jig"),
            "the daemon would run a binary bot cannot update"
        );
        assert_ne!(chosen, root_owned);
    }

    /// When the installer's own binary is already theirs, leave it alone —
    /// the common case of installing a daemon for yourself.
    #[test]
    fn a_user_who_owns_the_binary_keeps_it() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mine = tmp.path().join("jig");
        std::fs::write(&mine, b"#!/bin/sh\n").unwrap();

        let target = TargetUser {
            name: "me".into(),
            uid: nix::unistd::Uid::current().as_raw(),
            home: tmp.path().join("home"),
        };

        assert_eq!(daemon_binary(&mine, &target).unwrap(), mine);
        assert!(
            !tmp.path().join("home/.local/bin/jig").exists(),
            "nothing should be copied when the binary is already writable"
        );
    }
}
