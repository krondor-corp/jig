//! Global directory setup, which every `jig` invocation does on startup.

use crate::common::Sandbox;
use jig_cli::context::AppPaths;

#[test]
fn any_command_creates_the_global_dirs() {
    let sandbox = Sandbox::new();
    sandbox.jig().arg("version").assert().success();

    let jig = sandbox.paths().config_dir().to_path_buf();
    for dir in ["", "hooks", "state", "state/events", "state/logs"] {
        assert!(
            jig.join(dir).is_dir(),
            "missing {}",
            jig.join(dir).display()
        );
    }
}

#[test]
fn one_off_commands_leave_no_log_files() {
    let sandbox = Sandbox::with_repo();
    for args in [&["version"][..], &["ls"], &["ps"], &["daemon", "status"]] {
        sandbox.jig().args(args).output().unwrap();
    }
    let logs: Vec<_> = std::fs::read_dir(
        sandbox
            .paths()
            .config_dir()
            .to_path_buf()
            .join("state/logs"),
    )
    .unwrap()
    .collect();
    assert!(
        logs.is_empty(),
        "one-off commands log to stderr, not files: {logs:?}"
    );
}

#[test]
fn ensure_creates_every_dir() {
    let root = tempfile::tempdir().unwrap();
    let paths = AppPaths::under(&root.path().join("config"), &root.path().join("run"));
    paths.ensure().unwrap();
    for dir in [
        paths.config_dir().to_path_buf(),
        paths.hooks_dir(),
        paths.state_dir(),
        paths.events_dir(),
        paths.logs_dir(),
        paths.runtime_dir().to_path_buf(),
    ] {
        assert!(dir.is_dir(), "missing {}", dir.display());
    }
}
