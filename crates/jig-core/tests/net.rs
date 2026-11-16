//! The `static` net backend end to end, over loopback.

use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

use jig_core::net::{Net, NodeId, StaticConfig, StaticNet, StaticPeer};

fn loopback(port: u16) -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], port))
}

/// A listening machine on an ephemeral loopback port that trusts `allow`.
fn server(allow: &[&str]) -> (Arc<StaticNet>, SocketAddr) {
    let net = StaticNet::new(
        "server",
        StaticConfig {
            listen: loopback(0),
            peers: vec![],
            allow: allow.iter().map(|a| a.parse().unwrap()).collect(),
        },
    );
    let addr = net.listen().unwrap();
    (Arc::new(net), addr)
}

/// A machine whose only peer is `server` at `addr`.
fn client(addr: SocketAddr) -> StaticNet {
    StaticNet::new(
        "client",
        StaticConfig {
            listen: loopback(0),
            peers: vec![StaticPeer {
                name: "server".into(),
                addr,
            }],
            allow: vec![],
        },
    )
}

#[test]
fn a_trusted_peer_connects_and_is_identified() {
    let (server, addr) = server(&["127.0.0.1"]);
    let accepting = {
        let server = Arc::clone(&server);
        thread::spawn(move || {
            let (mut stream, from) = server.accept().unwrap();
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).unwrap();
            stream.write_all(b"pong").unwrap();
            (from, buf)
        })
    };

    let client = client(addr);
    let peer = client.peers().unwrap().remove(0);
    assert_eq!(peer.name, "server");
    let mut stream = client.dial(&peer.id).unwrap();
    stream.write_all(b"ping").unwrap();
    let mut reply = [0u8; 4];
    stream.read_exact(&mut reply).unwrap();

    let (from, got) = accepting.join().unwrap();
    assert_eq!(&got, b"ping");
    assert_eq!(&reply, b"pong");
    assert_eq!(from, NodeId::new("127.0.0.1"));
}

#[test]
fn an_untrusted_caller_is_dropped_inside_accept() {
    // Trusts someone else entirely; our loopback connection is a stranger.
    let (server, addr) = server(&["10.9.9.9"]);
    let accepted = {
        let server = Arc::clone(&server);
        // Never returns: nobody trusted ever calls. The thread dies with
        // the test binary.
        thread::spawn(move || server.accept().map(|(_, from)| from).ok())
    };

    let mut stream = client(addr).dial(&NodeId::new("127.0.0.1")).unwrap();
    let mut buf = [0u8; 1];
    // The server closed it without a word (EOF, or a reset).
    assert_eq!(stream.read(&mut buf).unwrap_or(0), 0);
    assert!(!accepted.is_finished(), "a stranger got through accept");
}
