//! Removing jig's git hooks, restoring anything it backed up.

use jig_cli::hooks::install::init_hooks;
use jig_cli::hooks::registry::{self, HookRegistry};
use jig_cli::hooks::uninstall::uninstall_hooks;
use jig_cli::hooks::MANAGED_HOOKS;

fn fake_repo() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().to_path_buf();
    std::fs::create_dir_all(repo.join(".git/hooks")).unwrap();
    (tmp, repo)
}

#[test]
fn uninstall_removes_all_hooks() {
    let (_tmp, repo) = fake_repo();
    init_hooks(&repo, false).unwrap();

    let result = uninstall_hooks(&repo, None).unwrap();
    assert_eq!(result.outcomes.len(), MANAGED_HOOKS.len());

    // Hooks should be gone
    for name in MANAGED_HOOKS {
        assert!(!repo.join(".git/hooks").join(name).exists());
    }

    // Registry should be gone
    assert!(!registry::registry_path(&repo).exists());
}

#[test]
fn uninstall_restores_user_hooks() {
    let (_tmp, repo) = fake_repo();
    let hook_path = repo.join(".git/hooks/post-commit");
    std::fs::write(&hook_path, "#!/bin/bash\necho 'original'").unwrap();

    init_hooks(&repo, false).unwrap();

    // Verify user hook was moved
    assert!(repo.join(".git/hooks/post-commit.user").exists());

    uninstall_hooks(&repo, None).unwrap();

    // Original hook should be restored
    let content = std::fs::read_to_string(&hook_path).unwrap();
    assert!(content.contains("echo 'original'"));

    // .user file should be cleaned up
    assert!(!repo.join(".git/hooks/post-commit.user").exists());
}

#[test]
fn uninstall_specific_hook() {
    let (_tmp, repo) = fake_repo();
    init_hooks(&repo, false).unwrap();

    uninstall_hooks(&repo, Some("post-commit")).unwrap();

    // post-commit gone, others remain
    assert!(!repo.join(".git/hooks/post-commit").exists());
    assert!(repo.join(".git/hooks/post-merge").exists());
    assert!(repo.join(".git/hooks/pre-commit").exists());

    // Registry still exists with remaining hooks
    let registry = HookRegistry::load(&repo).unwrap();
    assert!(!registry.is_installed("post-commit"));
    assert!(registry.is_installed("post-merge"));
}

#[test]
fn uninstall_is_idempotent() {
    let (_tmp, repo) = fake_repo();
    init_hooks(&repo, false).unwrap();

    uninstall_hooks(&repo, None).unwrap();
    // Second uninstall should be a no-op
    let result = uninstall_hooks(&repo, None).unwrap();
    assert!(result.outcomes.is_empty());
}

#[test]
fn uninstall_handles_missing_hooks() {
    let (_tmp, repo) = fake_repo();
    init_hooks(&repo, false).unwrap();

    // Manually delete a hook file
    std::fs::remove_file(repo.join(".git/hooks/post-commit")).unwrap();

    // Uninstall should still work
    let result = uninstall_hooks(&repo, None).unwrap();
    assert_eq!(result.outcomes.len(), MANAGED_HOOKS.len());
}
