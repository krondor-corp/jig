use serde::Deserialize;

use super::super::rest::RestRequest;

#[derive(Deserialize)]
pub(crate) struct RawPrCommit {
    pub(crate) sha: String,
    pub(crate) commit: RawCommitMeta,
}

#[derive(Deserialize)]
pub(crate) struct RawCommitMeta {
    pub(crate) message: String,
}

pub(crate) struct GetPrCommits {
    pub(crate) pr_number: u64,
}

impl RestRequest for GetPrCommits {
    type Response = Vec<RawPrCommit>;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}/commits", repo, self.pr_number)
    }
}
