//! `POST /api/v0/workers/retry-setup`: re-run a failed `on_create` in the
//! existing worktree, and relaunch the agent if it passes.

use reqwest::{Client as HttpClient, RequestBuilder, Url};
use serde::{Deserialize, Serialize};

use crate::ApiRequest;

use super::CommandOutcome;

pub const PATH: &str = "/api/v0/workers/retry-setup";

/// Needs the session token. Answers once setup finishes, which can take as
/// long as `on_create` does. **Daemon route:** [`PATH`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrySetupRequest {
    /// `repo/branch`, as in [`crate::Worker::key`].
    pub worker: String,
}

impl ApiRequest for RetrySetupRequest {
    type Response = CommandOutcome;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        http.post(base.join(PATH).expect("static path")).json(&self)
    }
}
