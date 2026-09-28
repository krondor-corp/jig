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
