//! Git hook handler implementations.
//!
//! Called by `jig hooks <name>` when git hook wrappers fire.
//! Each handler emits events to the worker's event log.

use std::path::Path;

use crate::context::RepoCtx;
use crate::worker::events::{Event, EventKind};
use crate::worker::Worker;

use super::HookError;
use jig_core::git::conventional::ValidationConfig;
use jig_core::git::Worktree;

/// Handle post-commit hook: emit a Commit event with the HEAD SHA.
///
/// Silently does nothing if `dir` is not a jig-managed worktree.
pub fn handle_post_commit(repo: &RepoCtx, dir: &Path) -> Result<(), HookError> {
    let Some(wt) = worktree_at(dir) else {
        return Ok(());
    };

    let worker = Worker::open(repo, &wt.branch_name())?;
    worker.event_log().append(&Event::now(EventKind::Commit {
        sha: wt.head_sha().unwrap_or_default(),
        repo: worker.repo_name(),
    }))?;

    Ok(())
}

/// Handle post-merge hook: emit a Push event.
pub fn handle_post_merge(repo: &RepoCtx, dir: &Path) -> Result<(), HookError> {
    let Some(wt) = worktree_at(dir) else {
        return Ok(());
    };

    let worker = Worker::open(repo, &wt.branch_name())?;
    worker.event_log().append(&Event::now(EventKind::Push {
        sha: wt.head_sha().unwrap_or_default(),
        repo: worker.repo_name(),
    }))?;

    Ok(())
}

/// Handle commit-msg hook: validate the commit message against conventional commits spec.
///
/// Reads the commit message from the file path provided by git (the first argument).
/// If a `[commits]` section exists in `jig.toml`, validates against those rules.
/// Returns an error (blocking the commit) if validation fails.
pub fn handle_commit_msg(_repo_path: &Path, commit_msg_file: &str) -> Result<(), HookError> {
    let message = std::fs::read_to_string(commit_msg_file)?;

    // Strip git comment lines (lines starting with #)
    let cleaned: String = message
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return Ok(());
    }

    let config = ValidationConfig::default();

    match config.parse_and_validate(cleaned) {
        Ok((_msg, errors)) => {
            if errors.is_empty() {
                Ok(())
            } else {
                let msgs: Vec<String> = errors.iter().map(|e| format!("  {}", e)).collect();
                Err(super::HookError::Validation(format!(
                    "commit message does not follow conventional commits:\n{}\n\n  \
                     Run `jig commit examples` for help.",
                    msgs.join("\n"),
                )))
            }
        }
        Err(e) => Err(super::HookError::Validation(format!(
            "commit message does not follow conventional commits:\n  {}\n\n  \
             Run `jig commit examples` for help.",
            e,
        ))),
    }
}

/// Handle pre-commit hook: currently a no-op.
pub fn handle_pre_commit(_repo_path: &Path) -> Result<(), HookError> {
    Ok(())
}

/// The jig worktree at `dir`, if there is one.
fn worktree_at(dir: &Path) -> Option<Worktree> {
    Worktree::open(dir).ok()
}
