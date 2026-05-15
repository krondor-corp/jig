use serde::Deserialize;

use super::super::client::GitHubClient;
use super::super::error::Result;
use super::super::types::PrCommit;

#[derive(Deserialize)]
struct RawPrCommit {
    sha: String,
    commit: RawCommitMeta,
}

#[derive(Deserialize)]
struct RawCommitMeta {
    message: String,
}

impl GitHubClient {
    /// Get commits on a PR.
    pub fn get_pr_commits(&self, pr_number: u64) -> Result<Vec<PrCommit>> {
        let commits: Vec<RawPrCommit> =
            self.gh_api_json(&format!("repos/{}/pulls/{}/commits", self.repo, pr_number))?;

        Ok(commits
            .into_iter()
            .map(|c| PrCommit {
                sha: c.sha.chars().take(7).collect(),
                message: c
                    .commit
                    .message
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }
}
