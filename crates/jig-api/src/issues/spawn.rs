//! `POST /api/v0/issues/spawn`: start a worker for an issue, on the machine
//! whose repo it is in.

use reqwest::{Client as HttpClient, RequestBuilder, Url};
use serde::{Deserialize, Serialize};

use crate::{ApiRequest, CommandOutcome};

pub const PATH: &str = "/api/v0/issues/spawn";

/// Needs the session token. Answers once the worker's setup has run, which
/// can take as long as `on_create` does. Starts a real agent.
/// **Daemon route:** [`PATH`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnRequest {
    /// The repo's directory name, as in [`super::RepoIssues::repo`].
    pub repo: String,
    /// The issue's id, as in [`super::Issue::id`].
    pub issue: String,
    /// A [`crate::Machine::node`]; `None` for the machine serving the page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
}

impl ApiRequest for SpawnRequest {
    type Response = CommandOutcome;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        http.post(base.join(PATH).expect("static path")).json(&self)
    }
}
