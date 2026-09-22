//! Global directory setup, which every `jig` invocation does on startup.

mod common;

use common::Sandbox;

#[test]
fn any_command_creates_the_global_dirs() {
    let sandbox = Sandbox::new();
    sandbox.jig().arg("version").assert().success();

    let jig = sandbox.jig_dir();
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
    let logs: Vec<_> = std::fs::read_dir(sandbox.jig_dir().join("state/logs"))
        .unwrap()
        .collect();
    assert!(
        logs.is_empty(),
        "one-off commands log to stderr, not files: {logs:?}"
    );
}
