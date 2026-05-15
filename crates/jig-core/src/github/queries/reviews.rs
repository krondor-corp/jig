use serde::Deserialize;

use super::super::client::GitHubClient;
use super::super::error::{GitHubError, Result};
use super::super::types::{ReviewComment, ReviewState};

#[derive(Deserialize)]
struct RawReview {
    state: String,
    body: String,
    user: RawUser,
}

#[derive(Deserialize)]
struct RawUser {
    login: String,
}

#[derive(Deserialize)]
struct RawReviewComment {
    body: String,
    path: Option<String>,
    line: Option<u64>,
    original_line: Option<u64>,
    user: RawUser,
    in_reply_to_id: Option<u64>,
}

#[derive(Deserialize)]
struct GraphQlResponse<T> {
    data: T,
}

#[derive(Deserialize)]
struct RepositoryQuery {
    repository: RepositoryData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryData {
    pull_request: PullRequestData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullRequestData {
    review_threads: ReviewThreadsConnection,
}

#[derive(Deserialize)]
struct ReviewThreadsConnection {
    nodes: Vec<RawReviewThread>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawReviewThread {
    is_resolved: bool,
    comments: CommentsConnection,
}

#[derive(Deserialize)]
struct CommentsConnection {
    nodes: Vec<RawThreadComment>,
}

#[derive(Deserialize)]
struct RawThreadComment {
    body: String,
    path: Option<String>,
    line: Option<u64>,
    author: RawAuthor,
}

#[derive(Deserialize)]
struct RawAuthor {
    login: String,
}

impl GitHubClient {
    /// Get review comments on a PR.
    ///
    /// Excludes `PENDING` reviews — those are in-progress drafts that the
    /// reviewer hasn't submitted yet.
    pub fn get_reviews(&self, pr_number: u64) -> Result<Vec<ReviewComment>> {
        let reviews: Vec<RawReview> =
            self.gh_api(&format!("repos/{}/pulls/{}/reviews", self.repo, pr_number))?;

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
        match self.get_unresolved_review_comments_graphql(pr_number) {
            Ok(comments) => return Ok(comments),
            Err(e) => tracing::debug!(
                pr_number,
                error = %e,
                "graphql review threads failed; falling back to REST"
            ),
        }

        let comments: Vec<RawReviewComment> =
            self.gh_api(&format!("repos/{}/pulls/{}/comments", self.repo, pr_number))?;

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

    /// Fetch review comments from unresolved threads only (via GraphQL).
    fn get_unresolved_review_comments_graphql(&self, pr_number: u64) -> Result<Vec<ReviewComment>> {
        let (owner, name) = self
            .repo
            .split_once('/')
            .ok_or_else(|| GitHubError::Other("invalid repo format".to_string()))?;

        let query = format!(
            r#"{{
              repository(owner: "{owner}", name: "{name}") {{
                pullRequest(number: {pr_number}) {{
                  reviewThreads(first: 100) {{
                    nodes {{
                      isResolved
                      comments(first: 1) {{
                        nodes {{
                          body
                          path
                          line: originalLine
                          author {{ login }}
                        }}
                      }}
                    }}
                  }}
                }}
              }}
            }}"#,
        );

        let response: GraphQlResponse<RepositoryQuery> = self.gh_graphql(&query)?;

        let threads = response.data.repository.pull_request.review_threads.nodes;

        let mut comments = Vec::new();
        for thread in threads {
            if thread.is_resolved {
                continue;
            }
            if let Some(first) = thread.comments.nodes.into_iter().next() {
                comments.push(ReviewComment {
                    body: first.body,
                    path: first.path,
                    line: first.line,
                    state: ReviewState::Commented,
                    author: first.author.login,
                });
            }
        }

        Ok(comments)
    }
}
