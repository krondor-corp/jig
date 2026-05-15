use serde::Deserialize;

use super::super::client::GitHubClient;
use super::super::error::Result;
use super::super::types::{CheckRun, CheckStatus};

#[derive(Deserialize)]
struct RawCheckRunsResponse {
    check_runs: Vec<RawCheckRun>,
}

#[derive(Deserialize)]
struct RawCheckRun {
    name: String,
    status: String,
    conclusion: Option<String>,
    details_url: Option<String>,
}

impl GitHubClient {
    /// Get check runs for a git ref (branch name or SHA).
    pub fn get_check_runs(&self, git_ref: &str) -> Result<Vec<CheckRun>> {
        let response: RawCheckRunsResponse = self.gh_api(&format!(
            "repos/{}/commits/{}/check-runs",
            self.repo, git_ref
        ))?;

        Ok(response
            .check_runs
            .into_iter()
            .map(|r| CheckRun {
                name: r.name,
                status: match r.status.as_str() {
                    "completed" => CheckStatus::Completed,
                    "in_progress" => CheckStatus::InProgress,
                    _ => CheckStatus::Queued,
                },
                conclusion: r.conclusion,
                details_url: r.details_url,
            })
            .collect())
    }

    /// Get failed check runs for a ref.
    pub fn get_failed_checks(&self, git_ref: &str) -> Result<Vec<CheckRun>> {
        let all = self.get_check_runs(git_ref)?;
        Ok(all.into_iter().filter(|r| r.is_failure()).collect())
    }
}
