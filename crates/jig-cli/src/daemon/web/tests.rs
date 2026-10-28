use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use jig_core::git::Branch;

use super::*;
use crate::context::RepoEntry;
use crate::daemon::ipc::{DaemonInfo, WorkerSnapshot};
use crate::worker::events::{SetupExit, SetupPhase, WorkerState};
use crate::worker::{MuxStatus, WorkerStatus};

const PORT: u16 = 7477;
const TOKEN: &str = "s3cret";

/// A daemon stand-in: a fixed frame, and a record of every command.
#[derive(Clone, Default)]
struct Fake {
    workers: Vec<WorkerState>,
    ran: Arc<Mutex<Vec<(WorkerKey, Command)>>>,
    /// What every command answers.
    answer: Option<String>,
}

impl Backend for Fake {
    fn status(&self) -> DaemonStatus {
        DaemonStatus {
            info: DaemonInfo {
                pid: 1,
                version: "0.0.0".into(),
                started_at: 0,
                ticked_at: 0,
                tick_interval: 2,
                log: None,
                actors: vec![],
                machine: None,
                web: None,
            },
            workers: self.workers.iter().map(WorkerSnapshot::from).collect(),
            triages: vec![],
            spawning: vec![],
            poll_remaining: 0,
        }
    }

    fn run_command(&self, worker: WorkerKey, cmd: Command) -> Response {
        self.ran.lock().unwrap().push((worker, cmd));
        match &self.answer {
            None => Response::Ok,
            Some(reason) => Response::Error(reason.clone()),
        }
    }
}

fn worker(name: &str) -> WorkerState {
    let now = chrono::Utc::now();
    WorkerState {
        name: name.into(),
        resolved_branch: Branch::new(name),
        status: WorkerStatus::Running,
        mux_status: MuxStatus::Running,
        repo: Some(RepoEntry {
            path: PathBuf::from("/src/jig"),
            added: now,
            last_used: now,
        }),
        ..Default::default()
    }
}

fn site(fake: Fake) -> Site<Fake> {
    Site {
        backend: fake,
        port: PORT,
        token: TOKEN.into(),
    }
}

fn get(path: &str, host: &str) -> HttpRequest {
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    HttpRequest {
        method: "GET".into(),
        path: path.into(),
        query: query.into(),
        headers: vec![("host".into(), host.into())],
        body: vec![],
    }
}

fn post(body: &str, headers: &[(&str, &str)]) -> HttpRequest {
    let mut all = vec![("host".to_string(), format!("127.0.0.1:{PORT}"))];
    all.extend(headers.iter().map(|(k, v)| (k.to_string(), v.to_string())));
    HttpRequest {
        method: "POST".into(),
        path: "/command".into(),
        query: String::new(),
        headers: all,
        body: body.as_bytes().to_vec(),
    }
}

fn body(r: &HttpResponse) -> String {
    String::from_utf8(r.body.clone()).unwrap()
}

const NUDGE: &str = r#"{"request":"nudge","worker":"jig/feat/web","text":"rebase please"}"#;

#[test]
fn the_page_is_served_to_our_own_host() {
    let site = site(Fake {
        workers: vec![worker("feat/web")],
        ..Default::default()
    });
    for host in ["127.0.0.1:7477", "localhost:7477"] {
        let r = site.respond(&get("/", host));
        assert_eq!(r.status, 200, "{host}");
        assert!(body(&r).contains("feat/web"));
        assert!(body(&r).contains(TOKEN));
    }
}

#[test]
fn a_foreign_host_is_refused() {
    let site = site(Fake::default());
    for host in [
        "evil.example:7477",
        "127.0.0.1:80",
        "localhost",
        "127.0.0.1.evil:7477",
    ] {
        for path in ["/", "/view", "/setup-log?worker=jig/x", "/app.js"] {
            let r = site.respond(&get(path, host));
            assert_eq!(r.status, 403, "{host} {path}");
            assert!(!body(&r).contains(TOKEN));
        }
    }
    let mut no_host = get("/", "");
    no_host.headers.clear();
    assert_eq!(site.respond(&no_host).status, 403);

    let mut foreign_post = post(NUDGE, &[("x-jig-token", TOKEN)]);
    foreign_post.headers[0].1 = "evil.example:7477".into();
    assert_eq!(site.respond(&foreign_post).status, 403);
}

#[test]
fn a_command_runs_with_the_token() {
    let fake = Fake::default();
    let site = site(fake.clone());
    let r = site.respond(&post(
        NUDGE,
        &[("x-jig-token", TOKEN), ("origin", "http://127.0.0.1:7477")],
    ));
    assert_eq!(r.status, 200, "{}", body(&r));
    assert!(body(&r).contains(r#""ok":true"#));
    assert_eq!(
        fake.ran.lock().unwrap().as_slice(),
        &[(
            "jig/feat/web".parse().unwrap(),
            Command::Nudge {
                text: "rebase please".into()
            }
        )]
    );
}

#[test]
fn a_command_without_the_right_token_is_refused() {
    let fake = Fake::default();
    let site = site(fake.clone());
    for headers in [&[][..], &[("x-jig-token", "guess")], &[("x-jig-token", "")]] {
        let r = site.respond(&post(NUDGE, headers));
        assert_eq!(r.status, 403, "{headers:?}");
    }
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[test]
fn a_command_from_a_foreign_origin_is_refused() {
    let fake = Fake::default();
    let site = site(fake.clone());
    let r = site.respond(&post(
        NUDGE,
        &[("x-jig-token", TOKEN), ("origin", "https://evil.example")],
    ));
    assert_eq!(r.status, 403);
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[test]
fn get_never_runs_a_command() {
    let fake = Fake::default();
    let site = site(fake.clone());
    let mut r = get("/command", "127.0.0.1:7477");
    r.headers.push(("x-jig-token".into(), TOKEN.into()));
    assert_eq!(site.respond(&r).status, 405);
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[test]
fn every_verb_is_accepted() {
    let fake = Fake::default();
    let site = site(fake.clone());
    for body in [
        r#"{"request":"kill","worker":"jig/feat/web"}"#,
        NUDGE,
        r#"{"request":"prune","worker":"jig/feat/web"}"#,
        r#"{"request":"retry_setup","worker":"jig/feat/web"}"#,
        r#"{"request":"model","worker":"jig/feat/web","model":"opus"}"#,
    ] {
        let r = site.respond(&post(body, &[("x-jig-token", TOKEN)]));
        assert_eq!(r.status, 200, "{body}");
    }
    let verbs: Vec<_> = fake
        .ran
        .lock()
        .unwrap()
        .iter()
        .map(|(_, c)| c.name())
        .collect();
    assert_eq!(verbs, ["kill", "nudge", "prune", "retry-setup", "model"]);
}

#[test]
fn a_query_is_not_a_command() {
    let fake = Fake::default();
    let site = site(fake.clone());
    for body in [
        r#"{"request":"shutdown"}"#,
        r#"{"request":"get_status"}"#,
        "nope",
    ] {
        assert_eq!(
            site.respond(&post(body, &[("x-jig-token", TOKEN)])).status,
            400,
            "{body}"
        );
    }
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[test]
fn a_refused_command_shows_the_daemons_reason() {
    let site = site(Fake {
        answer: Some("worker 'feat/web' is still running — kill it first".into()),
        ..Default::default()
    });
    let r = site.respond(&post(
        r#"{"request":"prune","worker":"jig/feat/web"}"#,
        &[("x-jig-token", TOKEN)],
    ));
    assert_eq!(r.status, 409);
    assert!(body(&r).contains("still running"));
    assert!(body(&r).contains(r#""ok":false"#));
}

#[test]
fn the_setup_log_is_fetched_by_worker_not_by_path() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("setup.log");
    std::fs::write(&log, "step 1\nstep 2: boom\n").unwrap();

    let mut w = worker("feat/broken");
    w.status = WorkerStatus::Failed;
    w.setup = SetupState::Failed {
        phase: SetupPhase::OnCreate,
        exit: SetupExit::Code(1),
        tail: vec!["step 2: boom".into()],
        log_path: Some(log),
    };
    let site = site(Fake {
        workers: vec![w, worker("feat/fine")],
        ..Default::default()
    });

    let r = site.respond(&get(
        "/setup-log?worker=jig%2Ffeat%2Fbroken",
        "127.0.0.1:7477",
    ));
    assert_eq!(r.status, 200);
    assert_eq!(body(&r), "step 1\nstep 2: boom\n");

    let none = site.respond(&get("/setup-log?worker=jig/feat/fine", "127.0.0.1:7477"));
    assert_eq!(none.status, 404);
    // A path is not a worker key.
    let bad = site.respond(&get("/setup-log?worker=/etc/passwd", "127.0.0.1:7477"));
    assert_eq!(bad.status, 400);
    let traversal = site.respond(&get("/setup-log?worker=jig/../../etc", "127.0.0.1:7477"));
    assert_eq!(traversal.status, 400);
    let unknown = site.respond(&get("/setup-log?worker=jig/feat/nobody", "127.0.0.1:7477"));
    assert_eq!(unknown.status, 404);
    let missing = site.respond(&get("/setup-log", "127.0.0.1:7477"));
    assert_eq!(missing.status, 400);
}

#[test]
fn a_long_log_returns_its_end() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("setup.log");
    std::fs::write(&path, "0123456789").unwrap();
    assert_eq!(read_tail(&path, 4).unwrap(), b"6789");
    assert_eq!(read_tail(&path, 100).unwrap(), b"0123456789");
}

#[test]
fn token_comparison() {
    assert!(constant_time_eq(b"abc", b"abc"));
    assert!(!constant_time_eq(b"abc", b"abd"));
    assert!(!constant_time_eq(b"abc", b"ab"));
}

// ── Over a real socket ──────────────────────────────────────────────

/// Start a server on a free port; returns its port and token.
fn serve(fake: Fake) -> (u16, String, Arc<AtomicBool>) {
    let server = WebServer::bind(0).unwrap();
    let port = server.port;
    let token = server.token.clone();
    let quit = Arc::new(AtomicBool::new(false));
    server.spawn(fake, Arc::clone(&quit));
    (port, token, quit)
}

fn send(port: u16, raw: &str) -> String {
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
    stream.write_all(raw.as_bytes()).unwrap();
    let mut out = String::new();
    stream.read_to_string(&mut out).unwrap();
    out
}

#[test]
fn it_binds_loopback_only() {
    let server = WebServer::bind(0).unwrap();
    assert!(server.listener.local_addr().unwrap().ip().is_loopback());
    assert!(server.url().starts_with("http://127.0.0.1:"));
    // Fresh per session, and long enough not to guess.
    assert_ne!(server.token, WebServer::bind(0).unwrap().token);
    assert_eq!(server.token.len(), 32);
}

#[test]
fn end_to_end_page_then_command() {
    let fake = Fake {
        workers: vec![worker("feat/web")],
        ..Default::default()
    };
    let (port, token, quit) = serve(fake.clone());

    let page = send(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n"),
    );
    assert!(page.starts_with("HTTP/1.1 200 OK"), "{page}");
    assert!(page.contains("Content-Security-Policy:"));
    assert!(page.contains(&format!(r#"content="{token}""#)));

    let rebound = send(port, "GET / HTTP/1.1\r\nHost: attacker.example\r\n\r\n");
    assert!(rebound.starts_with("HTTP/1.1 403"), "{rebound}");
    assert!(!rebound.contains(&token));

    let body = r#"{"request":"kill","worker":"jig/feat/web"}"#;
    let tokenless = send(
        port,
        &format!(
            "POST /command HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(tokenless.starts_with("HTTP/1.1 403"), "{tokenless}");
    assert!(fake.ran.lock().unwrap().is_empty());

    let ok = send(
        port,
        &format!(
            "POST /command HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Jig-Token: {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(ok.starts_with("HTTP/1.1 200 OK"), "{ok}");
    assert_eq!(fake.ran.lock().unwrap().len(), 1);

    quit.store(true, Ordering::Relaxed);
}
