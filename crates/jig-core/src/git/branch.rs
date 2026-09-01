//! Branch name newtype.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Branch(String);

impl Branch {
    pub fn new(s: impl Into<String>) -> Self {
        let name = s.into();
        let refname = format!("refs/heads/{name}");
        assert!(
            git2::Reference::is_valid_name(&refname),
            "invalid git branch name: {name:?}"
        );
        Self(name)
    }

    /// Returns the segment before the first `/`, or `None` if the branch name
    /// has no slash. For `origin/main` this is `"origin"`; for `main` it is
    /// `None`.
    pub fn remote_prefix(&self) -> Option<&str> {
        self.0.split_once('/').map(|(prefix, _)| prefix)
    }

    /// Returns the branch name with any `origin/` remote prefix stripped.
    /// `origin/main` and `main` both return `"main"`.
    pub fn local(&self) -> &str {
        self.0.strip_prefix("origin/").unwrap_or(&self.0)
    }

    /// Returns the `origin/<local>` remote-tracking ref name, normalizing
    /// away any existing `origin/` prefix first.
    pub fn remote_ref(&self) -> String {
        format!("origin/{}", self.local())
    }
}

impl std::fmt::Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for Branch {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl From<String> for Branch {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for Branch {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<Branch> for String {
    fn from(b: Branch) -> Self {
        b.0
    }
}

impl PartialEq<str> for Branch {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for Branch {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for Branch {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_valid() {
        Branch::new("main");
        Branch::new("feature/foo");
        Branch::new("origin/main");
        Branch::new("feature/aut-4969-spawn-agent-thread-is-broken");
    }

    #[test]
    fn remote_prefix_with_slash() {
        assert_eq!(Branch::new("origin/main").remote_prefix(), Some("origin"));
        assert_eq!(
            Branch::new("upstream/develop").remote_prefix(),
            Some("upstream")
        );
    }

    #[test]
    fn remote_prefix_without_slash() {
        assert_eq!(Branch::new("main").remote_prefix(), None);
        assert_eq!(Branch::new("develop").remote_prefix(), None);
    }

    #[test]
    fn remote_prefix_local_branch_with_slash() {
        assert_eq!(Branch::new("feature/foo").remote_prefix(), Some("feature"));
    }

    #[test]
    #[should_panic(expected = "invalid git branch name")]
    fn new_rejects_empty() {
        Branch::new("");
    }

    #[test]
    #[should_panic(expected = "invalid git branch name")]
    fn new_rejects_double_dots() {
        Branch::new("a..b");
    }

    #[test]
    #[should_panic(expected = "invalid git branch name")]
    fn new_rejects_space() {
        Branch::new("a b");
    }

    #[test]
    #[should_panic(expected = "invalid git branch name")]
    fn new_rejects_dot_lock() {
        Branch::new("branch.lock");
    }

    #[test]
    fn local_strips_origin_prefix() {
        assert_eq!(Branch::new("origin/main").local(), "main");
        assert_eq!(Branch::new("main").local(), "main");
        assert_eq!(Branch::new("origin/feature/foo").local(), "feature/foo");
    }

    #[test]
    fn remote_ref_normalizes_prefix() {
        assert_eq!(Branch::new("main").remote_ref(), "origin/main");
        assert_eq!(Branch::new("origin/main").remote_ref(), "origin/main");
    }
}
