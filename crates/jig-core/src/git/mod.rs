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

use crate::exec::Timeout;

use error::Result;

/// Bound how long libgit2 will wait on a remote.
///
/// libgit2 ships with no network timeouts at all — its default reads back as
/// 0, meaning wait forever — so `fetch` and `push_branch` block indefinitely
/// against a server that accepts the connection and then stops talking.
///
/// For someone at a terminal that is an annoyance they can ctrl-c. In the
/// daemon it is fatal: the spawn actor handles one request at a time and drops
/// the rest while one is in flight, so a wedged push stops auto-spawn for the
/// life of the process while `jig daemon status` still reports healthy.
///
/// Both settings reach the HTTPS *and* SSH transports — they share libgit2's
/// socket stream, and git2's `ssh` feature is on.
///
/// Unlike [`crate::exec::Exec`], this cannot be per-call: libgit2 exposes the
/// timeout only as a process-wide setting, with nothing on `FetchOptions` or
/// `PushOptions`. Set once from `main`, **before any thread is spawned** —
/// these write C globals with no synchronization.
///
/// Failures are ignored: a jig that cannot set a timeout should still run,
/// exactly as it did before this existed.
pub fn set_network_timeouts(connect: Timeout, idle: Timeout) {
    // `c_int` is `i32` on every platform jig builds for. libgit2 reads 0 as
    // "wait forever", which is exactly `Timeout::Unlimited`.
    fn millis(timeout: Timeout) -> i32 {
        match timeout {
            Timeout::After(d) => d.as_millis().min(i32::MAX as u128) as i32,
            Timeout::Unlimited => 0,
        }
    }

    unsafe {
        let _ = git2::opts::set_server_connect_timeout_in_milliseconds(millis(connect));
        let _ = git2::opts::set_server_timeout_in_milliseconds(millis(idle));
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
