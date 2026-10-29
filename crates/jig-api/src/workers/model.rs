//! `POST /api/v0/workers/model`: switch a worker's agent to another model.

use reqwest::{Client as HttpClient, RequestBuilder, Url};
use serde::{Deserialize, Serialize};

use crate::ApiRequest;

use super::CommandOutcome;

pub const PATH: &str = "/api/v0/workers/model";

/// Needs the session token. **Daemon route:** [`PATH`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRequest {
    /// `repo/branch`, as in [`crate::Worker::key`].
    pub worker: String,
    /// As the agent's `/model` command takes it, e.g. `opus`.
    pub model: String,
}

impl ApiRequest for ModelRequest {
    type Response = CommandOutcome;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        http.post(base.join(PATH).expect("static path")).json(&self)
    }
}
