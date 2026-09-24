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

use error::Result;

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
