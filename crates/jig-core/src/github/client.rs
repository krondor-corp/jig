//! GitHub client wrapping `gh` CLI.

use std::collections::HashSet;
use std::path::Path;

use crate::exec::Timeout;

use super::error::{GitHubError, Result};
use super::gh::gh;

/// `gh repo view` reduced to the owner/repo string.
const REPO_VIEW: [&str; 6] = [
    "repo",
    "view",
    "--json",
    "nameWithOwner",
    "-q",
    ".nameWithOwner",
];
use super::graphql::GraphQlClient;
use super::queries::check_runs::GetCheckRuns;
use super::queries::conflicts::GetPrMergeable;
use super::queries::pr_commits::GetPrCommits;
use super::queries::pr_for_branch::{parse_pr_summary, GetPrsForBranch};
use super::queries::pr_state::GetPrState;
use super::queries::review_activity::{
    GetPrCommentsTimestamps, GetPrCommitsActivity, GetPrReviewsActivity,
};
use super::queries::reviews::{
    GetReviewComments, GetReviews, GetUnresolvedThreads, RawReviewThread,
};
use super::rest::RestClient;
use super::types::{
    CheckRun, CheckStatus, PrCommit, PrInfo, PrState, PrStateInfo, ReviewComment, ReviewState,
};

/// GitHub API client using `gh` CLI.
///
/// Auth is delegated entirely to `gh` — it uses `GITHUB_TOKEN`,
/// `gh auth login`, or whatever the user has configured.
pub struct GitHubClient {
    /// Repository in `owner/repo` format.
    pub(crate) repo: String,
    pub(crate) rest: RestClient,
    pub(crate) graphql: GraphQlClient,
    /// How long any one `gh` call gets. Set here rather than threaded
    /// through every query method, which would be a dozen signatures
    /// carrying the same constant.
    pub(crate) timeout: Timeout,
}

impl GitHubClient {
    /// Create a client for the given repository.
    pub fn new(repo: impl Into<String>) -> Self {
        Self::with_timeout(repo, Timeout::NETWORK)
    }

    /// Create a client whose `gh` calls get `timeout` each.
    pub fn with_timeout(repo: impl Into<String>, timeout: Timeout) -> Self {
        Self {
            repo: repo.into(),
            rest: RestClient { timeout },
            graphql: GraphQlClient { timeout },
            timeout,
        }
    }

    /// Detect the repository from the current git remote.
    pub fn from_remote() -> Result<Self> {
        let repo = gh(&REPO_VIEW, None, Timeout::NETWORK)
            .map_err(|e| GitHubError::Cli(format!("Failed to detect GitHub repository: {e}")))?;
        if repo.is_empty() {
            return Err(GitHubError::Other(
                "Could not determine repository name".to_string(),
            ));
        }
        Ok(Self::new(repo))
    }

    /// Detect the repository from a specific repo path (runs `gh` in that directory).
    pub fn from_repo_path(repo_path: &Path) -> Result<Self> {
        let repo = gh(&REPO_VIEW, Some(repo_path), Timeout::NETWORK).map_err(|e| {
            GitHubError::Cli(format!(
                "Failed to detect GitHub repository at {}: {e}",
                repo_path.display()
            ))
        })?;

        if repo.is_empty() {
            return Err(GitHubError::Other(format!(
                "Could not determine repository name at {}",
                repo_path.display()
            )));
        }

        tracing::debug!(
            repo_path = %repo_path.display(),
            owner_repo = %repo,
            "created GitHub client from repo path"
        );

        Ok(Self::new(repo))
    }

    /// Check if `gh` CLI is available and authenticated.
    pub fn is_healthy() -> bool {
        gh(&["auth", "status"], None, Timeout::QUICK).is_ok()
    }

    /// Create a draft PR via `gh pr create`.
    /// Returns the PR URL on success.
    pub fn create_pr(
        &self,
        base: &str,
        head: Option<&str>,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<String> {
        let mut args = vec![
            "pr".to_string(),
            "create".to_string(),
            "--draft".to_string(),
            "--repo".to_string(),
            self.repo.clone(),
            "--base".to_string(),
            base.to_string(),
        ];

        if let Some(h) = head {
            args.push("--head".to_string());
            args.push(h.to_string());
        }

        if let Some(t) = title {
            args.push("--title".to_string());
            args.push(t.to_string());
        }

        if let Some(b) = body {
            args.push("--body".to_string());
            args.push(b.to_string());
        }

        if title.is_none() {
            args.push("--fill".to_string());
        }

        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        gh(&arg_refs, None, self.timeout)
    }

    // ── Query orchestration ───────────────────────────────────────────────────

    /// Get check runs for a git ref (branch name or SHA).
    pub fn get_check_runs(&self, git_ref: &str) -> Result<Vec<CheckRun>> {
        let response = self.rest.call(
            &GetCheckRuns {
                git_ref: git_ref.to_string(),
            },
            &self.repo,
        )?;

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

    /// Check if a PR has merge conflicts.
    pub fn has_conflicts(&self, pr_number: u64) -> Result<bool> {
        let pr = self.rest.call(&GetPrMergeable { pr_number }, &self.repo)?;
        Ok(pr.mergeable_state.as_deref() == Some("dirty") || pr.mergeable == Some(false))
    }

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

    /// Get PR info for a branch (any state: open, closed, or merged).
    pub fn get_pr_for_branch(&self, branch: &str) -> Result<Option<PrInfo>> {
        let prs = self.rest.call(
            &GetPrsForBranch {
                branch: branch.to_string(),
            },
            &self.repo,
        )?;

        let Some(pr) = prs.into_iter().next() else {
            return Ok(None);
        };

        Ok(Some(parse_pr_summary(pr, branch)))
    }

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

    /// Get review comments on a PR.
    ///
    /// Excludes `PENDING` reviews — those are in-progress drafts that the
    /// reviewer hasn't submitted yet.
    pub fn get_reviews(&self, pr_number: u64) -> Result<Vec<ReviewComment>> {
        let reviews = self.rest.call(&GetReviews { pr_number }, &self.repo)?;

        Ok(reviews
            .into_iter()
            .filter_map(|r| {
                let state = match r.state.as_str() {
                    "APPROVED" => ReviewState::Approved,
                    "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
                    "COMMENTED" => ReviewState::Commented,
                    "DISMISSED" => ReviewState::Dismissed,
                    "PENDING" => return None,
                    _ => return None,
                };

                Some(ReviewComment {
                    body: r.body,
                    path: None,
                    line: None,
                    state,
                    author: r.user.login,
                })
            })
            .collect())
    }

    /// Get inline review comments from **unresolved** threads on a PR.
    ///
    /// Uses the GraphQL API to fetch only unresolved review threads, so
    /// resolved conversations don't trigger review nudges. Falls back to
    /// the REST endpoint (all comments, replies excluded) if GraphQL fails.
    pub fn get_review_comments(&self, pr_number: u64) -> Result<Vec<ReviewComment>> {
        if let Some((owner, name)) = self.repo.split_once('/') {
            match self.graphql.call(&GetUnresolvedThreads {
                owner: owner.to_string(),
                name: name.to_string(),
                pr_number,
            }) {
                Ok(response) => {
                    let threads = response.data.repository.pull_request.review_threads.nodes;
                    return Ok(open_submitted_threads(threads));
                }
                Err(e) => tracing::debug!(
                    pr_number,
                    error = %e,
                    "graphql review threads failed; falling back to REST"
                ),
            }
        }

        let comments = self
            .rest
            .call(&GetReviewComments { pr_number }, &self.repo)?;
        let reviews = self.rest.call(&GetReviews { pr_number }, &self.repo)?;
        let pending = pending_review_ids(reviews.iter().map(|r| (r.id, r.state.as_str())));

        Ok(comments
            .into_iter()
            .filter(|c| c.in_reply_to_id.is_none())
            .filter(|c| !in_pending_review(c.pull_request_review_id, &pending))
            .map(|c| ReviewComment {
                body: c.body,
                path: c.path,
                line: c.line.or(c.original_line),
                state: ReviewState::Commented,
                author: c.user.login,
            })
            .collect())
    }

    /// Check whether the latest commit on a PR is newer than the latest review activity.
    ///
    /// Returns `true` if the developer has pushed commits after the most recent
    /// review or inline comment, meaning the feedback has likely been addressed
    /// and nudging would be premature (the ball is in the reviewer's court).
    ///
    /// Returns `false` (= should nudge) on any API error or if there are no commits.
    pub fn dev_pushed_after_reviews(&self, pr_number: u64) -> bool {
        let commits = match self
            .rest
            .call(&GetPrCommitsActivity { pr_number }, &self.repo)
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

        let reviews = match self
            .rest
            .call(&GetPrReviewsActivity { pr_number }, &self.repo)
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

        let comments = match self
            .rest
            .call(&GetPrCommentsTimestamps { pr_number }, &self.repo)
        {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!(pr_number, error = %e, "dev_pushed_after_reviews: comments API failed");
                return false;
            }
        };
        let pending = pending_review_ids(reviews.iter().map(|r| (r.id, r.state.as_str())));
        let latest_comment_date = comments
            .iter()
            .filter(|c| !in_pending_review(c.pull_request_review_id, &pending))
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

/// Review/comment state GitHub uses for a review its author hasn't submitted.
///
/// A draft review is visible to its own author through the API — and the
/// daemon's `gh` is usually logged in as the person reviewing — so drafts
/// must be filtered out everywhere feedback is read, or the daemon nudges
/// workers about comments the reviewer is still writing.
const PENDING: &str = "PENDING";

/// First comment of each unresolved thread, skipping threads opened in a
/// review the reviewer hasn't submitted yet — those are drafts, not feedback.
fn open_submitted_threads(threads: Vec<RawReviewThread>) -> Vec<ReviewComment> {
    threads
        .into_iter()
        .filter(|t| !t.is_resolved)
        .filter_map(|t| t.comments.nodes.into_iter().next())
        .filter(|c| c.state != PENDING)
        .map(|c| ReviewComment {
            body: c.body,
            path: c.path,
            line: c.line,
            state: ReviewState::Commented,
            author: c.author.login,
        })
        .collect()
}

fn pending_review_ids<'a>(reviews: impl Iterator<Item = (u64, &'a str)>) -> HashSet<u64> {
    reviews
        .filter(|(_, state)| *state == PENDING)
        .map(|(id, _)| id)
        .collect()
}

fn in_pending_review(review_id: Option<u64>, pending: &HashSet<u64>) -> bool {
    review_id.is_some_and(|id| pending.contains(&id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_repo() {
        let client = GitHubClient::new("owner/repo");
        assert_eq!(client.repo, "owner/repo");
    }

    #[test]
    fn is_healthy_does_not_panic() {
        let _ = GitHubClient::is_healthy();
    }

    fn thread(resolved: bool, state: &str, body: &str) -> serde_json::Value {
        serde_json::json!({
            "isResolved": resolved,
            "comments": { "nodes": [{
                "state": state,
                "body": body,
                "path": "src/lib.rs",
                "line": 3,
                "author": { "login": "reviewer" }
            }]}
        })
    }

    #[test]
    fn draft_review_threads_are_not_feedback() {
        let threads: Vec<RawReviewThread> = serde_json::from_value(serde_json::json!([
            thread(false, "SUBMITTED", "please rename"),
            thread(false, "PENDING", "still writing this review"),
            thread(true, "SUBMITTED", "already resolved"),
        ]))
        .unwrap();

        let open: Vec<_> = open_submitted_threads(threads)
            .into_iter()
            .map(|c| c.body)
            .collect();
        assert_eq!(open, vec!["please rename"]);
    }

    #[test]
    fn comments_in_a_pending_review_are_dropped() {
        let pending = pending_review_ids(
            [(1, "COMMENTED"), (2, "PENDING"), (3, "CHANGES_REQUESTED")].into_iter(),
        );
        assert!(in_pending_review(Some(2), &pending));
        assert!(!in_pending_review(Some(1), &pending));
        assert!(!in_pending_review(None, &pending));
    }
}
