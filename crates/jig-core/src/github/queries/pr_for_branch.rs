use super::super::client::GitHubClient;
use super::super::error::Result;
use super::super::types::{PrInfo, PrState};

impl GitHubClient {
    /// Get PR info for a branch (any state: open, closed, or merged).
    pub fn get_pr_for_branch(&self, branch: &str) -> Result<Option<PrInfo>> {
        let encoded_branch = urlencoding::encode(branch);
        let output = self.gh_api(&format!(
            "repos/{}/pulls?head={}:{}&state=all",
            self.repo,
            self.repo.split('/').next().unwrap_or(""),
            encoded_branch
        ))?;

        let prs: Vec<serde_json::Value> = serde_json::from_str(&output)?;
        let Some(pr) = prs.first() else {
            return Ok(None);
        };

        Ok(Some(parse_pr_json(pr, branch)))
    }
}

pub(crate) fn parse_pr_json(pr: &serde_json::Value, branch: &str) -> PrInfo {
    let merged = pr["merged_at"].is_string();
    let state_str = pr["state"].as_str().unwrap_or("open");
    let state = if merged {
        PrState::Merged
    } else if state_str == "closed" {
        PrState::Closed
    } else {
        PrState::Open
    };

    PrInfo {
        number: pr["number"].as_u64().unwrap_or(0),
        title: pr["title"].as_str().unwrap_or("").to_string(),
        state,
        mergeable: pr["mergeable_state"].as_str().map(|s| s.to_uppercase()),
        head_branch: branch.to_string(),
        url: pr["html_url"].as_str().unwrap_or("").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_open_pr() {
        let json = serde_json::json!({
            "number": 17, "title": "feat: add widget", "state": "open",
            "merged_at": null, "mergeable_state": "clean",
            "html_url": "https://github.com/org/repo/pull/17",
        });
        let info = parse_pr_json(&json, "feature/test");
        assert_eq!(info.state, PrState::Open);
        assert_eq!(info.number, 17);
    }

    #[test]
    fn parses_closed_pr() {
        let json = serde_json::json!({
            "number": 17, "title": "feat: add widget", "state": "closed",
            "merged_at": null, "mergeable_state": null,
            "html_url": "https://github.com/org/repo/pull/17",
        });
        let info = parse_pr_json(&json, "feature/test");
        assert_eq!(info.state, PrState::Closed);
    }

    #[test]
    fn parses_merged_pr() {
        let json = serde_json::json!({
            "number": 18, "title": "feat: add widget", "state": "closed",
            "merged_at": "2026-05-10T22:33:59Z", "mergeable_state": null,
            "html_url": "https://github.com/org/repo/pull/18",
        });
        let info = parse_pr_json(&json, "feature/test");
        assert_eq!(info.state, PrState::Merged);
    }

    /// GitHub returns state=closed for both closed and merged PRs.
    /// merged_at being non-null is what distinguishes them.
    #[test]
    fn merged_at_takes_precedence_over_state_closed() {
        let json = serde_json::json!({
            "number": 18, "state": "closed", "title": "x",
            "merged_at": "2026-05-10T22:33:59Z",
            "mergeable_state": null, "html_url": "https://github.com/org/repo/pull/18",
        });
        assert_eq!(json["state"].as_str().unwrap(), "closed");
        let info = parse_pr_json(&json, "feature/test");
        assert_eq!(info.state, PrState::Merged);
    }

    /// Integration test: hits real GitHub API via `gh` CLI.
    /// Uses known closed PR #17 and merged PR #18 on krondor-corp/jig.
    /// Run with: cargo test -p jig-core -- --ignored
    #[test]
    #[ignore]
    fn gh_api_returns_closed_pr_for_branch() {
        use super::super::super::client::GitHubClient;

        let gh = GitHubClient::new("krondor-corp/jig");
        let result = gh
            .get_pr_for_branch("feature/kro-141-test-cleanup-verification-add-comment-to-readme")
            .expect("gh api call failed");

        let pr = result.expect("expected PR, got None — the state=all fix is broken");
        assert_eq!(pr.number, 17);
        assert_eq!(
            pr.state,
            PrState::Closed,
            "PR #17 was closed without merging, expected Closed but got {:?}",
            pr.state
        );
    }

    #[test]
    #[ignore]
    fn gh_api_returns_merged_pr_for_branch() {
        use super::super::super::client::GitHubClient;

        let gh = GitHubClient::new("krondor-corp/jig");
        let result = gh
            .get_pr_for_branch(
                "feature/kro-142-test-merge-cleanup-verification-add-html-comment-to-readme",
            )
            .expect("gh api call failed");

        let pr = result.expect("expected PR, got None — the state=all fix is broken");
        assert_eq!(pr.number, 18);
        assert_eq!(
            pr.state,
            PrState::Merged,
            "PR #18 was merged, expected Merged but got {:?}",
            pr.state
        );
    }
}
