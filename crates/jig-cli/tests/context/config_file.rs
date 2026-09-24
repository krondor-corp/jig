//! Reading and writing the global `config.toml`.

use std::fs;

use jig_cli::context::{Config, NotifyConfig};

#[test]
fn roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config.toml");

    let cfg = Config {
        silence_threshold_seconds: 600,
        notify: NotifyConfig {
            exec: Some("notify-send".to_string()),
            events: vec!["worker.done".to_string()],
            ..Default::default()
        },
        default_base_branch: Some("origin/develop".to_string()),
        ..Default::default()
    };

    cfg.save_to(&path).unwrap();
    let loaded = Config::load_from(&path).unwrap();

    assert_eq!(loaded.silence_threshold_seconds, 600);
    assert_eq!(loaded.notify.exec.as_deref(), Some("notify-send"));
    assert_eq!(loaded.notify.events, vec!["worker.done"]);
    assert_eq!(
        loaded.default_base_branch.as_deref(),
        Some("origin/develop")
    );
}

#[test]
fn missing_file_returns_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("nonexistent.toml");
    let cfg = Config::load_from(&path).unwrap();
    assert_eq!(cfg.silence_threshold_seconds, 300);
}

#[test]
fn linear_profile_with_filters() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config.toml");
    fs::write(
        &path,
        r#"
[linear.profiles.work]
api_key = "lin_api_test"
team = "ENG"
projects = ["Backend", "Platform"]
assignee = "me"
labels = ["auto", "backend"]
"#,
    )
    .unwrap();

    let cfg = Config::load_from(&path).unwrap();
    let profile = cfg.linear.profiles.get("work").unwrap();
    assert_eq!(profile.api_key, "lin_api_test");
    assert_eq!(profile.team.as_deref(), Some("ENG"));
    assert_eq!(profile.projects, vec!["Backend", "Platform"]);
    assert_eq!(profile.assignee.as_deref(), Some("me"));
    assert_eq!(profile.labels, vec!["auto", "backend"]);
}

#[test]
fn partial_toml_fills_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config.toml");
    fs::write(&path, "silence_threshold_seconds = 600\n").unwrap();

    let cfg = Config::load_from(&path).unwrap();
    assert_eq!(cfg.silence_threshold_seconds, 600);
    assert_eq!(cfg.max_concurrent_workers, 3);
    assert!(cfg.notify.exec.is_none());
}
