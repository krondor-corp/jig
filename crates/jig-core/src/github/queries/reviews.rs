use serde::Deserialize;

use super::super::graphql::GraphQlRequest;
use super::super::rest::RestRequest;

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

// ── GraphQL primitives ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct GraphQlResponse<T> {
    pub(crate) data: T,
}

#[derive(Deserialize)]
pub(crate) struct RepositoryQuery {
    pub(crate) repository: RepositoryData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepositoryData {
    pub(crate) pull_request: PullRequestData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PullRequestData {
    pub(crate) review_threads: ReviewThreadsConnection,
}

#[derive(Deserialize)]
pub(crate) struct ReviewThreadsConnection {
    pub(crate) nodes: Vec<RawReviewThread>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawReviewThread {
    pub(crate) is_resolved: bool,
    pub(crate) comments: CommentsConnection,
}

#[derive(Deserialize)]
pub(crate) struct CommentsConnection {
    pub(crate) nodes: Vec<RawThreadComment>,
}

#[derive(Deserialize)]
pub(crate) struct RawThreadComment {
    pub(crate) body: String,
    pub(crate) path: Option<String>,
    pub(crate) line: Option<u64>,
    pub(crate) author: RawAuthor,
}

#[derive(Deserialize)]
pub(crate) struct RawAuthor {
    pub(crate) login: String,
}

pub(crate) struct GetUnresolvedThreads {
    pub(crate) owner: String,
    pub(crate) name: String,
    pub(crate) pr_number: u64,
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
