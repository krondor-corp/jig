use serde::Deserialize;

use super::super::client::GitHubClient;

#[derive(Deserialize)]
struct RawPrCommit {
    commit: RawCommitActivity,
}

#[derive(Deserialize)]
struct RawCommitActivity {
    committer: RawCommitter,
}

#[derive(Deserialize)]
struct RawCommitter {
    date: String,
}

#[derive(Deserialize)]
struct RawReview {
    state: String,
    submitted_at: Option<String>,
}

#[derive(Deserialize)]
struct RawCommentTimestamp {
    created_at: String,
}

impl GitHubClient {
    /// Check whether the latest commit on a PR is newer than the latest review activity.
    ///
    /// Returns `true` if the developer has pushed commits after the most recent
    /// review or inline comment, meaning the feedback has likely been addressed
    /// and nudging would be premature (the ball is in the reviewer's court).
    ///
    /// Returns `false` (= should nudge) on any API error or if there are no commits.
    pub fn dev_pushed_after_reviews(&self, pr_number: u64) -> bool {
        let commits: Vec<RawPrCommit> = match self
            .gh_api_json(&format!("repos/{}/pulls/{}/commits", self.repo, pr_number))
        {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!(pr_number, error = %e, "dev_pushed_after_reviews: commits API failed");
                return false;
            }
        };
        let latest_commit_date = commits
            .last()
            .map(|c| c.commit.committer.date.as_str())
            .unwrap_or("");

        if latest_commit_date.is_empty() {
            tracing::debug!(pr_number, "dev_pushed_after_reviews: no commit date found");
            return false;
        }

        let reviews: Vec<RawReview> = match self
            .gh_api_json(&format!("repos/{}/pulls/{}/reviews", self.repo, pr_number))
        {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!(pr_number, error = %e, "dev_pushed_after_reviews: reviews API failed");
                return false;
            }
        };
        let latest_review_date = reviews
            .iter()
            .filter(|r| r.state != "PENDING")
            .filter_map(|r| r.submitted_at.as_deref())
            .max()
            .unwrap_or("");

        let comments: Vec<RawCommentTimestamp> = match self
            .gh_api_json(&format!("repos/{}/pulls/{}/comments", self.repo, pr_number))
        {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!(pr_number, error = %e, "dev_pushed_after_reviews: comments API failed");
                return false;
            }
        };
        let latest_comment_date = comments
            .iter()
            .map(|c| c.created_at.as_str())
            .max()
            .unwrap_or("");

        let latest_feedback = std::cmp::max(latest_review_date, latest_comment_date);

        if latest_feedback.is_empty() {
            tracing::debug!(
                pr_number,
                "dev_pushed_after_reviews: no review activity found"
            );
            return false;
        }

        let result = latest_commit_date > latest_feedback;
        tracing::info!(
            pr_number,
            latest_commit_date,
            latest_review_date,
            latest_comment_date,
            %latest_feedback,
            result,
            "dev_pushed_after_reviews"
        );
        result
    }
}
