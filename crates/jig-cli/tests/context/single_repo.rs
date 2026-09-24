//! Building a context for a repo, and what it records globally.

use crate::common::Sandbox;
use jig_cli::context::{Config, Context, RepoConfig, RepoRegistry, JIG_DIR};

#[test]
fn test_single_repo_context_registers_the_repo() {
    let sandbox = Sandbox::with_repos(1);
    let paths = sandbox.paths();
    let dir = sandbox.repo(0);

    let repo = RepoConfig::from_path(dir).unwrap();
    let ctx = Context::for_repo(&paths, repo, Config::default());

    let repo = ctx.repo().unwrap();
    assert_eq!(
        repo.repo_root.canonicalize().unwrap(),
        dir.canonicalize().unwrap()
    );
    assert!(repo.worktrees_path.ends_with(JIG_DIR));
    assert!(repo.session_name().starts_with("jig-"));
    assert_eq!(repo.base_branch(&ctx.config), "origin/main");

    // The context's own registry holds just this repo...
    assert_eq!(
        ctx.registry.repos().len(),
        1,
        "registry returned by from_cwd should contain the current repo"
    );
    assert_eq!(
        ctx.registry.repos()[0].path.canonicalize().unwrap(),
        dir.canonicalize().unwrap()
    );

    // ...and it is persisted globally so -g commands and the daemon see it.
    let global = RepoRegistry::load(&paths).unwrap();
    assert_eq!(global.repos().len(), 1);
    assert_eq!(
        global.repos()[0].path.canonicalize().unwrap(),
        dir.canonicalize().unwrap()
    );
}

#[test]
fn test_registry_no_duplicate_on_repeated_register() {
    let dir = tempfile::tempdir().unwrap();
    let repo_path = dir.path().to_path_buf();

    let mut registry = RepoRegistry::default();
    let added_first = registry.register(repo_path.clone());
    let added_second = registry.register(repo_path.clone());

    assert!(added_first, "first register should report newly added");
    assert!(
        !added_second,
        "second register should report already present"
    );
    assert_eq!(
        registry.repos().len(),
        1,
        "repeated register must not duplicate entries"
    );
}

#[test]
fn test_base_branch_from_jig_toml() {
    let sandbox = Sandbox::with_repos(1);
    let dir = sandbox.repo(0);
    std::fs::write(
        dir.join("jig.toml"),
        "[worktree]\nbase = \"origin/develop\"\n",
    )
    .unwrap();

    let repo = RepoConfig::from_path(dir).unwrap();
    assert_eq!(repo.base_branch(&Config::default()), "origin/develop");
}
