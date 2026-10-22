//! `jig.toml` and the gitignored `jig.local.toml` overlay on disk.

use std::fs;

use jig_cli::context::{RepoConfig, JIG_LOCAL_TOML, JIG_TOML};
use jig_core::agents::AgentKind;

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
    let config = RepoConfig::load(dir.path()).unwrap().unwrap();
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
    let config = RepoConfig::load(dir.path()).unwrap().unwrap();
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
    assert!(RepoConfig::load(dir.path()).unwrap().is_none());
}

#[test]
fn local_only_exists_detects_correctly() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!RepoConfig::local_only_exists(dir.path()));
    fs::write(dir.path().join(JIG_LOCAL_TOML), "[issues]\n").unwrap();
    assert!(RepoConfig::local_only_exists(dir.path()));
    fs::write(dir.path().join(JIG_TOML), "[worktree]\n").unwrap();
    assert!(!RepoConfig::local_only_exists(dir.path()));
}

#[test]
fn the_agent_type_is_read_from_config() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join(JIG_TOML),
        "[agent]\ntype = \"claude\"\nmodel = \"opus\"\n",
    )
    .unwrap();
    let config = RepoConfig::load(dir.path()).unwrap().unwrap();
    assert_eq!(config.agent.agent_type, AgentKind::Claude);
    assert_eq!(config.agent.agent().model(), Some("opus"));
}

/// A typo must not quietly run some other agent than the one configured.
#[test]
fn an_unknown_agent_type_fails_the_load() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(JIG_TOML), "[agent]\ntype = \"claud\"\n").unwrap();
    let err = RepoConfig::load(dir.path()).unwrap_err().to_string();
    assert!(err.contains("claud"), "{err}");
}
