use std::path::PathBuf;
use std::sync::Mutex;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use tower::ServiceExt;

use jig_core::git::Branch;

use super::api::read_tail;
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

fn post(body: &str, headers: &[(&str, &str)]) -> Request<Body> {
    let mut request = Request::post(jig_api::COMMAND)
        .header(header::HOST, HOST)
        .header(header::CONTENT_TYPE, "application/json");
    for (k, v) in headers {
        request = request.header(*k, *v);
    }
    request.body(Body::from(body.to_string())).unwrap()
}

const NUDGE: &str =
    r#"{"worker":"jig/feat/web","command":{"verb":"nudge","text":"rebase please"}}"#;

#[tokio::test]
async fn the_status_is_the_ps_view_as_json() {
    let app = app(Fake {
        workers: vec![worker("feat/web")],
        ..Default::default()
    });
    for host in [HOST, "localhost:7477"] {
        let (status, body) = send(&app, get(jig_api::STATUS, host)).await;
        assert_eq!(status, StatusCode::OK, "{host}");
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
    let (status, body) = send(&app(Fake::default()), get(jig_api::SESSION, HOST)).await;
    assert_eq!(status, StatusCode::OK);
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
        for path in ["/", jig_api::SESSION, jig_api::STATUS, "/app.js"] {
            let (status, body) = send(&app, get(path, host)).await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{host} {path}");
            assert!(!body.contains(TOKEN));
        }
    }
    let no_host = Request::get(jig_api::SESSION).body(Body::empty()).unwrap();
    assert_eq!(send(&app, no_host).await.0, StatusCode::FORBIDDEN);

    let mut foreign_post = post(NUDGE, &[(jig_api::TOKEN_HEADER, TOKEN)]);
    foreign_post
        .headers_mut()
        .insert(header::HOST, "evil.example:7477".parse().unwrap());
    assert_eq!(send(&app, foreign_post).await.0, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_command_runs_with_the_token() {
    let fake = Fake::default();
    let app = app(fake.clone());
    let (status, body) = send(
        &app,
        post(
            NUDGE,
            &[
                (jig_api::TOKEN_HEADER, TOKEN),
                ("origin", "http://127.0.0.1:7477"),
            ],
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let outcome: jig_api::CommandOutcome = serde_json::from_str(&body).unwrap();
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
        let (status, body) = send(&app, post(NUDGE, headers)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{headers:?}");
        assert!(serde_json::from_str::<jig_api::ApiError>(&body).is_ok());
    }
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_command_from_a_foreign_origin_is_refused() {
    let fake = Fake::default();
    let (status, _) = send(
        &app(fake.clone()),
        post(
            NUDGE,
            &[
                (jig_api::TOKEN_HEADER, TOKEN),
                ("origin", "https://evil.example"),
            ],
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[tokio::test]
async fn get_never_runs_a_command() {
    let fake = Fake::default();
    let mut request = get(jig_api::COMMAND, HOST);
    request
        .headers_mut()
        .insert(jig_api::TOKEN_HEADER, TOKEN.parse().unwrap());
    let (status, _) = send(&app(fake.clone()), request).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[tokio::test]
async fn every_verb_is_accepted() {
    let fake = Fake::default();
    let app = app(fake.clone());
    for command in [
        r#"{"verb":"kill"}"#,
        r#"{"verb":"nudge","text":"rebase please"}"#,
        r#"{"verb":"prune"}"#,
        r#"{"verb":"retry-setup"}"#,
        r#"{"verb":"model","model":"opus"}"#,
    ] {
        let body = format!(r#"{{"worker":"jig/feat/web","command":{command}}}"#);
        let (status, _) = send(&app, post(&body, &[(jig_api::TOKEN_HEADER, TOKEN)])).await;
        assert_eq!(status, StatusCode::OK, "{body}");
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
async fn a_malformed_command_is_refused() {
    let fake = Fake::default();
    let app = app(fake.clone());
    for body in [
        r#"{"worker":"jig/feat/web","command":{"verb":"shutdown"}}"#,
        r#"{"worker":"not-a-key","command":{"verb":"kill"}}"#,
        "nope",
    ] {
        let (status, body) = send(&app, post(body, &[(jig_api::TOKEN_HEADER, TOKEN)])).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert!(
            serde_json::from_str::<jig_api::ApiError>(&body).is_ok(),
            "{body}"
        );
    }
    assert!(fake.ran.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_refused_command_shows_the_daemons_reason() {
    let app = app(Fake {
        answer: Some("worker 'feat/web' is still running — kill it first".into()),
        ..Default::default()
    });
    let (status, body) = send(
        &app,
        post(
            r#"{"worker":"jig/feat/web","command":{"verb":"prune"}}"#,
            &[(jig_api::TOKEN_HEADER, TOKEN)],
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let error: jig_api::ApiError = serde_json::from_str(&body).unwrap();
    assert!(error.error.contains("still running"), "{}", error.error);
}

#[tokio::test]
async fn a_failed_setup_is_in_the_status() {
    let mut w = worker("feat/broken");
    w.status = WorkerStatus::Failed;
    w.setup = SetupState::Failed {
        phase: SetupPhase::OnCreate,
        exit: SetupExit::Code(1),
        tail: vec!["step 2: boom".into()],
        log_path: Some(PathBuf::from("/tmp/setup.log")),
    };
    let (_, body) = send(
        &app(Fake {
            workers: vec![w],
            ..Default::default()
        }),
        get(jig_api::STATUS, HOST),
    )
    .await;
    let view: jig_api::Status = serde_json::from_str(&body).unwrap();
    let failed = view.repos[0].workers[0].failed_setup.as_ref().unwrap();
    assert_eq!(failed.tail, ["step 2: boom"]);
    assert!(failed.has_log);
}

#[tokio::test]
async fn the_setup_log_is_fetched_by_worker_not_by_path() {
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
    let app = app(Fake {
        workers: vec![w, worker("feat/fine")],
        ..Default::default()
    });
    let log_of = |query: &str| get(&format!("{}{query}", jig_api::SETUP_LOG), HOST);

    let (status, body) = send(&app, log_of("?worker=jig%2Ffeat%2Fbroken")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "step 1\nstep 2: boom\n");

    assert_eq!(
        send(&app, log_of("?worker=jig/feat/fine")).await.0,
        StatusCode::NOT_FOUND
    );
    // A path is not a worker key.
    assert_eq!(
        send(&app, log_of("?worker=/etc/passwd")).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        send(&app, log_of("?worker=jig/../../etc")).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        send(&app, log_of("?worker=jig/feat/nobody")).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(send(&app, log_of("")).await.0, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn an_unknown_api_route_is_an_api_error_not_the_app() {
    let (status, body) = send(&app(Fake::default()), get("/api/v0/nope", HOST)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        serde_json::from_str::<jig_api::ApiError>(&body).is_ok(),
        "{body}"
    );
}

#[tokio::test]
async fn every_response_is_hardened() {
    let app = app(Fake::default());
    let response = app.oneshot(get(jig_api::STATUS, HOST)).await.unwrap();
    let headers = response.headers();
    assert_eq!(headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(headers[header::X_FRAME_OPTIONS], "DENY");
    let csp = headers[header::CONTENT_SECURITY_POLICY].to_str().unwrap();
    assert!(csp.contains("frame-ancestors 'none'"), "{csp}");
    assert!(!csp.contains("unsafe-inline"), "{csp}");
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

#[test]
fn it_binds_loopback_only() {
    let server = WebServer::bind(0).unwrap();
    assert!(server.listener.local_addr().unwrap().ip().is_loopback());
    assert!(server.url().starts_with("http://127.0.0.1:"));
    // Fresh per session, and long enough not to guess.
    assert_ne!(server.token, WebServer::bind(0).unwrap().token);
    assert_eq!(server.token.len(), 32);
}
