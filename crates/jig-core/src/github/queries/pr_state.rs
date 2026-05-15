use serde::Deserialize;

use super::super::rest::RestRequest;

#[derive(Deserialize)]
pub(crate) struct RawPrState {
    pub(crate) merged: bool,
    pub(crate) state: String,
    pub(crate) draft: bool,
    pub(crate) head: RawPrHead,
}

#[derive(Deserialize)]
pub(crate) struct RawPrHead {
    pub(crate) sha: String,
}

pub(crate) struct GetPrState {
    pub(crate) pr_number: u64,
}

impl RestRequest for GetPrState {
    type Response = RawPrState;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}", repo, self.pr_number)
    }
}
