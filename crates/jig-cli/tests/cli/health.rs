//! `jig health` reports on the fleet network.

use crate::common::Sandbox;

fn stderr(sandbox: &Sandbox) -> String {
    let out = sandbox.jig().arg("health").output().unwrap();
    String::from_utf8(out.stderr).unwrap()
}

#[test]
fn net_is_reported_off_when_unconfigured() {
    let sandbox = Sandbox::new();
    let err = stderr(&sandbox);
    assert!(err.contains("Net"), "{err}");
    assert!(err.contains("off"), "{err}");
}

#[test]
fn a_static_net_reports_its_checks() {
    let sandbox = Sandbox::new();
    sandbox.write_global_config(
        r#"
[fleet]
net = "static"

[fleet.static]
listen = "127.0.0.1:7473"
peers = [{ name = "laptop", addr = "127.0.0.2:7473" }]
"#,
    );
    let err = stderr(&sandbox);
    assert!(err.contains("Net: static"), "{err}");
    assert!(err.contains("127.0.0.1 is on this machine"), "{err}");
    assert!(err.contains("1 peer"), "{err}");
}

#[test]
fn a_static_net_on_a_foreign_address_fails_health() {
    let sandbox = Sandbox::new();
    sandbox.write_global_config(
        "[fleet]\nnet = \"static\"\n[fleet.static]\nlisten = \"192.0.2.1:7473\"\n",
    );
    let out = sandbox.jig().arg("health").output().unwrap();
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(!out.status.success());
    assert!(err.contains("not an address of this machine"), "{err}");
}

#[test]
fn a_static_net_without_its_section_says_so() {
    let sandbox = Sandbox::new();
    sandbox.write_global_config("[fleet]\nnet = \"static\"\n");
    let err = stderr(&sandbox);
    assert!(err.contains("[fleet.static]"), "{err}");
}
