use serde::de::DeserializeOwned;

use crate::exec::Timeout;

use super::error::Result;
use super::gh::gh;

/// A typed GraphQL request against the GitHub API.
pub(crate) trait GraphQlRequest {
    type Response: DeserializeOwned;
    fn query(&self) -> String;
}

/// Thin wrapper around `gh api graphql`. Auth and caching are delegated to `gh`.
pub(crate) struct GraphQlClient {
    pub(crate) timeout: Timeout,
}

impl GraphQlClient {
    pub(crate) fn call<T: GraphQlRequest>(&self, request: &T) -> Result<T::Response> {
        let query = request.query();
        let body = gh(
            &[
                "api",
                "graphql",
                "--cache",
                "60s",
                "-f",
                &format!("query={query}"),
            ],
            None,
            self.timeout,
        )?;
        serde_json::from_str(&body).map_err(Into::into)
    }
}
