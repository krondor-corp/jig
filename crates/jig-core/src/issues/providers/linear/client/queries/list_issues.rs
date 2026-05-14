use serde::Deserialize;

use super::super::error::Result;
use super::super::request::LinearRequest;
use super::super::types::{NodeList, RawIssue};

const QUERY: &str = r#"
query ListIssues($filter: IssueFilter, $first: Int) {
  issues(filter: $filter, first: $first) {
    nodes {
      id
      identifier
      title
      description
      url
      priority
      branchName
      state { type }
      project { name }
      team { name }
      parent { identifier title description branchName state { type } }
      children { nodes { identifier branchName state { type } } }
      labels { nodes { name } }
      inverseRelations {
        nodes {
          id
          type
          issue { identifier }
          relatedIssue { identifier }
        }
      }
    }
  }
}
"#;

/// Fetch issues from Linear.
///
/// This is the **single** underlying GraphQL operation for both
/// list-many and get-one workflows. If you need a single issue,
/// pass `first: 1` and call `.into_iter().next()` on the result.
///
/// **Do not** add a separate `GetIssue` query — all issue field
/// selection lives here so the field set has one canonical source
/// of truth. If you need genuinely different field selection
/// (e.g., a lightweight fetch that omits `inverseRelations`),
/// define a distinct query with a distinct output type and
/// document the intentional divergence — but only when the
/// difference is real, not for output-type ergonomics.
pub struct ListIssues {
    pub filter: serde_json::Value,
    pub first: i32,
}

#[derive(Debug, Deserialize)]
pub struct ListIssuesResponse {
    pub issues: NodeList<RawIssue>,
}

impl LinearRequest for ListIssues {
    type Response = ListIssuesResponse;
    type Output = Vec<RawIssue>;

    const QUERY: &'static str = QUERY;

    fn variables(&self) -> serde_json::Value {
        serde_json::json!({
            "filter": self.filter,
            "first": self.first,
        })
    }

    fn extract(response: Self::Response) -> Result<Self::Output> {
        Ok(response.issues.nodes)
    }
}
