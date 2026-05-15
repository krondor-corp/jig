use serde::Deserialize;

use super::super::error::{GitHubError, Result};
use super::super::graphql::{GraphQlClient, GraphQlRequest};
use super::super::rest::RestRequest;
use super::super::types::{ReviewComment, ReviewState};

// ── REST primitives ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct RawReview {
    pub(crate) state: String,
    pub(crate) body: String,
    pub(crate) user: RawUser,
}

#[derive(Deserialize)]
pub(crate) struct RawUser {
    pub(crate) login: String,
}

#[derive(Deserialize)]
pub(crate) struct RawReviewComment {
    pub(crate) body: String,
    pub(crate) path: Option<String>,
    pub(crate) line: Option<u64>,
    pub(crate) original_line: Option<u64>,
    pub(crate) user: RawUser,
    pub(crate) in_reply_to_id: Option<u64>,
}

pub(crate) struct GetReviews {
    pub(crate) pr_number: u64,
}

impl RestRequest for GetReviews {
    type Response = Vec<RawReview>;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}/reviews", repo, self.pr_number)
    }
}

pub(crate) struct GetReviewComments {
    pub(crate) pr_number: u64,
}

impl RestRequest for GetReviewComments {
    type Response = Vec<RawReviewComment>;
    fn endpoint(&self, repo: &str) -> String {
        format!("repos/{}/pulls/{}/comments", repo, self.pr_number)
    }
}

// ── GraphQL primitive ─────────────────────────────────────────────────────────

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

struct GetUnresolvedThreads {
    owner: String,
    name: String,
    pr_number: u64,
}

impl GraphQlRequest for GetUnresolvedThreads {
    type Response = GraphQlResponse<RepositoryQuery>;
    fn query(&self) -> String {
        let owner = &self.owner;
        let name = &self.name;
        let pr_number = self.pr_number;
        format!(
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
        )
    }
}

// ── GraphQL helper ────────────────────────────────────────────────────────────

/// Fetch unresolved review thread comments via GraphQL.
///
/// Keeps the GraphQL schema types private to this module.
pub(crate) fn fetch_unresolved_threads(
    graphql: &GraphQlClient,
    repo: &str,
    pr_number: u64,
) -> Result<Vec<ReviewComment>> {
    let (owner, name) = repo
        .split_once('/')
        .ok_or_else(|| GitHubError::Other("invalid repo format".to_string()))?;

    let response = graphql.call(&GetUnresolvedThreads {
        owner: owner.to_string(),
        name: name.to_string(),
        pr_number,
    })?;

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
