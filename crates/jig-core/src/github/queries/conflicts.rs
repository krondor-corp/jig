use serde::Deserialize;

use super::super::client::GitHubClient;
use super::super::error::Result;

#[derive(Deserialize)]
struct RawPrMergeable {
    mergeable: Option<bool>,
    mergeable_state: Option<String>,
}

impl GitHubClient {
    /// Check if a PR has merge conflicts.
    pub fn has_conflicts(&self, pr_number: u64) -> Result<bool> {
        let pr: RawPrMergeable =
            self.gh_api(&format!("repos/{}/pulls/{}", self.repo, pr_number))?;
        Ok(pr.mergeable_state.as_deref() == Some("dirty") || pr.mergeable == Some(false))
    }
}
