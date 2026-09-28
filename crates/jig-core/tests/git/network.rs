//! libgit2's network timeouts.

use jig_core::exec::Timeout;
use jig_core::git::set_network_timeouts;

/// libgit2 waits forever by default, which in the daemon means a push to a
/// server that accepts and then goes quiet stops auto-spawn permanently.
///
/// These are process-wide C globals, so this is the only test in this binary
/// that touches them.
#[test]
fn jig_bounds_how_long_libgit2_waits_on_a_remote() {
    // SAFETY: the getters read the same C globals the setter writes, and
    // nothing else in this binary touches them.
    unsafe {
        assert_eq!(
            git2::opts::get_server_timeout_in_milliseconds().unwrap(),
            0,
            "libgit2 should start with no timeout at all — that is the bug"
        );

        set_network_timeouts(Timeout::secs(10), Timeout::secs(60));

        assert_eq!(
            git2::opts::get_server_connect_timeout_in_milliseconds().unwrap(),
            10_000
        );
        assert_eq!(
            git2::opts::get_server_timeout_in_milliseconds().unwrap(),
            60_000
        );

        // `"none"` in config has to restore libgit2's own behaviour, not a
        // zero-length deadline that kills every fetch instantly.
        set_network_timeouts(Timeout::Unlimited, Timeout::Unlimited);
        assert_eq!(git2::opts::get_server_timeout_in_milliseconds().unwrap(), 0);
    }
}

/// Fetch over SSH with no agent available — a daemon installed as a service.
///
/// Ignored: needs the network and a **passphrase-less** key in `~/.ssh` that
/// GitHub accepts — i.e. a headless box, which is the case this exists for. On
/// a dev machine whose key is passphrase-protected (macOS `UseKeychain`) it
/// fails by design: without an agent there is nowhere to get the passphrase.
///
/// Run with `cargo test -p jig-core --test git -- --ignored ssh`.
#[test]
#[ignore]
fn fetches_over_ssh_without_an_agent() {
    let tmp = tempfile::TempDir::new().unwrap();
    let repo = jig_core::git::Repo::init(tmp.path()).unwrap();
    repo.inner()
        .remote("origin", "git@github.com:krondor-corp/jig.git")
        .unwrap();

    // What a systemd service sees: no agent, only key files on disk.
    unsafe { std::env::remove_var("SSH_AUTH_SOCK") };

    repo.fetch("origin", &["refs/heads/main:refs/remotes/origin/main"])
        .expect("fetch should fall back to ~/.ssh keys when there is no agent");
}
