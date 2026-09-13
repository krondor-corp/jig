mod common;

use common::Sandbox;
use predicates::prelude::*;

// ============================================================================
// Attach Auto-Detection Tests
// ============================================================================

#[test]
fn test_attach_outside_repo_requires_branch() {
    Sandbox::new()
        .jig()
        .args(["attach"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nothing to target"));
}

#[test]
fn test_attach_outside_repo_nonexistent_worktree() {
    Sandbox::new()
        .jig()
        .args(["attach", "nonexistent-worker"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
