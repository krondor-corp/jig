use serde::Deserialize;

use super::super::rest::RestRequest;

#[derive(Deserialize)]
pub(crate) struct RawPrMergeable {
    pub(crate) mergeable: Option<bool>,
    pub(crate) mergeable_state: Option<String>,
}

pub(crate) struct GetPrMergeable {
    pub(crate) pr_number: u64,
}

impl RestRequest for GetPrMergeable {
    type Response = RawPrMergeable;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}", repo, self.pr_number)
    }
}
