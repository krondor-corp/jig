//! Typed calls to the daemon, over `fetch`.
//!
//! Every call goes through `jig_api`'s [`Client`]: the same `ApiRequest`
//! types the daemon's routes deserialize and its tests send, via reqwest's
//! wasm backend. Commands use a [`Client::with_token`] client, as the API
//! requires.

use jig_api::{
    ApiRequest, Client, Fleet, FleetRequest, Issues, IssuesRequest, KillRequest, ModelRequest,
    NudgeRequest, PruneRequest, RefreshIssuesRequest, RetrySetupRequest, SessionRequest,
    SetupLogRequest, SpawnRequest, Url,
};

/// This page's origin: the daemon that served it.
fn base() -> Result<Url, String> {
    let origin = web_sys::window()
        .ok_or("no window")?
        .location()
        .origin()
        .map_err(|_| "no origin".to_string())?;
    Url::parse(&origin).map_err(|e| e.to_string())
}

/// A client for the read-only routes.
fn reader() -> Result<Client, String> {
    Client::new(&base()?).map_err(|e| e.to_string())
}

/// A client that can send commands.
fn commander(token: &str) -> Result<Client, String> {
    Client::with_token(&base()?, token).map_err(|e| e.to_string())
}

async fn call<R: ApiRequest>(client: Client, req: R) -> Result<R::Response, String> {
    client.call(req).await.map_err(|e| e.to_string())
}

/// The command token.
pub async fn session() -> Result<String, String> {
    Ok(call(reader()?, SessionRequest).await?.token)
}

/// Every machine's frame, this one first.
pub async fn fleet() -> Result<Fleet, String> {
    call(reader()?, FleetRequest).await
}

/// A machine's issues (`machine`: its node id, `None` for this one).
pub async fn issues(machine: Option<String>) -> Result<Issues, String> {
    call(reader()?, IssuesRequest { machine }).await
}

/// The end of a worker's `setup.log`.
pub async fn setup_log(worker: &str) -> Result<String, String> {
    let request = SetupLogRequest {
        worker: worker.to_string(),
    };
    Ok(call(reader()?, request).await?.text)
}

/// Something the page can ask a daemon to do: a `jig_api::workers` verb,
/// or an `issues` spawn or refresh. All need the session token.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Kill(KillRequest),
    Nudge(NudgeRequest),
    Prune(PruneRequest),
    RetrySetup(RetrySetupRequest),
    Model(ModelRequest),
    Spawn(SpawnRequest),
    RefreshIssues(RefreshIssuesRequest),
}

impl Action {
    /// The CLI verb, e.g. `retry-setup`.
    pub fn name(&self) -> &'static str {
        match self {
            Action::Kill(_) => "kill",
            Action::Nudge(_) => "nudge",
            Action::Prune(_) => "prune",
            Action::RetrySetup(_) => "retry-setup",
            Action::Model(_) => "model",
            Action::Spawn(_) => "spawn",
            Action::RefreshIssues(_) => "refresh",
        }
    }

    /// What it acts on: `repo/branch`, an issue, or the issue list.
    pub fn target(&self) -> &str {
        match self {
            Action::Kill(r) => &r.worker,
            Action::Nudge(r) => &r.worker,
            Action::Prune(r) => &r.worker,
            Action::RetrySetup(r) => &r.worker,
            Action::Model(r) => &r.worker,
            Action::Spawn(r) => &r.issue,
            Action::RefreshIssues(_) => "issues",
        }
    }

    /// The same verb, for the machine whose node id is `machine` (`None`:
    /// the machine serving the page).
    pub fn on(self, machine: Option<String>) -> Self {
        match self {
            Action::Kill(r) => Action::Kill(KillRequest { machine, ..r }),
            Action::Nudge(r) => Action::Nudge(NudgeRequest { machine, ..r }),
            Action::Prune(r) => Action::Prune(PruneRequest { machine, ..r }),
            Action::RetrySetup(r) => Action::RetrySetup(RetrySetupRequest { machine, ..r }),
            Action::Model(r) => Action::Model(ModelRequest { machine, ..r }),
            Action::Spawn(r) => Action::Spawn(SpawnRequest { machine, ..r }),
            Action::RefreshIssues(_) => Action::RefreshIssues(RefreshIssuesRequest { machine }),
        }
    }

    /// Send it with `token`. The daemon's message, or its reason for
    /// refusing.
    pub async fn send(self, token: &str) -> Result<String, String> {
        let client = commander(token)?;
        let outcome = match self {
            Action::Kill(r) => call(client, r).await,
            Action::Nudge(r) => call(client, r).await,
            Action::Prune(r) => call(client, r).await,
            Action::RetrySetup(r) => call(client, r).await,
            Action::Model(r) => call(client, r).await,
            Action::Spawn(r) => call(client, r).await,
            Action::RefreshIssues(r) => call(client, r).await,
        }?;
        Ok(outcome.message)
    }
}
