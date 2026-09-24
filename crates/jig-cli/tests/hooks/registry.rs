//! The hook registry as it is stored in a repo.

use jig_cli::hooks::registry::{registry_path, HookRegistry};

#[test]
fn save_load_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let mut registry = HookRegistry::new();
    registry.mark_installed("post-commit");
    registry.mark_installed("pre-commit");
    registry.mark_existing_backed_up("post-commit", "post-commit.backup");

    registry.save(tmp.path()).unwrap();

    let loaded = HookRegistry::load(tmp.path()).unwrap();
    assert_eq!(loaded.version, "1");
    assert!(loaded.is_installed("post-commit"));
    assert!(loaded.is_installed("pre-commit"));
    assert!(loaded.installed["post-commit"].had_existing);
}

#[test]
fn load_missing_file_returns_new() {
    let tmp = tempfile::tempdir().unwrap();
    let loaded = HookRegistry::load(tmp.path()).unwrap();
    assert_eq!(loaded.version, "1");
    assert!(loaded.installed.is_empty());
}

#[test]
fn save_is_pretty_printed() {
    let tmp = tempfile::tempdir().unwrap();
    let mut registry = HookRegistry::new();
    registry.mark_installed("post-commit");
    registry.save(tmp.path()).unwrap();

    let content = std::fs::read_to_string(registry_path(tmp.path())).unwrap();
    assert!(content.contains('\n'));
    assert!(content.contains("  ")); // indented
}
