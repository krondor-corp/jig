//! A repo to test against: [`Repo::init`] plus what a commit needs.

use std::path::Path;

use super::Repo;

/// A repo on `main` with one empty "init" commit.
///
/// Sets a local identity and turns signing off, so commits work regardless
/// of the developer's global git config.
pub(crate) fn seeded(dir: &Path) -> git2::Repository {
    let repo = Repo::init(dir).expect("git init");
    let inner = repo.inner();
    {
        let mut config = inner.config().expect("repo config");
        config.set_str("user.email", "test@test.com").unwrap();
        config.set_str("user.name", "Test").unwrap();
        config.set_bool("commit.gpgsign", false).unwrap();
    }
    let tree_oid = inner.index().unwrap().write_tree().unwrap();
    let tree = inner.find_tree(tree_oid).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    inner
        .commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .expect("initial commit");

    git2::Repository::open(dir).expect("reopen")
}
