//! `jig.toml` and the gitignored `jig.local.toml` overlay on disk.

use std::fs;

use jig_cli::context::{JigToml, JIG_LOCAL_TOML, JIG_TOML};

#[test]
fn load_with_local_overlay() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join(JIG_TOML),
        "[worktree]\nbase = \"origin/main\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(JIG_LOCAL_TOML),
        "[issues]\nauto_spawn_labels = []\n",
    )
    .unwrap();
    let config = JigToml::load(dir.path()).unwrap().unwrap();
    assert_eq!(config.worktree.base.as_deref(), Some("origin/main"));
    assert_eq!(config.issues.auto_spawn_labels, Some(vec![]));
    assert!(config.has_local_overlay);
}

#[test]
fn local_only_loads_standalone() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join(JIG_LOCAL_TOML),
        "[issues]\nauto_spawn_labels = [\"auto\"]\n",
    )
    .unwrap();
    let config = JigToml::load(dir.path()).unwrap().unwrap();
    assert!(config.local_only);
    assert!(!config.has_local_overlay);
    assert_eq!(
        config.issues.auto_spawn_labels,
        Some(vec!["auto".to_string()])
    );
}

#[test]
fn neither_toml_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    assert!(JigToml::load(dir.path()).unwrap().is_none());
}

#[test]
fn local_only_exists_detects_correctly() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!JigToml::local_only_exists(dir.path()));
    fs::write(dir.path().join(JIG_LOCAL_TOML), "[issues]\n").unwrap();
    assert!(JigToml::local_only_exists(dir.path()));
    fs::write(dir.path().join(JIG_TOML), "[worktree]\n").unwrap();
    assert!(!JigToml::local_only_exists(dir.path()));
}
