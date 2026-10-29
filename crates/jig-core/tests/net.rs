//! The `static` net backend end to end, over loopback.

use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

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

#[test]
fn a_peer_calls_from_its_own_listen_address() {
    // Needs a second loopback address: Linux routes all of 127/8, macOS
    // only has the ones `./bin/dev net-setup` adds.
    let ours = "127.0.0.2".parse().unwrap();
    if std::net::TcpListener::bind(SocketAddr::new(ours, 0)).is_err() {
        eprintln!("skipped: 127.0.0.2 is not an address of this machine");
        return;
    }
    // Trusts only 127.0.0.2: the OS's own choice of source (127.0.0.1, or on
    // macOS the destination's address) would be dropped.
    let (server, addr) = server(&["127.0.0.2"]);
    // A dropped caller leaves `accept` waiting forever, so wait for it with
    // a deadline: a failure, not a hung test.
    let (accepted, from) = mpsc::channel();
    {
        let server = Arc::clone(&server);
        thread::spawn(move || {
            let _ = accepted.send(server.accept().map(|(_, from)| from).unwrap());
        });
    }
    let client = StaticNet::new(
        "client",
        StaticConfig {
            listen: SocketAddr::new(ours, 0),
            peers: vec![StaticPeer {
                name: "server".into(),
                addr,
            }],
            allow: vec![],
        },
    );
    let _stream = client.dial(&NodeId::new("127.0.0.1")).unwrap();
    let from = from
        .recv_timeout(Duration::from_secs(5))
        .expect("the server dropped us: we called from an address it doesn't trust");
    assert_eq!(from, NodeId::new("127.0.0.2"));
}
