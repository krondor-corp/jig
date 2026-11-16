//! Two real daemons on one `static` mesh over loopback, spoken to the way a
//! peer speaks to them: one newline-delimited JSON request per connection on
//! the fleet port, as a caller the mesh trusts.
//!
//! Both listen on `127.0.0.1` (macOS has no other loopback address), so each
//! trusts and dials the other by that address, and so does this test. That
//! holds for two nodes; a third would need an address of its own
//! (`./bin/dev net-setup`).

use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::time::Duration;

use predicates::prelude::*;
use serde_json::{json, Value};

use crate::common::{wait_for, Sandbox};

/// A loopback port nothing is listening on right now.
fn free_port() -> u16 {
    TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// A machine called `name`, listening for the fleet on `port`, whose one
/// peer is `peer` on `peer_port`. `commands`: whether peers may run verbs.
fn node(name: &str, port: u16, peer: &str, peer_port: u16, commands: bool) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.write_global_config(&format!(
        r#"[fleet]
name = "{name}"
net = "static"
net_commands = {commands}

[fleet.static]
listen = "127.0.0.1:{port}"
peers = [{{ name = "{peer}", addr = "127.0.0.1:{peer_port}" }}]
"#
    ));
    sandbox
}

/// alice and bob, meshed; `bob_commands`: whether bob takes verbs from
/// peers. Returns bob's fleet port.
fn mesh(bob_commands: bool) -> (Sandbox, Sandbox, u16) {
    let (alice_port, bob_port) = (free_port(), free_port());
    let alice = node("alice", alice_port, "bob", bob_port, true);
    let bob = node("bob", bob_port, "alice", alice_port, bob_commands);
    (alice, bob, bob_port)
}

/// One request to the fleet listener on `port`, and its answer.
fn ask(port: u16, request: Value) -> Value {
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    writeln!(stream, "{request}").unwrap();
    let mut reply = String::new();
    BufReader::new(&stream).read_line(&mut reply).unwrap();
    serde_json::from_str(&reply).unwrap_or_else(|e| panic!("{e}: {reply}"))
}

/// Wait until the fleet listener on `port` answers.
fn wait_for_listener(port: u16) {
    wait_for(
        || TcpStream::connect((Ipv4Addr::LOCALHOST, port)).is_ok(),
        "the fleet listener",
    );
}

/// The reason in an `Error` answer, or a panic naming what came instead.
fn refusal(answer: &Value) -> &str {
    assert_eq!(answer["response"], "error", "{answer}");
    answer["body"].as_str().unwrap()
}

#[test]
fn a_peer_reads_a_machine_s_own_frame() {
    let (_alice, bob, bob_port) = mesh(false);
    let _b = bob.start_daemon();
    wait_for_listener(bob_port);

    // Reads need no `net_commands`.
    let status = ask(bob_port, json!({ "request": "get_status" }));
    assert_eq!(status["response"], "status", "{status}");
    assert_eq!(status["body"]["info"]["machine"]["name"], "bob");

    let issues = ask(bob_port, json!({ "request": "get_issues" }));
    assert_eq!(issues["response"], "issues", "{issues}");
}

#[test]
fn a_verb_from_a_peer_runs_on_the_machine_it_reaches() {
    let (_alice, bob, bob_port) = mesh(true);
    let _b = bob.start_daemon();
    wait_for_listener(bob_port);

    // bob's executor answering is what its refusal proves.
    let kill = ask(
        bob_port,
        json!({ "request": "kill", "worker": "no-such-repo/x" }),
    );
    assert!(
        refusal(&kill).contains("no tracked repo named 'no-such-repo'"),
        "{kill}"
    );
    // So is a spawn's; no agent starts.
    let spawn = ask(
        bob_port,
        json!({ "request": "spawn", "repo": "no-such-repo", "issue": "KRO-1" }),
    );
    assert!(
        refusal(&spawn).contains("no tracked repo named 'no-such-repo'"),
        "{spawn}"
    );
}

#[test]
fn a_machine_refuses_verbs_and_spawns_from_peers_unless_turned_on() {
    let (_alice, bob, bob_port) = mesh(false);
    let _b = bob.start_daemon();
    wait_for_listener(bob_port);

    for request in [
        json!({ "request": "kill", "worker": "no-such-repo/x" }),
        json!({ "request": "spawn", "repo": "no-such-repo", "issue": "KRO-1" }),
    ] {
        let answer = ask(bob_port, request);
        assert!(
            refusal(&answer).contains("bob does not take commands over the network"),
            "{answer}"
        );
    }
}

#[test]
fn a_peer_cannot_stop_a_daemon() {
    let (_alice, bob, bob_port) = mesh(true);
    let _b = bob.start_daemon();
    wait_for_listener(bob_port);

    let answer = ask(bob_port, json!({ "request": "shutdown" }));
    assert!(
        refusal(&answer).contains("only taken on the local socket"),
        "{answer}"
    );
    bob.jig().args(["daemon", "status"]).assert().success();
}

#[test]
fn ps_fleet_needs_a_fleet_network() {
    let sandbox = Sandbox::new();
    sandbox
        .jig()
        .args(["ps", "--fleet"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no fleet network"));
}

#[test]
fn ps_fleet_asks_every_peer() {
    let (alice, bob, bob_port) = mesh(false);
    let _a = alice.start_daemon();
    let _b = bob.start_daemon();
    wait_for_listener(bob_port);

    alice
        .jig()
        .args(["ps", "--fleet"])
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "No spawned sessions on any machine",
        ))
        .stderr(predicates::str::contains("not answering").not());
}

#[test]
fn ps_fleet_names_the_silent_peers() {
    let (alice, bob, _) = mesh(true);
    let _a = alice.start_daemon();
    // bob is configured but never started.
    drop(bob);

    alice
        .jig()
        .args(["ps", "--fleet"])
        .assert()
        .success()
        .stderr(predicates::str::contains("bob not answering"));
}
