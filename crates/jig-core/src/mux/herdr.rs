//! Herdr backend for the [`Mux`] trait.
//!
//! Maps jig's group/window model onto herdr's hierarchy: the group becomes a
//! herdr *workspace* (labeled like the tmux session, e.g. `jig-<repo>`) inside
//! the user's default herdr session, and each window (branch) becomes a *tab*.
//! All control goes through the `herdr` CLI, which talks to the herdr server
//! over its local socket — so windows survive client disconnects and can be
//! reattached from any terminal, including over SSH.
//!
//! Every herdr CLI command returns JSON on stdout; responses are parsed into
//! typed structs and unexpected shapes are surfaced as errors rather than
//! silently defaulted.

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::Deserialize;

use super::{AgentState, Mux, MuxError, KNOWN_SHELLS};

const HERDR_TIMEOUT: Duration = Duration::from_secs(10);

// ── CLI response shapes ─────────────────────────────────────────────

#[derive(Deserialize)]
struct Reply<T> {
    result: T,
}

#[derive(Deserialize)]
struct WorkspaceList {
    workspaces: Vec<Workspace>,
}

#[derive(Deserialize)]
struct Workspace {
    workspace_id: String,
    label: String,
}

#[derive(Deserialize)]
struct WorkspaceCreated {
    workspace: Workspace,
}

#[derive(Deserialize)]
struct TabList {
    tabs: Vec<Tab>,
}

#[derive(Deserialize)]
struct Tab {
    tab_id: String,
    label: String,
    agent_status: AgentState,
}

#[derive(Deserialize)]
struct TabCreated {
    root_pane: Pane,
}

#[derive(Deserialize)]
struct PaneList {
    panes: Vec<Pane>,
}

#[derive(Deserialize)]
struct Pane {
    pane_id: String,
    tab_id: String,
}

#[derive(Deserialize)]
struct ProcessInfoReply {
    process_info: ProcessInfo,
}

#[derive(Deserialize)]
struct ProcessInfo {
    #[serde(default)]
    foreground_processes: Vec<ForegroundProcess>,
}

#[derive(Deserialize)]
struct ForegroundProcess {
    argv0: String,
}

// ── Backend ─────────────────────────────────────────────────────────

/// Herdr backend for the [`Mux`] trait.
///
/// Wraps a single herdr workspace — tabs map to branches.
pub struct HerdrMux {
    workspace_label: String,
}

impl HerdrMux {
    pub fn new(workspace_label: impl Into<String>) -> Self {
        Self {
            workspace_label: workspace_label.into(),
        }
    }

    fn run(&self, args: &[&str]) -> Result<Vec<u8>, MuxError> {
        let mut child = Command::new("herdr")
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let start = std::time::Instant::now();
        loop {
            match child.try_wait()? {
                Some(status) => {
                    let mut stdout = Vec::new();
                    if let Some(mut s) = child.stdout.take() {
                        io::Read::read_to_end(&mut s, &mut stdout).unwrap_or(0);
                    }
                    if !status.success() {
                        let mut stderr = Vec::new();
                        if let Some(mut s) = child.stderr.take() {
                            io::Read::read_to_end(&mut s, &mut stderr).unwrap_or(0);
                        }
                        return Err(MuxError::CommandFailed {
                            command: format!("herdr {}", args.join(" ")),
                            detail: String::from_utf8_lossy(&stderr).into_owned(),
                        });
                    }
                    return Ok(stdout);
                }
                None => {
                    if start.elapsed() >= HERDR_TIMEOUT {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(MuxError::Timeout {
                            command: format!("herdr {}", args.join(" ")),
                        });
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
    }

    fn run_json<T: serde::de::DeserializeOwned>(&self, args: &[&str]) -> Result<T, MuxError> {
        let stdout = self.run(args)?;
        let reply: Reply<T> =
            serde_json::from_slice(&stdout).map_err(|e| MuxError::CommandFailed {
                command: format!("herdr {}", args.join(" ")),
                detail: format!("unexpected JSON: {e}: {}", String::from_utf8_lossy(&stdout)),
            })?;
        Ok(reply.result)
    }

    fn find_workspace(&self) -> Result<Option<Workspace>, MuxError> {
        let list: WorkspaceList = self.run_json(&["workspace", "list"])?;
        Ok(list
            .workspaces
            .into_iter()
            .find(|w| w.label == self.workspace_label))
    }

    /// Find the workspace or create it (detached, rooted at `dir`).
    fn ensure_workspace(&self, dir: &Path) -> Result<Workspace, MuxError> {
        if let Some(ws) = self.find_workspace()? {
            return Ok(ws);
        }
        let dir_str = dir.to_string_lossy();
        let created: WorkspaceCreated = self.run_json(&[
            "workspace",
            "create",
            "--label",
            &self.workspace_label,
            "--cwd",
            &dir_str,
            "--no-focus",
        ])?;
        Ok(created.workspace)
    }

    fn find_tab(&self, name: &str) -> Result<Option<Tab>, MuxError> {
        let Some(ws) = self.find_workspace()? else {
            return Ok(None);
        };
        let list: TabList = self.run_json(&["tab", "list", "--workspace", &ws.workspace_id])?;
        Ok(list.tabs.into_iter().find(|t| t.label == name))
    }

    /// Resolve a window name to the tab's pane id (first pane in the tab).
    fn find_pane(&self, name: &str) -> Result<Option<String>, MuxError> {
        let Some(ws) = self.find_workspace()? else {
            return Ok(None);
        };
        let list: TabList = self.run_json(&["tab", "list", "--workspace", &ws.workspace_id])?;
        let Some(tab) = list.tabs.into_iter().find(|t| t.label == name) else {
            return Ok(None);
        };
        let panes: PaneList = self.run_json(&["pane", "list", "--workspace", &ws.workspace_id])?;
        Ok(panes
            .panes
            .into_iter()
            .find(|p| p.tab_id == tab.tab_id)
            .map(|p| p.pane_id))
    }

    fn pane_or_not_found(&self, name: &str) -> Result<String, MuxError> {
        self.find_pane(name)?
            .ok_or_else(|| MuxError::SessionNotFound(format!("{}:{}", self.workspace_label, name)))
    }

    /// Translate a tmux-style key name to herdr's logical key syntax.
    fn translate_key(key: &str) -> Option<String> {
        match key {
            "Enter" => Some("enter".into()),
            "Escape" | "Esc" => Some("esc".into()),
            "Tab" => Some("tab".into()),
            "Space" => Some("space".into()),
            "Up" => Some("up".into()),
            "Down" => Some("down".into()),
            "Left" => Some("left".into()),
            "Right" => Some("right".into()),
            k => k
                .strip_prefix("C-")
                .map(|rest| format!("ctrl+{}", rest.to_lowercase())),
        }
    }
}

impl Mux for HerdrMux {
    fn create_window(&self, name: &str, dir: &Path) -> Result<(), MuxError> {
        let ws = self.ensure_workspace(dir)?;
        if self.window_exists(name) {
            return Ok(());
        }
        let dir_str = dir.to_string_lossy();
        let created: TabCreated = self.run_json(&[
            "tab",
            "create",
            "--workspace",
            &ws.workspace_id,
            "--label",
            name,
            "--cwd",
            &dir_str,
            "--no-focus",
        ])?;
        // Label the pane after the branch too — the sidebar and agent list
        // show pane labels, and the default is a generic shell title.
        self.run(&["pane", "rename", &created.root_pane.pane_id, name])?;
        Ok(())
    }

    fn window_exists(&self, name: &str) -> bool {
        matches!(self.find_tab(name), Ok(Some(_)))
    }

    fn kill_window(&self, name: &str) -> Result<(), MuxError> {
        let Some(tab) = self.find_tab(name)? else {
            return Ok(());
        };
        self.run(&["tab", "close", &tab.tab_id])?;
        Ok(())
    }

    fn kill_all(&self) -> Result<(), MuxError> {
        let Some(ws) = self.find_workspace()? else {
            return Ok(());
        };
        self.run(&["workspace", "close", &ws.workspace_id])?;
        Ok(())
    }

    fn send_keys(&self, name: &str, keys: &[&str]) -> Result<(), MuxError> {
        let pane = self.pane_or_not_found(name)?;
        for key in keys {
            match Self::translate_key(key) {
                Some(logical) => {
                    self.run(&["pane", "send-keys", &pane, &logical])?;
                }
                None => {
                    self.run(&["pane", "send-text", &pane, key])?;
                }
            }
        }
        Ok(())
    }

    fn send_message(&self, name: &str, message: &str) -> Result<(), MuxError> {
        let pane = self.pane_or_not_found(name)?;
        let single_line = super::tmux::collapse_to_single_line(message);
        self.run(&["pane", "send-text", &pane, &single_line])?;
        std::thread::sleep(Duration::from_millis(100));
        self.run(&["pane", "send-keys", &pane, "enter"])?;
        Ok(())
    }

    fn is_running(&self, name: &str) -> bool {
        let Ok(Some(pane)) = self.find_pane(name) else {
            return false;
        };
        let Ok(info) =
            self.run_json::<ProcessInfoReply>(&["pane", "process-info", "--pane", &pane])
        else {
            return false;
        };
        info.process_info
            .foreground_processes
            .iter()
            .any(|p| !KNOWN_SHELLS.contains(&p.argv0.as_str()))
    }

    fn focus_window(&self, name: &str) -> Result<(), MuxError> {
        let Some(tab) = self.find_tab(name)? else {
            return Err(MuxError::SessionNotFound(format!(
                "{}:{}",
                self.workspace_label, name
            )));
        };
        self.run(&["tab", "focus", &tab.tab_id])?;
        Ok(())
    }

    fn focus(&self) -> Result<(), MuxError> {
        let Some(ws) = self.find_workspace()? else {
            return Err(MuxError::SessionNotFound(self.workspace_label.clone()));
        };
        self.run(&["workspace", "focus", &ws.workspace_id])?;
        Ok(())
    }

    /// Inside a herdr-managed pane the user's TUI already followed the
    /// focus calls — there is nothing to connect. Outside, exec-replace
    /// into the herdr client.
    fn connect(&self) -> Result<(), MuxError> {
        if std::env::var("HERDR_ENV").as_deref() == Ok("1") {
            return Ok(());
        }
        self.attach_client()
    }

    /// Herdr classifies the pane's agent as idle/working/blocked/done via
    /// lifecycle hooks or screen-manifest detection — the extra signal
    /// tmux can't provide.
    fn agent_state(&self, name: &str) -> Option<AgentState> {
        self.find_tab(name).ok().flatten().map(|t| t.agent_status)
    }
}

/// Point stdin/stdout/stderr at the controlling terminal.
///
/// Fails loudly when there is no controlling terminal: exec'ing an
/// interactive client with nowhere to draw would hang instead of erroring.
#[cfg(unix)]
fn redirect_std_to_tty() -> Result<(), MuxError> {
    use nix::unistd::{dup2_stderr, dup2_stdin, dup2_stdout};

    let tty = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .map_err(|e| MuxError::CommandFailed {
            command: "herdr".to_string(),
            detail: format!("no controlling terminal to attach to: {e}"),
        })?;

    for redirect in [dup2_stdin, dup2_stdout, dup2_stderr] {
        redirect(&tty).map_err(|e| MuxError::CommandFailed {
            command: "herdr".to_string(),
            detail: format!("failed to redirect to /dev/tty: {e}"),
        })?;
    }
    Ok(())
}

impl HerdrMux {
    /// Attach the herdr TUI client. On Unix, replaces the current process.
    ///
    /// The client renders to stdout rather than to `/dev/tty`, so it must be
    /// handed the real terminal on all three standard fds before the exec.
    /// jig is routinely run with its stdout captured — the shell integration
    /// wraps every command in `$(command jig ...)` to catch `cd` output — and
    /// a TUI drawing into that pipe looks to the user like a frozen terminal.
    #[cfg(unix)]
    fn attach_client(&self) -> Result<(), MuxError> {
        use std::ffi::CString;
        redirect_std_to_tty()?;
        let cmd = CString::new("herdr").unwrap();
        let argv = [cmd.as_c_str()];
        let err = nix::unistd::execvp(cmd.as_c_str(), &argv);
        Err(MuxError::CommandFailed {
            command: "herdr".to_string(),
            detail: format!("exec failed: {err:?}"),
        })
    }

    #[cfg(not(unix))]
    fn attach_client(&self) -> Result<(), MuxError> {
        let status = Command::new("herdr").status()?;
        if !status.success() {
            return Err(MuxError::CommandFailed {
                command: "herdr".to_string(),
                detail: "non-zero exit".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_special_keys() {
        assert_eq!(HerdrMux::translate_key("Enter").as_deref(), Some("enter"));
        assert_eq!(HerdrMux::translate_key("Escape").as_deref(), Some("esc"));
        assert_eq!(HerdrMux::translate_key("C-c").as_deref(), Some("ctrl+c"));
    }

    #[test]
    fn plain_text_is_not_a_key() {
        assert_eq!(HerdrMux::translate_key("claude --resume abc"), None);
        assert_eq!(HerdrMux::translate_key("echo hi"), None);
    }

    #[test]
    fn agent_status_parses() {
        let s: AgentState = serde_json::from_str("\"blocked\"").unwrap();
        assert_eq!(s, AgentState::Blocked);
    }
}
