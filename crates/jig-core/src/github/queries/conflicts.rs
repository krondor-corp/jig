use serde::Deserialize;

use super::super::client::GitHubClient;
use super::super::error::Result;
use super::super::rest::RestRequest;

#[derive(Deserialize)]
struct RawPrMergeable {
    mergeable: Option<bool>,
    mergeable_state: Option<String>,
}

struct GetPrMergeable {
    pr_number: u64,
}

impl RestRequest for GetPrMergeable {
    type Response = RawPrMergeable;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}", repo, self.pr_number)
    }
}

impl GitHubClient {
    /// Check if a PR has merge conflicts.
    pub fn has_conflicts(&self, pr_number: u64) -> Result<bool> {
        let pr = self.rest.call(&GetPrMergeable { pr_number }, &self.repo)?;
        Ok(pr.mergeable_state.as_deref() == Some("dirty") || pr.mergeable == Some(false))
    }
}
