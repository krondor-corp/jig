use serde::Deserialize;

use super::super::rest::RestRequest;

#[derive(Deserialize)]
pub(crate) struct RawCheckRunsResponse {
    pub(crate) check_runs: Vec<RawCheckRun>,
}

#[derive(Deserialize)]
pub(crate) struct RawCheckRun {
    pub(crate) name: String,
    pub(crate) status: String,
    pub(crate) conclusion: Option<String>,
    pub(crate) details_url: Option<String>,
}

pub(crate) struct GetCheckRuns {
    pub(crate) git_ref: String,
}

impl RestRequest for GetCheckRuns {
    type Response = RawCheckRunsResponse;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/commits/{}/check-runs", repo, self.git_ref)
    }
}
