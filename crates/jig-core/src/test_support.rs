//! Fixtures shared by jig-core's and jig-cli's tests.
//!
//! Compiled for jig-core's own tests, and for other crates only through the
//! `test-support` feature — which jig-cli enables from its
//! `[dev-dependencies]`, so it never reaches a release build.

use std::path::Path;

/// Make `dir` a git repo on `main` with one empty commit ("init").
///
/// Sets a local identity and turns commit signing off, so later commits —
/// through git2 or the `git` CLI — work on any machine regardless of the
/// developer's global git config.
pub fn init_repo(dir: &Path) -> git2::Repository {
    let repo = git2::Repository::init(dir).expect("git init");
    {
        let mut config = repo.config().expect("repo config");
        config.set_str("user.email", "test@test.com").unwrap();
        config.set_str("user.name", "Test").unwrap();
        config.set_bool("commit.gpgsign", false).unwrap();
    }
    {
        let tree_oid = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .expect("initial commit");
    }
    repo.head()
        .unwrap()
        .rename("refs/heads/main", true, "init main")
        .expect("rename branch to main");
    repo.set_head("refs/heads/main").unwrap();
    repo
}

/// Temp directories for one test: a config root (what `XDG_CONFIG_HOME`
/// would be), a runtime root (`XDG_RUNTIME_DIR`), and any number of git
/// repos made by [`init_repo`]. Everything is removed on drop.
///
/// It only hands out directories. Unit tests turn it into jig's path layout
/// (`AppPaths::from(&fixture)` in jig-cli); integration tests pass the roots
/// to the `jig` binary as environment variables.
pub struct Fixture {
    config: tempfile::TempDir,
    runtime: tempfile::TempDir,
    repos: Vec<tempfile::TempDir>,
}

impl Default for Fixture {
    fn default() -> Self {
        Self::new()
    }
}

impl Fixture {
    /// Config and runtime roots, no repos.
    pub fn new() -> Self {
        Self {
            config: tempfile::tempdir().expect("config root"),
            runtime: tempfile::tempdir().expect("runtime root"),
            repos: Vec::new(),
        }
    }

    /// Config and runtime roots plus `n` repos.
    pub fn with_repos(n: usize) -> Self {
        let mut fixture = Self::new();
        for _ in 0..n {
            fixture.add_repo();
        }
        fixture
    }

    /// Add a repo (see [`init_repo`]) and return its path.
    pub fn add_repo(&mut self) -> &Path {
        let dir = tempfile::tempdir().expect("repo dir");
        init_repo(dir.path());
        self.repos.push(dir);
        self.repos.last().unwrap().path()
    }

    /// The `i`th repo, in the order they were added.
    pub fn repo(&self, i: usize) -> &Path {
        self.repos[i].path()
    }

    /// Every repo, in the order they were added.
    pub fn repos(&self) -> impl Iterator<Item = &Path> {
        self.repos.iter().map(|d| d.path())
    }

    /// What `XDG_CONFIG_HOME` would be.
    pub fn config_home(&self) -> &Path {
        self.config.path()
    }

    /// What `XDG_RUNTIME_DIR` would be.
    pub fn runtime_home(&self) -> &Path {
        self.runtime.path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_repo_is_on_main_with_one_commit() {
        let dir = tempfile::tempdir().unwrap();
        let repo = init_repo(dir.path());
        let head = repo.head().unwrap();
        assert_eq!(head.shorthand(), Some("main"));
        assert_eq!(head.peel_to_commit().unwrap().message(), Some("init"));
    }

    #[test]
    fn fixture_hands_out_distinct_repos() {
        let fixture = Fixture::with_repos(3);
        let repos: Vec<_> = fixture.repos().collect();
        assert_eq!(repos.len(), 3);
        assert_ne!(repos[0], repos[1]);
        for repo in repos {
            assert!(git2::Repository::open(repo).is_ok());
        }
        assert_ne!(fixture.config_home(), fixture.runtime_home());
    }
}
