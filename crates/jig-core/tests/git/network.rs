//! Remote operations go through the `git` binary.

use jig_core::exec::Timeout;

/// Fetching uses whatever credentials `git` itself uses — the user's
/// `~/.ssh/config` with its per-host `IdentityFile`, the agent, the keychain,
/// credential helpers. jig has no opinion about them, which is the point:
/// libgit2 reads none of that, so jig had to guess what credential to hand
/// it, and guessing key paths is wrong on any machine whose key is not
/// `~/.ssh/id_*` — a service account's usually is not.
///
/// Ignored: needs the network. Run with
/// `cargo test -p jig-core --test git -- --ignored fetch`.
#[test]
#[ignore]
fn fetching_uses_the_same_credentials_git_does() {
    let tmp = tempfile::TempDir::new().unwrap();
    let repo = jig_core::git::Repo::init(tmp.path()).unwrap();
    repo.inner()
        .remote("origin", "git@github.com:krondor-corp/jig.git")
        .unwrap();

    repo.fetch(
        "origin",
        &["refs/heads/main:refs/remotes/origin/main"],
        Timeout::secs(120),
    )
    .expect("fetch should work wherever `git fetch` does");

    assert!(
        repo.inner()
            .find_reference("refs/remotes/origin/main")
            .is_ok(),
        "the fetched ref should be there"
    );
}
