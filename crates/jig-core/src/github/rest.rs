use serde::de::DeserializeOwned;

use crate::exec::Timeout;

use super::error::Result;
use super::gh::gh;

/// A typed REST request against the GitHub API.
pub(crate) trait RestRequest {
    type Response: DeserializeOwned;
    fn endpoint(&self, repo: &str) -> String;
}

/// Thin wrapper around `gh api`. Auth and caching are delegated to `gh`.
pub(crate) struct RestClient {
    pub(crate) timeout: Timeout,
}

impl RestClient {
    pub(crate) fn call<T: RestRequest>(&self, request: &T, repo: &str) -> Result<T::Response> {
        let endpoint = request.endpoint(repo);
        let body = gh(&["api", &endpoint, "--cache", "60s"], None, self.timeout)?;
        serde_json::from_str(&body).map_err(Into::into)
    }
}
