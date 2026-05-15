use std::process::{Command, Stdio};

use serde::de::DeserializeOwned;

use super::error::{GitHubError, Result};

/// A typed REST request against the GitHub API.
pub(crate) trait RestRequest {
    type Response: DeserializeOwned;
    fn endpoint(&self, repo: &str) -> String;
}

/// Thin wrapper around `gh api`. Stateless — auth and caching are delegated to `gh`.
pub(crate) struct RestClient;

impl RestClient {
    pub(crate) fn call<T: RestRequest>(&self, request: &T, repo: &str) -> Result<T::Response> {
        let endpoint = request.endpoint(repo);
        let output = Command::new("gh")
            .args(["api", &endpoint, "--cache", "60s"])
            .stdin(Stdio::null())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubError::Cli(format!("gh api failed: {}", stderr)));
        }

        let body = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&body).map_err(GitHubError::from)
    }
}
