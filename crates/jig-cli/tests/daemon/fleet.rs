//! Two real daemons on one `static` mesh over loopback: each one's web view
//! shows the other's frame, and a verb naming the other machine runs there.
//!
//! Both listen on `127.0.0.1` (macOS has no other loopback address), so each
//! trusts and dials the other by that address. That holds for two nodes; a
//! third would need an address of its own (`./bin/dev net-setup`).

use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::time::Duration;

use jig_api::{ApiError, Client, Fleet, FleetRequest, KillRequest, SessionRequest};

use super::web::{web_url, with_client};
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
web = true
web_port = 0
net = "static"
net_commands = {commands}

[fleet.static]
listen = "127.0.0.1:{port}"
peers = [{{ name = "{peer}", addr = "127.0.0.1:{peer_port}" }}]
"#
    ));
    sandbox
}

fn fleet(url: &str) -> Fleet {
    with_client(url, async |base| {
        Client::new(base).unwrap().call(FleetRequest).await
    })
    .expect("fleet")
}

/// Kill `worker` on `machine`, through the web view at `url`.
fn kill_on(url: &str, machine: &str, worker: &str) -> Result<String, ApiError> {
    with_client(url, async |base| {
        let session = Client::new(base).unwrap().call(SessionRequest).await?;
        let request = KillRequest {
            worker: worker.into(),
            machine: Some(machine.into()),
        };
        let outcome = Client::with_token(base, &session.token)
            .unwrap()
            .call(request)
            .await?;
        Ok(outcome.message)
    })
}

/// alice and bob, meshed; `bob_commands`: whether bob takes verbs from peers.
/// Returns once alice's page shows bob's frame.
fn mesh(bob_commands: bool) -> (Sandbox, Sandbox, u16) {
    let (alice_port, bob_port) = (free_port(), free_port());
    let alice = node("alice", alice_port, "bob", bob_port, true);
    let bob = node("bob", bob_port, "alice", alice_port, bob_commands);
    (alice, bob, bob_port)
}

#[test]
fn each_web_view_shows_the_whole_fleet() {
    let (alice, bob, _) = mesh(false);
    let _a = alice.start_daemon();
    let _b = bob.start_daemon();

    for (sandbox, here, there) in [(&alice, "alice", "bob"), (&bob, "bob", "alice")] {
        let url = web_url(sandbox);
        wait_for(
            || {
                fleet(&url)
                    .machines
                    .iter()
                    .any(|m| m.status.is_some() && !m.is_local())
            },
            &format!("{here}'s page to show {there}"),
        );
        let fleet = fleet(&url);
        let names: Vec<_> = fleet.machines.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, [here, there], "{fleet:?}");
        assert!(fleet.machines[0].is_local());
        let peer = &fleet.machines[1];
        assert_eq!(peer.node.as_deref(), Some("127.0.0.1"));
        assert_eq!(
            peer.status.as_ref().and_then(|s| s.machine.as_deref()),
            Some(there),
            "the frame is {there}'s own"
        );
    }
}

#[test]
fn a_verb_for_another_machine_runs_there() {
    let (alice, bob, _) = mesh(true);
    let _a = alice.start_daemon();
    let _b = bob.start_daemon();
    let url = web_url(&alice);
    wait_for(
        || {
            fleet(&url)
                .machines
                .iter()
                .any(|m| m.status.is_some() && !m.is_local())
        },
        "alice's page to show bob",
    );

    // bob's executor answering is what its refusal proves.
    match kill_on(&url, "127.0.0.1", "no-such-repo/x") {
        Err(ApiError::Refused(code, reason)) => {
            assert_eq!(code.as_u16(), 409);
            assert!(
                reason.contains("no tracked repo named 'no-such-repo'"),
                "{reason}"
            );
        }
        other => panic!("expected bob's refusal, got {other:?}"),
    }
}

#[test]
fn a_machine_refuses_verbs_from_peers_unless_turned_on() {
    let (alice, bob, _) = mesh(false);
    let _a = alice.start_daemon();
    let _b = bob.start_daemon();
    let url = web_url(&alice);
    wait_for(
        || {
            fleet(&url)
                .machines
                .iter()
                .any(|m| m.status.is_some() && !m.is_local())
        },
        "alice's page to show bob",
    );

    match kill_on(&url, "127.0.0.1", "no-such-repo/x") {
        Err(ApiError::Refused(code, reason)) => {
            assert_eq!(code.as_u16(), 409);
            assert!(
                reason.contains("bob does not take commands over the network"),
                "{reason}"
            );
        }
        other => panic!("expected bob to refuse, got {other:?}"),
    }
}

#[test]
fn a_peer_cannot_stop_a_daemon() {
    let (alice, bob, bob_port) = mesh(true);
    let _a = alice.start_daemon();
    let _b = bob.start_daemon();
    let url = web_url(&alice);
    wait_for(
        || {
            fleet(&url)
                .machines
                .iter()
                .any(|m| m.status.is_some() && !m.is_local())
        },
        "bob's fleet listener",
    );

    // Straight onto bob's fleet port, as a trusted peer would.
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, bob_port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(b"{\"request\":\"shutdown\"}\n").unwrap();
    let mut reply = String::new();
    BufReader::new(&stream).read_line(&mut reply).unwrap();
    assert!(reply.contains("\"error\""), "{reply}");
    assert!(reply.contains("only taken on the local socket"), "{reply}");

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
fn ps_fleet_asks_every_peer_and_names_the_silent_ones() {
    let (alice, bob, _) = mesh(true);
    let _a = alice.start_daemon();
    // bob is configured but never started.
    drop(bob);

    alice
        .jig()
        .args(["ps", "--fleet"])
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "No spawned sessions on any machine",
        ))
        .stderr(predicates::str::contains("bob not answering"));
}
