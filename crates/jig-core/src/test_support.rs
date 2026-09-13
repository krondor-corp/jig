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
