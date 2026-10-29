//! `POST /api/v0/workers/prune`: remove a stopped worker's worktree,
//! uncommitted changes included. Refused while its window is open.

use reqwest::{Client as HttpClient, RequestBuilder, Url};
use serde::{Deserialize, Serialize};

use crate::ApiRequest;

use super::CommandOutcome;

pub const PATH: &str = "/api/v0/workers/prune";

/// Needs the session token. **Daemon route:** [`PATH`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruneRequest {
    /// `repo/branch`, as in [`crate::Worker::key`].
    pub worker: String,
}

impl ApiRequest for PruneRequest {
    type Response = CommandOutcome;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        http.post(base.join(PATH).expect("static path")).json(&self)
    }
}
