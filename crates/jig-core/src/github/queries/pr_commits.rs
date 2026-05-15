use serde::Deserialize;

use super::super::client::GitHubClient;
use super::super::error::Result;
use super::super::rest::RestRequest;
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

struct GetPrCommits {
    pr_number: u64,
}

impl RestRequest for GetPrCommits {
    type Response = Vec<RawPrCommit>;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}/commits", repo, self.pr_number)
    }
}

impl GitHubClient {
    /// Get commits on a PR.
    pub fn get_pr_commits(&self, pr_number: u64) -> Result<Vec<PrCommit>> {
        let commits = self.rest.call(&GetPrCommits { pr_number }, &self.repo)?;

        Ok(commits
            .into_iter()
            .map(|c| PrCommit {
                sha: c.sha.chars().take(7).collect(),
                message: c.commit.message.lines().next().unwrap_or("").to_string(),
            })
            .collect())
    }
}
