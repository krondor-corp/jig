//! `POST /api/v0/workers/nudge`: type a message into a worker's agent.

use reqwest::{Client as HttpClient, RequestBuilder, Url};
use serde::{Deserialize, Serialize};

use crate::ApiRequest;

use super::CommandOutcome;

pub const PATH: &str = "/api/v0/workers/nudge";

/// Needs the session token. **Daemon route:** [`PATH`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NudgeRequest {
    /// `repo/branch`, as in [`crate::Worker::key`].
    pub worker: String,
    /// Free text, sent as one message.
    pub text: String,
}

impl ApiRequest for NudgeRequest {
    type Response = CommandOutcome;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        http.post(base.join(PATH).expect("static path")).json(&self)
    }
}
