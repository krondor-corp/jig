//! Running external commands under a deadline.
//!
//! `std::process` has no timeout: `Command::output()` and `Child::wait()` block
//! until the child decides to exit. That is fine for a person at a terminal who
//! can press ctrl-c. It is not fine inside the daemon, whose actors have
//! single-slot queues — one call that never returns stops that actor for the
//! life of the process, with nothing in the logs to say why.
//!
//! So jig does not call `Command::output()`. Everything it shells out to goes
//! through [`Exec`], which always has a deadline, and [`Hook`] is the shape a
//! user-configured command takes in `jig.toml`.

use std::ffi::OsStr;
use std::fmt;
use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use nix::sys::signal::{killpg, Signal};
use nix::unistd::Pid;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize, Serializer};

/// How often to check whether the child has exited.
const POLL: Duration = Duration::from_millis(100);

/// How long a killed process group gets to exit on `SIGTERM` before `SIGKILL`.
const GRACE: Duration = Duration::from_secs(2);

// ── Timeout ──────────────────────────────────────────────────────────

/// How long a command may run before it is killed.
///
/// In config this is a number of seconds, or `"none"` for no limit:
///
/// ```toml
/// timeout = 900
/// timeout = "none"
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timeout {
    After(Duration),
    /// No limit. For commands where hanging is the user's problem and they can
    /// press ctrl-c — never for anything the daemon runs unattended.
    Unlimited,
}

impl Timeout {
    /// Generous on purpose. Omitting a timeout must not kill a legitimately
    /// slow command; callers that know better narrow it themselves.
    pub const DEFAULT: Self = Self::secs(600);
    /// For talking to a server that may accept a connection and then go quiet.
    pub const NETWORK: Self = Self::secs(60);
    /// For a local command that should answer immediately, like `--version`.
    pub const QUICK: Self = Self::secs(10);

    pub const fn secs(n: u64) -> Self {
        Self::After(Duration::from_secs(n))
    }

    /// When a command started now would run out of time, or `None` if never.
    fn deadline(&self) -> Option<Instant> {
        match self {
            Self::After(d) => Some(Instant::now() + *d),
            Self::Unlimited => None,
        }
    }
}

impl Default for Timeout {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for Timeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::After(d) => write!(f, "{}s", d.as_secs()),
            Self::Unlimited => f.write_str("no limit"),
        }
    }
}

impl Serialize for Timeout {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::After(d) => s.serialize_u64(d.as_secs()),
            Self::Unlimited => s.serialize_str("none"),
        }
    }
}

impl<'de> Deserialize<'de> for Timeout {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Secs(u64),
            Word(String),
        }

        match Repr::deserialize(d)? {
            // Zero is what people reach for to mean "off"; a zero-second
            // deadline would be useless otherwise.
            Repr::Secs(0) => Ok(Self::Unlimited),
            Repr::Secs(n) => Ok(Self::secs(n)),
            Repr::Word(w) if matches!(w.as_str(), "none" | "never") => Ok(Self::Unlimited),
            Repr::Word(w) => Err(de::Error::custom(format!(
                r#"expected a number of seconds or "none", got {w:?}"#
            ))),
        }
    }
}

// ── Hook ─────────────────────────────────────────────────────────────

/// A command as the user configures it.
///
/// Deserializes from a bare string, or from a table that also sets a timeout:
///
/// ```toml
/// on_create = "pnpm install"
///
/// [worktree.on_create]
/// command = "pnpm install --frozen-lockfile"
/// timeout = 1800
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "HookRepr", into = "HookRepr")]
pub struct Hook {
    pub command: String,
    pub timeout: Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum HookRepr {
    Bare(String),
    Table {
        command: String,
        #[serde(default)]
        timeout: Timeout,
    },
}

impl From<HookRepr> for Hook {
    fn from(repr: HookRepr) -> Self {
        match repr {
            HookRepr::Bare(command) => Self {
                command,
                timeout: Timeout::default(),
            },
            HookRepr::Table { command, timeout } => Self { command, timeout },
        }
    }
}

impl From<Hook> for HookRepr {
    fn from(hook: Hook) -> Self {
        // Round-trip the short form when there is nothing extra to say.
        if hook.timeout == Timeout::default() {
            Self::Bare(hook.command)
        } else {
            Self::Table {
                command: hook.command,
                timeout: hook.timeout,
            }
        }
    }
}

impl Hook {
    pub fn new(command: impl Into<String>, timeout: Timeout) -> Self {
        Self {
            command: command.into(),
            timeout,
        }
    }

    /// Ready to run. `label` is what a failure calls this hook.
    pub fn exec(&self, label: &str) -> Exec {
        Exec::shell(&self.command)
            .labeled(label)
            .timeout(self.timeout)
    }
}

// ── Errors and output ────────────────────────────────────────────────

/// A command that never produced an exit status.
///
/// A command that ran and exited is not an error however it exited — callers
/// disagree about what a non-zero status means (`gh auth status` failing is
/// just "not logged in"), so the status comes back as data in [`Output`].
#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("failed to run {label}: {source}")]
    Spawn {
        label: String,
        #[source]
        source: std::io::Error,
    },

    #[error("{label} timed out after {timeout} and was killed{}", trailing(.stderr))]
    TimedOut {
        label: String,
        timeout: Timeout,
        /// Whatever it printed before it hung — often the only clue as to
        /// where it got stuck.
        stderr: String,
    },
}

/// Renders captured output as a suffix, or nothing when there was none.
fn trailing(stderr: &str) -> String {
    if stderr.is_empty() {
        String::new()
    } else {
        format!(". Last output: {stderr}")
    }
}

/// A command that ran to completion, whatever its status.
#[derive(Debug)]
pub struct Output {
    pub status: ExitStatus,
    /// Empty unless the command was built with [`Exec::capturing`].
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    pub fn success(&self) -> bool {
        self.status.success()
    }

    /// Why it failed, for an error message — its stderr, else its status.
    ///
    /// A command that fails silently used to produce messages that trailed off
    /// after the colon.
    pub fn failure(&self) -> String {
        match self.stderr.as_str() {
            "" => match self.status.code() {
                Some(code) => format!("exited with status {code}, no output"),
                None => "killed by a signal, no output".to_string(),
            },
            stderr => stderr.to_string(),
        }
    }
}

// ── Exec ─────────────────────────────────────────────────────────────

/// An external command and the deadline it has to finish within.
pub struct Exec {
    command: Command,
    label: String,
    timeout: Timeout,
    stdin: Option<Vec<u8>>,
    capture_stdout: bool,
}

impl Exec {
    /// Run a shell one-liner — the form jig's configurable commands take.
    pub fn shell(script: &str) -> Self {
        let mut command = Command::new("sh");
        command.args(["-c", script]);
        Self::command(command)
    }

    /// Run a command the caller has built (args, env, cwd).
    pub fn command(command: Command) -> Self {
        let label = command.get_program().to_string_lossy().to_string();
        Self {
            command,
            label,
            timeout: Timeout::default(),
            stdin: None,
            capture_stdout: false,
        }
    }

    /// What to call this in an error. Defaults to the program name, which is
    /// no use when everything is `sh` — name the thing the user configured.
    pub fn labeled(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn timeout(mut self, timeout: Timeout) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn in_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.command.current_dir(dir);
        self
    }

    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.command.arg(arg);
        self
    }

    /// Write `bytes` to the child's stdin, then close it.
    pub fn stdin(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.stdin = Some(bytes.into());
        self
    }

    /// Keep stdout. Off by default: a cold `pnpm install` emits megabytes, and
    /// buffering that to find out whether it exited zero is pure waste.
    pub fn capturing(mut self) -> Self {
        self.capture_stdout = true;
        self
    }

    /// Run it, killing it if it outruns its deadline.
    pub fn run(mut self) -> Result<Output, ExecError> {
        self.command
            .stdin(if self.stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(if self.capture_stdout {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stderr(Stdio::piped())
            // Its own process group, so a timeout can take the whole tree
            // down. See `kill_group`.
            .process_group(0);

        let mut child = self.command.spawn().map_err(|source| ExecError::Spawn {
            label: self.label.clone(),
            source,
        })?;
        let group = Pid::from_raw(child.id() as i32);

        // Every pipe gets its own thread. A child that fills a pipe nobody
        // reads blocks on write, and a child nobody feeds blocks on read —
        // either deadlocks against the wait below, reintroducing the hang
        // this type exists to prevent.
        let feeding = self.stdin.take().map(|bytes| {
            let mut pipe = child.stdin.take();
            std::thread::spawn(move || {
                if let Some(pipe) = pipe.as_mut() {
                    let _ = pipe.write_all(&bytes);
                }
                // Dropping the pipe closes it, so the child sees EOF.
            })
        });
        let reading_out = drain(child.stdout.take());
        let reading_err = drain(child.stderr.take());

        let status = match self.timeout.deadline() {
            // No deadline: nothing to poll for, so just wait.
            None => Some(child.wait().map_err(|source| ExecError::Spawn {
                label: self.label.clone(),
                source,
            })?),
            Some(deadline) => loop {
                match child.try_wait() {
                    Ok(Some(status)) => break Some(status),
                    Ok(None) if Instant::now() >= deadline => {
                        kill_group(group);
                        // Reap it so it doesn't linger as a zombie.
                        let _ = child.wait();
                        break None;
                    }
                    Ok(None) => std::thread::sleep(POLL),
                    Err(source) => {
                        return Err(ExecError::Spawn {
                            label: self.label,
                            source,
                        })
                    }
                }
            },
        };

        // Every write end is closed once the child (or its group) is gone,
        // so these finish.
        if let Some(feeding) = feeding {
            let _ = feeding.join();
        }
        let stdout = join(reading_out);
        let stderr = join(reading_err);

        match status {
            Some(status) => Ok(Output {
                status,
                stdout,
                stderr,
            }),
            None => Err(ExecError::TimedOut {
                label: self.label,
                timeout: self.timeout,
                stderr,
            }),
        }
    }
}

/// Read a pipe to EOF on its own thread.
fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> Option<std::thread::JoinHandle<Vec<u8>>> {
    let mut pipe = pipe?;
    Some(std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        buf
    }))
}

fn join(handle: Option<std::thread::JoinHandle<Vec<u8>>>) -> String {
    handle
        .and_then(|h| h.join().ok())
        .map(|b| String::from_utf8_lossy(&b).trim().to_string())
        .unwrap_or_default()
}

/// Kill the whole process group, not just the child we spawned.
///
/// A configured command is `sh -c "..."`, so the thing that actually hangs is
/// usually a grandchild — the `pnpm` under the `sh`. Killing only the shell
/// leaves that grandchild running *and* holding the stderr pipe open, so the
/// draining thread never reaches EOF and joining it hangs forever. Signalling
/// the group is what makes the deadline real.
fn kill_group(group: Pid) {
    // Politeness first: something mid-write to a lockfile gets a chance to
    // clean up. Then insistence, because a hung process may ignore SIGTERM.
    let _ = killpg(group, Signal::SIGTERM);
    let deadline = Instant::now() + GRACE;
    while Instant::now() < deadline {
        // ESRCH means the group is gone, which is what we were after.
        if killpg(group, None).is_err() {
            return;
        }
        std::thread::sleep(POLL);
    }
    let _ = killpg(group, Signal::SIGKILL);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh(script: &str) -> Exec {
        Exec::shell(script).timeout(Timeout::secs(30))
    }

    // ── Timeout ──

    #[derive(Debug, Deserialize)]
    struct Cfg {
        #[serde(default)]
        timeout: Timeout,
    }

    #[test]
    fn a_timeout_reads_as_seconds() {
        let cfg: Cfg = toml::from_str("timeout = 900").unwrap();
        assert_eq!(cfg.timeout, Timeout::secs(900));
    }

    #[test]
    fn an_omitted_timeout_is_the_default() {
        let cfg: Cfg = toml::from_str("").unwrap();
        assert_eq!(cfg.timeout, Timeout::DEFAULT);
    }

    #[test]
    fn none_and_zero_both_mean_unlimited() {
        for src in [r#"timeout = "none""#, r#"timeout = "never""#, "timeout = 0"] {
            let cfg: Cfg = toml::from_str(src).unwrap();
            assert_eq!(cfg.timeout, Timeout::Unlimited, "for {src}");
        }
    }

    #[test]
    fn a_nonsense_timeout_says_what_was_expected() {
        let err = toml::from_str::<Cfg>(r#"timeout = "soon""#).unwrap_err();
        assert!(err.to_string().contains("number of seconds"), "{err}");
    }

    // ── Hook ──

    #[derive(Debug, Deserialize)]
    struct WithHook {
        on_create: Hook,
    }

    #[test]
    fn a_hook_can_be_a_bare_string() {
        let cfg: WithHook = toml::from_str(r#"on_create = "pnpm install""#).unwrap();
        assert_eq!(cfg.on_create.command, "pnpm install");
        assert_eq!(cfg.on_create.timeout, Timeout::DEFAULT);
    }

    #[test]
    fn a_hook_can_carry_its_own_timeout() {
        let cfg: WithHook = toml::from_str(
            r#"
[on_create]
command = "pnpm install"
timeout = 1800
"#,
        )
        .unwrap();
        assert_eq!(cfg.on_create.command, "pnpm install");
        assert_eq!(cfg.on_create.timeout, Timeout::secs(1800));
    }

    #[test]
    fn a_hook_can_opt_out_of_a_deadline() {
        let cfg: WithHook = toml::from_str(
            r#"
[on_create]
command = "./slow-thing.sh"
timeout = "none"
"#,
        )
        .unwrap();
        assert_eq!(cfg.on_create.timeout, Timeout::Unlimited);
    }

    // ── Exec ──

    #[test]
    fn a_command_that_exits_cleanly_succeeds() {
        assert!(sh("true").run().unwrap().success());
    }

    #[test]
    fn a_nonzero_exit_is_output_not_an_error() {
        // Callers disagree about what failure means, so they decide.
        let out = sh("echo boom >&2; exit 3").run().unwrap();
        assert!(!out.success());
        assert_eq!(out.stderr, "boom");
        assert_eq!(out.status.code(), Some(3));
    }

    #[test]
    fn stdout_is_discarded_unless_asked_for() {
        assert_eq!(sh("echo hello").run().unwrap().stdout, "");
        assert_eq!(sh("echo hello").capturing().run().unwrap().stdout, "hello");
    }

    #[test]
    fn a_missing_binary_is_a_spawn_error() {
        let err = Exec::command(Command::new("definitely-not-a-real-binary-xyz"))
            .run()
            .unwrap_err();
        assert!(matches!(err, ExecError::Spawn { .. }), "{err:?}");
    }

    #[test]
    fn stdin_is_delivered_and_closed() {
        // `cat` only exits once it sees EOF, so this also proves we close it.
        let out = sh("cat").stdin("hello stdin").capturing().run().unwrap();
        assert_eq!(out.stdout, "hello stdin");
    }

    #[test]
    fn a_chatty_command_does_not_deadlock() {
        // A child filling a pipe nobody reads blocks on write.
        let out = sh("yes hello | head -c 2000000 >&2").run().unwrap();
        assert!(out.success());
    }

    // ── Deadlines ──

    #[test]
    fn a_command_that_outruns_its_deadline_is_killed() {
        let start = Instant::now();
        let err = Exec::shell("sleep 60")
            .timeout(Timeout::secs(1))
            .run()
            .unwrap_err();

        assert!(matches!(err, ExecError::TimedOut { .. }), "{err:?}");
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "took {:?}, so it was not actually killed",
            start.elapsed()
        );
    }

    /// The reason [`kill_group`] exists. A configured command is `sh -c`, so
    /// what hangs is the grandchild — and it holds the stderr pipe open, so
    /// killing only the shell leaves the drain thread waiting on EOF forever.
    #[test]
    fn a_hung_grandchild_does_not_outlive_the_deadline() {
        let start = Instant::now();
        let err = Exec::shell("sleep 60 & wait")
            .timeout(Timeout::secs(1))
            .run()
            .unwrap_err();

        assert!(matches!(err, ExecError::TimedOut { .. }), "{err:?}");
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "took {:?} — the grandchild kept the pipe open",
            start.elapsed()
        );
    }

    #[test]
    fn a_timeout_reports_what_it_managed_to_print() {
        let err = Exec::shell("echo 'cloning...' >&2; sleep 60")
            .labeled("on_create hook")
            .timeout(Timeout::secs(1))
            .run()
            .unwrap_err();

        let message = err.to_string();
        assert!(message.contains("on_create hook"), "{message}");
        assert!(message.contains("cloning..."), "{message}");
    }

    #[test]
    fn an_unlimited_command_is_not_killed() {
        let out = Exec::shell("sleep 0.2; echo done")
            .timeout(Timeout::Unlimited)
            .capturing()
            .run()
            .unwrap();
        assert_eq!(out.stdout, "done");
    }

    #[test]
    fn a_label_defaults_to_the_program() {
        let err = Exec::command(Command::new("definitely-not-a-real-binary-xyz"))
            .run()
            .unwrap_err();
        assert!(
            err.to_string().contains("definitely-not-a-real-binary-xyz"),
            "{err}"
        );
    }

    #[test]
    fn a_silent_failure_still_says_why() {
        let out = sh("exit 3").run().unwrap();
        assert!(out.failure().contains('3'), "{}", out.failure());
    }
}
