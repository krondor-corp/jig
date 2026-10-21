//! `jig init`.

use crate::common::Sandbox;
use jig_cli::context::RepoRegistry;
use predicates::prelude::*;

#[test]
fn init_registers_the_repo_for_the_daemon() {
    let sandbox = Sandbox::with_repo();
    // Agent hooks install under `~/.claude`; keep them out of the real home.
    let home = tempfile::TempDir::new().unwrap();

    sandbox
        .jig()
        .env("HOME", home.path())
        .args(["init", "claude"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Registered repo"));

    let registry = RepoRegistry::load(&sandbox.paths()).unwrap();
    let repo = std::fs::canonicalize(sandbox.repo(0)).unwrap();
    assert!(
        registry.repos().iter().any(|e| e.path == repo),
        "repos.json should contain {}: {:?}",
        repo.display(),
        registry.repos()
    );
}
