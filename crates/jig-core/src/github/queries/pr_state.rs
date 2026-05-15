use serde::Deserialize;

use super::super::client::GitHubClient;
use super::super::error::Result;
use super::super::rest::RestRequest;
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

struct GetPrState {
    pr_number: u64,
}

impl RestRequest for GetPrState {
    type Response = RawPrState;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}", repo, self.pr_number)
    }
}

impl GitHubClient {
    /// Get the current state of a PR (open, closed, or merged) and whether it's a draft.
    pub fn get_pr_state(&self, pr_number: u64) -> Result<PrStateInfo> {
        let pr = self.rest.call(&GetPrState { pr_number }, &self.repo)?;

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
