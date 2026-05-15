//! GitHub client wrapping `gh` CLI.

use std::path::Path;
use std::process::{Command, Stdio};

use super::error::{GitHubError, Result};
use super::graphql::GraphQlClient;
use super::queries::check_runs::GetCheckRuns;
use super::queries::conflicts::GetPrMergeable;
use super::queries::pr_commits::GetPrCommits;
use super::queries::pr_for_branch::{parse_pr_summary, GetPrsForBranch};
use super::queries::pr_state::GetPrState;
use super::queries::review_activity::{
    GetPrCommentsTimestamps, GetPrCommitsActivity, GetPrReviewsActivity,
};
use super::queries::reviews::{GetReviewComments, GetReviews, GetUnresolvedThreads};
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
}

impl GitHubClient {
    /// Create a client for the given repository.
    pub fn new(repo: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            rest: RestClient,
            graphql: GraphQlClient,
        }
    }

    /// Detect the repository from the current git remote.
    pub fn from_remote() -> Result<Self> {
        let output = Command::new("gh")
            .args([
                "repo",
                "view",
                "--json",
                "nameWithOwner",
                "-q",
                ".nameWithOwner",
            ])
            .stdin(Stdio::null())
            .output()?;

        if !output.status.success() {
            return Err(GitHubError::Cli(
                "Failed to detect GitHub repository. Is `gh` authenticated?".into(),
            ));
        }

        let repo = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if repo.is_empty() {
            return Err(GitHubError::Other(
                "Could not determine repository name".into(),
            ));
        }

        Ok(Self::new(repo))
    }

    /// Detect the repository from a specific repo path (runs `gh` in that directory).
    pub fn from_repo_path(repo_path: &Path) -> Result<Self> {
        let output = Command::new("gh")
            .args([
                "repo",
                "view",
                "--json",
                "nameWithOwner",
                "-q",
                ".nameWithOwner",
            ])
            .current_dir(repo_path)
            .stdin(Stdio::null())
            .output()?;

        if !output.status.success() {
            return Err(GitHubError::Cli(format!(
                "Failed to detect GitHub repository at {}. Is `gh` authenticated?",
                repo_path.display()
            )));
        }

        let repo = String::from_utf8_lossy(&output.stdout).trim().to_string();
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
        Command::new("gh")
            .args(["auth", "status"])
            .stdin(Stdio::null())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
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

        let output = Command::new("gh")
            .args(&args)
            .stdin(Stdio::null())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubError::Cli(format!("gh pr create failed: {}", stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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
                    return Ok(threads
                        .into_iter()
                        .filter(|t| !t.is_resolved)
                        .filter_map(|t| t.comments.nodes.into_iter().next())
                        .map(|c| ReviewComment {
                            body: c.body,
                            path: c.path,
                            line: c.line,
                            state: ReviewState::Commented,
                            author: c.author.login,
                        })
                        .collect());
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

        Ok(comments
            .into_iter()
            .filter(|c| c.in_reply_to_id.is_none())
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
}
