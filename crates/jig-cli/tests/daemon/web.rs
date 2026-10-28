//! The web view, served by a real daemon process: turned on in config,
//! found through `jig web`, and answering over HTTP with the daemon's own
//! command executor behind it.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::time::Duration;

use predicates::prelude::*;

use crate::common::Sandbox;

/// A sandbox whose daemon serves the web view on `port` (0 picks one).
fn with_web(port: u16) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.write_global_config(&format!("[fleet]\nweb = true\nweb_port = {port}\n"));
    sandbox
}

/// The URL `jig web --no-open` prints.
fn web_url(sandbox: &Sandbox) -> String {
    let out = sandbox
        .jig()
        .args(["web", "--no-open"])
        .output()
        .expect("run jig web");
    assert!(out.status.success(), "jig web failed: {out:?}");
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// The port in `http://127.0.0.1:<port>/`.
fn port_of(url: &str) -> u16 {
    url.trim_start_matches("http://127.0.0.1:")
        .trim_end_matches('/')
        .parse()
        .unwrap_or_else(|_| panic!("not a web view URL: {url}"))
}

/// One raw HTTP exchange. Every request here says `Connection: close`, so
/// the server closes once it has answered; the timeout turns a request that
/// forgot into a failure instead of a hang.
fn send(port: u16, raw: &str) -> String {
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(raw.as_bytes()).unwrap();
    let mut out = String::new();
    stream.read_to_string(&mut out).unwrap();
    out
}

fn get(port: u16, path: &str) -> String {
    send(
        port,
        &format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    )
}

fn post_command(port: u16, token: &str, body: &str) -> String {
    send(
        port,
        &format!(
            "POST {} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{}: {token}\r\n\
             Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            jig_api::COMMAND,
            jig_api::TOKEN_HEADER,
            body.len()
        ),
    )
}

/// The JSON body of a raw HTTP response.
fn json_body<T: serde::de::DeserializeOwned>(response: &str) -> T {
    let (_, body) = response.split_once("\r\n\r\n").expect("a body");
    serde_json::from_str(body).unwrap_or_else(|e| panic!("not JSON ({e}): {body}"))
}

#[test]
fn the_web_view_is_off_unless_turned_on() {
    let sandbox = Sandbox::new();
    let _daemon = sandbox.start_daemon();

    sandbox
        .jig()
        .args(["web", "--no-open"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not serving a web view"));
}

#[test]
fn jig_web_says_when_no_daemon_is_running() {
    let sandbox = with_web(0);

    sandbox
        .jig()
        .args(["web", "--no-open"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("daemon is not running"));
}

#[test]
fn a_daemon_serves_its_web_view() {
    let sandbox = with_web(0);
    let _daemon = sandbox.start_daemon();

    let url = web_url(&sandbox);
    let port = port_of(&url);
    sandbox
        .jig()
        .args(["daemon", "status"])
        .assert()
        .success()
        .stderr(predicate::str::contains(url.as_str()));

    let session = get(port, jig_api::SESSION);
    assert!(session.starts_with("HTTP/1.1 200 OK"), "{session}");
    let token = json_body::<jig_api::Session>(&session).token;

    let status = get(port, jig_api::STATUS);
    assert!(status.starts_with("HTTP/1.1 200 OK"), "{status}");
    let status: jig_api::Status = json_body(&status);
    assert!(status.pid > 0);
    assert!(!status.headers.is_empty());

    // The app itself, or, in a build without `make build-web`, a page that
    // says so. Either way the daemon answers.
    let app = get(port, "/");
    assert!(
        app.starts_with("HTTP/1.1 200 OK") || app.starts_with("HTTP/1.1 503"),
        "{app}"
    );

    let rebound = send(
        port,
        "GET / HTTP/1.1\r\nHost: attacker.example\r\nConnection: close\r\n\r\n",
    );
    assert!(rebound.starts_with("HTTP/1.1 403"), "{rebound}");

    // Reaching the daemon's executor is what a refusal with its reason
    // proves: the web layer alone knows nothing about tracked repos.
    let kill = r#"{"worker":"no-such-repo/x","command":{"verb":"kill"}}"#;
    let refused = post_command(port, &token, kill);
    assert!(refused.starts_with("HTTP/1.1 409"), "{refused}");
    assert!(
        refused.contains("no tracked repo named 'no-such-repo'"),
        "{refused}"
    );

    let forged = post_command(port, "not-the-token", kill);
    assert!(forged.starts_with("HTTP/1.1 403"), "{forged}");
}

#[test]
fn a_taken_port_leaves_the_daemon_running_without_a_web_view() {
    let taken = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = taken.local_addr().unwrap().port();
    let sandbox = with_web(port);
    let _daemon = sandbox.start_daemon();

    sandbox
        .jig()
        .args(["web", "--no-open"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not serving a web view"));
}
