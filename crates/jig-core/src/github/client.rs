//! GitHub client wrapping `gh` CLI.

use std::path::Path;
use std::process::{Command, Stdio};

use super::error::{GitHubError, Result};
use super::graphql::GraphQlClient;
use super::rest::RestClient;

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
