use std::process::{Command, Stdio};

use serde::de::DeserializeOwned;

use super::error::{GitHubError, Result};

/// A typed GraphQL request against the GitHub API.
pub(crate) trait GraphQlRequest {
    type Response: DeserializeOwned;
    fn query(&self) -> String;
}

/// Thin wrapper around `gh api graphql`. Stateless — auth and caching are delegated to `gh`.
pub(crate) struct GraphQlClient;

impl GraphQlClient {
    pub(crate) fn call<T: GraphQlRequest>(&self, request: &T) -> Result<T::Response> {
        let query = request.query();
        let output = Command::new("gh")
            .args([
                "api",
                "graphql",
                "--cache",
                "60s",
                "-f",
                &format!("query={}", query),
            ])
            .stdin(Stdio::null())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubError::Cli(format!("gh graphql failed: {}", stderr)));
        }

        let body = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&body).map_err(GitHubError::from)
    }
}
