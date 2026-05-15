use serde::Deserialize;

use super::super::client::GitHubClient;
use super::super::error::Result;
use super::super::types::{PrState, PrStateInfo};

#[derive(Deserialize)]
struct RawPrState {
    merged: bool,
    state: String,
    draft: bool,
    head: RawPrHead,
}

#[derive(Deserialize)]
struct RawPrHead {
    sha: String,
}

impl GitHubClient {
    /// Get the current state of a PR (open, closed, or merged) and whether it's a draft.
    pub fn get_pr_state(&self, pr_number: u64) -> Result<PrStateInfo> {
        let pr: RawPrState = self.gh_api(&format!("repos/{}/pulls/{}", self.repo, pr_number))?;

        let state = if pr.merged {
            PrState::Merged
        } else if pr.state == "closed" {
            PrState::Closed
        } else {
            PrState::Open
        };

        Ok(PrStateInfo {
            state,
            is_draft: pr.draft,
            head_sha: Some(pr.head.sha),
        })
    }
}
