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
