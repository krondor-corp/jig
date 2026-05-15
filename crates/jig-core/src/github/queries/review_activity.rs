use serde::Deserialize;

use super::super::rest::RestRequest;

// ── REST primitives ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct RawActivityCommit {
    pub(crate) commit: RawActivityCommitMeta,
}

#[derive(Deserialize)]
pub(crate) struct RawActivityCommitMeta {
    pub(crate) committer: RawActivityCommitter,
}

#[derive(Deserialize)]
pub(crate) struct RawActivityCommitter {
    pub(crate) date: String,
}

#[derive(Deserialize)]
pub(crate) struct RawActivityReview {
    pub(crate) state: String,
    pub(crate) submitted_at: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct RawActivityCommentTimestamp {
    pub(crate) created_at: String,
}

pub(crate) struct GetPrCommitsActivity {
    pub(crate) pr_number: u64,
}

impl RestRequest for GetPrCommitsActivity {
    type Response = Vec<RawActivityCommit>;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}/commits", repo, self.pr_number)
    }
}

pub(crate) struct GetPrReviewsActivity {
    pub(crate) pr_number: u64,
}

impl RestRequest for GetPrReviewsActivity {
    type Response = Vec<RawActivityReview>;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}/reviews", repo, self.pr_number)
    }
}

pub(crate) struct GetPrCommentsTimestamps {
    pub(crate) pr_number: u64,
}

impl RestRequest for GetPrCommentsTimestamps {
    type Response = Vec<RawActivityCommentTimestamp>;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}/comments", repo, self.pr_number)
    }
}
