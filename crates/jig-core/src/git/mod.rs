//! Git operations — opinionated module built on top of git2.
//!
//! - [`Repo`] wraps `git2::Repository` with jig-specific operations.
//! - [`Worktree`] / [`WorktreeRef`] represent resolved and serializable
//!   worktree handles respectively.

mod branch;
mod commit;
mod diff;
mod error;
mod repo;
mod worktree;

pub use branch::Branch;
pub use commit::conventional;
pub use commit::Oid;
pub use diff::{Diff, FileDiff, Stats as DiffStats};
pub use error::GitError;
pub use repo::Repo;
pub use worktree::{Worktree, WorktreeRef};

pub const WORKTREES_DIR: &str = ".jig";

use std::io::Write;
use std::path::Path;
use std::time::Duration;

use error::Result;

/// How long to wait for a remote to accept a connection.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// How long to wait on a remote that accepted the connection and then went
/// quiet. Generous enough for a slow clone to keep streaming.
const IDLE_TIMEOUT: Duration = Duration::from_secs(60);

/// Bound how long libgit2 will wait on a remote.
///
/// libgit2 ships with no network timeouts, so `fetch` and `push_branch` block
/// forever against a server that accepts a connection and then stops talking.
/// In the daemon that is fatal rather than annoying: the spawn actor handles
/// one request at a time, so a wedged push stops auto-spawn for the life of
/// the process while `jig daemon status` still reports healthy.
///
/// Both settings reach the HTTPS and SSH transports — they share libgit2's
/// socket stream, and `git2`'s `ssh` feature is on.
///
/// # Panics
///
/// Never. Failures are ignored: a jig that cannot set a timeout should still
/// run, just as it did before this existed.
///
/// Call once from `main`, **before any thread is spawned** — these write C
/// globals with no synchronization.
pub fn set_network_timeouts() {
    // `c_int` is `i32` on every platform jig builds for.
    unsafe {
        let _ = git2::opts::set_server_connect_timeout_in_milliseconds(
            CONNECT_TIMEOUT.as_millis() as i32
        );
        let _ = git2::opts::set_server_timeout_in_milliseconds(IDLE_TIMEOUT.as_millis() as i32);
    }
}

// ---------------------------------------------------------------------------
// Free functions (filesystem-only, no git2)
// ---------------------------------------------------------------------------

/// Ensure `dir_name` is listed in the repository's local exclude file.
pub fn ensure_excluded(git_common_dir: &Path, dir_name: &str) -> Result<()> {
    let exclude_file = git_common_dir.join("info").join("exclude");
    let exclude_entry = format!("{}/", dir_name);

    if !exclude_file.exists() {
        std::fs::create_dir_all(exclude_file.parent().unwrap())?;
        std::fs::write(&exclude_file, format!("{}\n", exclude_entry))?;
        return Ok(());
    }

    let content = std::fs::read_to_string(&exclude_file)?;
    if !content.contains(dir_name) {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&exclude_file)?;
        writeln!(file, "{}", exclude_entry)?;
    }

    Ok(())
}
