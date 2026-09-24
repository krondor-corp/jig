//! Installing jig's git hooks into a repo.

use jig_cli::hooks::install::{init_hooks, should_install_hook, HookResult, InstallDecision};
use jig_cli::hooks::registry::HookRegistry;
use jig_cli::hooks::{is_jig_managed, MANAGED_HOOKS};

/// Create a minimal fake repo directory with `.git/hooks/`.
fn fake_repo() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().to_path_buf();
    std::fs::create_dir_all(repo.join(".git/hooks")).unwrap();
    (tmp, repo)
}

#[test]
fn init_fresh_repo() {
    let (_tmp, repo) = fake_repo();
    let result = init_hooks(&repo, false).unwrap();
    assert_eq!(result.results.len(), MANAGED_HOOKS.len());

    for name in MANAGED_HOOKS {
        let path = repo.join(".git/hooks").join(name);
        assert!(path.exists(), "{} not created", name);

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(is_jig_managed(&content));
    }
}

#[test]
fn init_is_idempotent() {
    let (_tmp, repo) = fake_repo();

    init_hooks(&repo, false).unwrap();
    let result = init_hooks(&repo, false).unwrap();

    assert!(result
        .results
        .iter()
        .all(|r| matches!(r, HookResult::AlreadyInstalled(_))));
}

#[test]
fn init_backs_up_user_hooks() {
    let (_tmp, repo) = fake_repo();
    let hook_path = repo.join(".git/hooks/post-commit");
    std::fs::write(&hook_path, "#!/bin/bash\necho 'user hook'").unwrap();

    init_hooks(&repo, false).unwrap();

    // User hook moved to .user
    let user_path = repo.join(".git/hooks/post-commit.user");
    assert!(user_path.exists());
    let user_content = std::fs::read_to_string(&user_path).unwrap();
    assert!(user_content.contains("user hook"));

    // Jig hook installed
    let installed_content = std::fs::read_to_string(&hook_path).unwrap();
    assert!(is_jig_managed(&installed_content));

    // Backup exists
    let entries: Vec<_> = std::fs::read_dir(repo.join(".git/hooks"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("post-commit.backup-")
        })
        .collect();
    assert_eq!(entries.len(), 1);

    // Registry tracks backup
    let registry = HookRegistry::load(&repo).unwrap();
    let entry = &registry.installed["post-commit"];
    assert!(entry.had_existing);
    assert!(entry.backed_up_to.is_some());
}

#[test]
fn init_force_reinstalls() {
    let (_tmp, repo) = fake_repo();

    init_hooks(&repo, false).unwrap();
    let result = init_hooks(&repo, true).unwrap();

    assert!(result
        .results
        .iter()
        .all(|r| matches!(r, HookResult::Installed(_))));
}

#[test]
fn hooks_are_executable() {
    let (_tmp, repo) = fake_repo();
    init_hooks(&repo, false).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in MANAGED_HOOKS {
            let path = repo.join(".git/hooks").join(name);
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert!(mode & 0o111 != 0, "{} not executable", name);
        }
    }
}

#[test]
fn registry_saved_after_init() {
    let (_tmp, repo) = fake_repo();
    init_hooks(&repo, false).unwrap();

    let registry = HookRegistry::load(&repo).unwrap();
    for name in MANAGED_HOOKS {
        assert!(registry.is_installed(name), "{} not in registry", name);
    }
}

#[test]
fn should_install_decisions() {
    let (_tmp, repo) = fake_repo();
    let hooks_dir = repo.join(".git/hooks");
    let registry = HookRegistry::new();

    // No file → Install
    let hook_path = hooks_dir.join("post-commit");
    assert_eq!(
        should_install_hook(&hook_path, &registry, "post-commit", false).unwrap(),
        InstallDecision::Install
    );

    // Force → Reinstall
    assert_eq!(
        should_install_hook(&hook_path, &registry, "post-commit", true).unwrap(),
        InstallDecision::Reinstall
    );

    // User hook → BackupAndInstall
    std::fs::write(&hook_path, "#!/bin/bash\necho 'user'").unwrap();
    assert_eq!(
        should_install_hook(&hook_path, &registry, "post-commit", false).unwrap(),
        InstallDecision::BackupAndInstall
    );

    // Jig-managed but not in registry → UpdateRegistry
    std::fs::write(
        &hook_path,
        "#!/bin/bash\n# jig-managed: v1\njig hooks post-commit",
    )
    .unwrap();
    assert_eq!(
        should_install_hook(&hook_path, &registry, "post-commit", false).unwrap(),
        InstallDecision::UpdateRegistry
    );

    // Jig-managed and in registry → Skip
    let mut registry = HookRegistry::new();
    registry.mark_installed("post-commit");
    assert_eq!(
        should_install_hook(&hook_path, &registry, "post-commit", false).unwrap(),
        InstallDecision::Skip
    );
}
