//! Commands that span every tracked repo.

mod common;

use common::Sandbox;
use predicates::prelude::*;

/// Base worktrees on the local `main` — sandbox repos have no `origin`.
fn use_local_main(repo: &std::path::Path) {
    std::fs::write(repo.join("jig.toml"), "[worktree]\nbase = \"main\"\n").unwrap();
}

#[test]
fn global_list_shows_worktrees_from_every_repo() {
    let sandbox = Sandbox::with_repos(2);
    for (i, name) in [(0, "feat-a"), (1, "feat-b")] {
        let repo = sandbox.repo(i);
        use_local_main(repo);
        // Running in a repo also registers it for `-g` commands.
        sandbox
            .jig_in(repo)
            .args(["create", name])
            .assert()
            .success();
    }

    sandbox
        .jig()
        .args(["ls", "-g", "--plain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feat-a").and(predicate::str::contains("feat-b")));

    // Repo-scoped listing stays scoped.
    sandbox
        .jig_in(sandbox.repo(1))
        .args(["ls", "--plain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feat-b").and(predicate::str::contains("feat-a").not()));
}
