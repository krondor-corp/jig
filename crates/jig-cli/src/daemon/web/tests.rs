use std::path::PathBuf;
use std::sync::Mutex;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use serde::Serialize;
use tower::ServiceExt;

use jig_api::workers::{kill, model, nudge, prune, retry_setup, setup_log};
use jig_api::{session, status};
use jig_api::{
    CommandOutcome, ErrorBody, KillRequest, ModelRequest, NudgeRequest, PruneRequest,
    RetrySetupRequest, SetupLog,
};
use jig_core::git::Branch;

use super::api::workers::setup_log::read_tail;
use super::guard::constant_time_eq;
use super::*;
use crate::context::RepoEntry;
use crate::daemon::ipc::{DaemonInfo, WorkerSnapshot};
use crate::worker::events::{SetupExit, SetupPhase, SetupState, WorkerState};
use crate::worker::{MuxStatus, WorkerStatus};

const PORT: u16 = 7477;
const HOST: &str = "127.0.0.1:7477";
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

fn failed(name: &str, log_path: Option<PathBuf>) -> WorkerState {
    let mut w = worker(name);
    w.status = WorkerStatus::Failed;
    w.setup = SetupState::Failed {
        phase: SetupPhase::OnCreate,
        exit: SetupExit::Code(1),
        tail: vec!["step 2: boom".into()],
        log_path,
    };
    w
}

fn app(fake: Fake) -> Router {
    router(Site {
        backend: fake,
        port: PORT,
        token: TOKEN.into(),
    })
}

/// Send one request through the whole router; the status and body.
async fn send(app: &Router, request: Request<Body>) -> (StatusCode, String) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

fn get(path: &str, host: &str) -> Request<Body> {
    Request::get(path)
        .header(header::HOST, host)
        .body(Body::empty())
        .unwrap()
}

/// A `POST` of a raw body to `path`, with `headers`.
fn post_raw(path: &str, body: &str, headers: &[(&str, &str)]) -> Request<Body> {
    let mut request = Request::post(path)
        .header(header::HOST, HOST)
        .header(header::CONTENT_TYPE, "application/json");
    for (k, v) in headers {
        request = request.header(*k, *v);
    }
    request.body(Body::from(body.to_string())).unwrap()
}

/// A `jig_api` request as its route sends it, with the token.
fn post<R: Serialize>(path: &str, request: &R) -> Request<Body> {
    post_raw(
        path,
        &serde_json::to_string(request).unwrap(),
        &[(jig_api::TOKEN_HEADER, TOKEN)],
    )
}

fn nudge_web() -> NudgeRequest {
    NudgeRequest {
        worker: "jig/feat/web".into(),
        text: "rebase please".into(),
    }
}

fn nudge_body() -> String {
    serde_json::to_string(&nudge_web()).unwrap()
}

#[tokio::test]
async fn the_status_is_the_ps_view_as_json() {
    let app = app(Fake {
        workers: vec![worker("feat/web")],
        ..Default::default()
    });
    for host in [HOST, "localhost:7477"] {
        let (code, body) = send(&app, get(status::get::PATH, host)).await;
        assert_eq!(code, StatusCode::OK, "{host}");
        let view: jig_api::Status = serde_json::from_str(&body).unwrap();
        assert_eq!(view.repos.len(), 1);
        assert_eq!(view.repos[0].name, "jig");
        let w = &view.repos[0].workers[0];
        assert_eq!(w.key, "jig/feat/web");
        assert_eq!(w.cells.len(), view.headers.len());
        assert!(w.failed_setup.is_none());
    }
}

#[tokio::test]
async fn the_session_hands_out_the_token() {
    let (code, body) = send(&app(Fake::default()), get(session::get::PATH, HOST)).await;
    assert_eq!(code, StatusCode::OK);
    let session: jig_api::Session = serde_json::from_str(&body).unwrap();
    assert_eq!(session.token, TOKEN);
}

#[tokio::test]
async fn a_foreign_host_is_refused() {
    let app = app(Fake::default());
    for host in [
        "evil.example:7477",
        "127.0.0.1:80",
        "localhost",
        "127.0.0.1.evil:7477",
    ] {
        for path in ["/", session::get::PATH, status::get::PATH, "/app.js"] {
            let (code, body) = send(&app, get(path, host)).await;
            assert_eq!(code, StatusCode::FORBIDDEN, "{host} {path}");
            assert!(!body.contains(TOKEN));
        }
    }
    let no_host = Request::get(session::get::PATH)
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&app, no_host).await.0, StatusCode::FORBIDDEN);

    let mut foreign_post = post(nudge::PATH, &nudge_web());
    foreign_post
        .headers_mut()
        .insert(header::HOST, "evil.example:7477".parse().unwrap());
    assert_eq!(send(&app, foreign_post).await.0, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_command_runs_with_the_token() {
    let fake = Fake::default();
    let app = app(fake.clone());
    let (code, body) = send(
        &app,
        post_raw(
            nudge::PATH,
            &nudge_body(),
            &[
                (jig_api::TOKEN_HEADER, TOKEN),
                ("origin", "http://127.0.0.1:7477"),
            ],
        ),
    )
    .await;
    assert_eq!(code, StatusCode::OK, "{body}");
    let outcome: CommandOutcome = serde_json::from_str(&body).unwrap();
    assert_eq!(outcome.message, "nudge jig/feat/web: done");
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

#[tokio::test]
async fn a_command_without_the_right_token_is_refused() {
    let fake = Fake::default();
    let app = app(fake.clone());
    for headers in [
        &[][..],
        &[(jig_api::TOKEN_HEADER, "guess")],
        &[(jig_api::TOKEN_HEADER, "")],
    ] {
        let (code, body) = send(&app, post_raw(nudge::PATH, &nudge_body(), headers)).await;
        assert_eq!(code, StatusCode::FORBIDDEN, "{headers:?}");
        assert!(serde_json::from_str::<ErrorBody>(&body).is_ok());
    }
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_command_from_a_foreign_origin_is_refused() {
    let fake = Fake::default();
    let (code, _) = send(
        &app(fake.clone()),
        post_raw(
            nudge::PATH,
            &nudge_body(),
            &[
                (jig_api::TOKEN_HEADER, TOKEN),
                ("origin", "https://evil.example"),
            ],
        ),
    )
    .await;
    assert_eq!(code, StatusCode::FORBIDDEN);
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[tokio::test]
async fn get_never_runs_a_command() {
    let fake = Fake::default();
    let app = app(fake.clone());
    for path in [
        kill::PATH,
        nudge::PATH,
        prune::PATH,
        retry_setup::PATH,
        model::PATH,
    ] {
        let mut request = get(path, HOST);
        request
            .headers_mut()
            .insert(jig_api::TOKEN_HEADER, TOKEN.parse().unwrap());
        assert_eq!(
            send(&app, request).await.0,
            StatusCode::METHOD_NOT_ALLOWED,
            "{path}"
        );
    }
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[tokio::test]
async fn every_verb_reaches_the_daemon() {
    let fake = Fake::default();
    let app = app(fake.clone());
    let worker = || "jig/feat/web".to_string();
    let requests = [
        post(kill::PATH, &KillRequest { worker: worker() }),
        post(nudge::PATH, &nudge_web()),
        post(prune::PATH, &PruneRequest { worker: worker() }),
        post(retry_setup::PATH, &RetrySetupRequest { worker: worker() }),
        post(
            model::PATH,
            &ModelRequest {
                worker: worker(),
                model: "opus".into(),
            },
        ),
    ];
    for request in requests {
        let (code, body) = send(&app, request).await;
        assert_eq!(code, StatusCode::OK, "{body}");
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

#[tokio::test]
async fn a_malformed_command_is_refused_with_a_reason() {
    let fake = Fake::default();
    let app = app(fake.clone());
    let token = &[(jig_api::TOKEN_HEADER, TOKEN)][..];
    for (path, body) in [
        (kill::PATH, r#"{"worker":"not-a-key"}"#),
        (kill::PATH, r#"{"worker":"jig/../etc"}"#),
        (nudge::PATH, r#"{"worker":"jig/feat/web"}"#),
        (kill::PATH, "nope"),
    ] {
        let (code, body) = send(&app, post_raw(path, body, token)).await;
        assert_eq!(code, StatusCode::BAD_REQUEST, "{path} {body}");
        assert!(serde_json::from_str::<ErrorBody>(&body).is_ok(), "{body}");
    }
    // A verb the API doesn't have is no route at all.
    let (code, _) = send(
        &app,
        post_raw("/api/v0/workers/shutdown", r#"{"worker":"jig/x"}"#, token),
    )
    .await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_refused_command_shows_the_daemons_reason() {
    let app = app(Fake {
        answer: Some("worker 'feat/web' is still running — kill it first".into()),
        ..Default::default()
    });
    let (code, body) = send(
        &app,
        post(
            prune::PATH,
            &PruneRequest {
                worker: "jig/feat/web".into(),
            },
        ),
    )
    .await;
    assert_eq!(code, StatusCode::CONFLICT);
    let error: ErrorBody = serde_json::from_str(&body).unwrap();
    assert!(error.error.contains("still running"), "{}", error.error);
}

#[tokio::test]
async fn a_failed_setup_is_in_the_status() {
    let (_, body) = send(
        &app(Fake {
            workers: vec![failed("feat/broken", Some(PathBuf::from("/tmp/setup.log")))],
            ..Default::default()
        }),
        get(status::get::PATH, HOST),
    )
    .await;
    let view: jig_api::Status = serde_json::from_str(&body).unwrap();
    let setup = view.repos[0].workers[0].failed_setup.as_ref().unwrap();
    assert_eq!(setup.tail, ["step 2: boom"]);
    assert!(setup.has_log);
}

// Touches the filesystem, but stays here rather than in `tests/`: it needs
// the private `Site` with a stand-in backend, which an integration test
// can't build. `tests/daemon/web.rs` covers the route on a real daemon.
#[tokio::test]
async fn the_setup_log_is_fetched_by_worker_not_by_path() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("setup.log");
    std::fs::write(&log, "step 1\nstep 2: boom\n").unwrap();

    let app = app(Fake {
        workers: vec![failed("feat/broken", Some(log)), worker("feat/fine")],
        ..Default::default()
    });
    let log_of = |query: &str| get(&format!("{}{query}", setup_log::PATH), HOST);

    let (code, body) = send(&app, log_of("?worker=jig%2Ffeat%2Fbroken")).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    let log: SetupLog = serde_json::from_str(&body).unwrap();
    assert_eq!(log.text, "step 1\nstep 2: boom\n");

    for (query, expected) in [
        ("?worker=jig/feat/fine", StatusCode::NOT_FOUND),
        // A path is not a worker key.
        ("?worker=/etc/passwd", StatusCode::BAD_REQUEST),
        ("?worker=jig/../../etc", StatusCode::BAD_REQUEST),
        ("?worker=jig/feat/nobody", StatusCode::NOT_FOUND),
        ("", StatusCode::BAD_REQUEST),
    ] {
        let (code, body) = send(&app, log_of(query)).await;
        assert_eq!(code, expected, "{query}");
        assert!(serde_json::from_str::<ErrorBody>(&body).is_ok(), "{body}");
    }
}

#[tokio::test]
async fn an_unknown_api_route_is_an_api_error_not_the_app() {
    let (code, body) = send(&app(Fake::default()), get("/api/v0/nope", HOST)).await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    assert!(serde_json::from_str::<ErrorBody>(&body).is_ok(), "{body}");
}

#[tokio::test]
async fn every_response_is_hardened() {
    let app = app(Fake::default());
    let response = app.oneshot(get(status::get::PATH, HOST)).await.unwrap();
    let headers = response.headers();
    assert_eq!(headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(headers[header::X_FRAME_OPTIONS], "DENY");
    let csp = headers[header::CONTENT_SECURITY_POLICY].to_str().unwrap();
    assert!(csp.contains("frame-ancestors 'none'"), "{csp}");
    assert!(!csp.contains("unsafe-inline"), "{csp}");
}

// Touches the filesystem, but `read_tail` is private to the web view, so the
// test has to live beside it.
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

#[test]
fn it_binds_loopback_only() {
    let server = WebServer::bind(0).unwrap();
    assert!(server.listener.local_addr().unwrap().ip().is_loopback());
    assert!(server.url().starts_with("http://127.0.0.1:"));
    // Fresh per session, and long enough not to guess.
    assert_ne!(server.token, WebServer::bind(0).unwrap().token);
    assert_eq!(server.token.len(), 32);
}
