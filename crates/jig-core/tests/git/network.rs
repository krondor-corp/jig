//! libgit2's network timeouts.

use std::time::Duration;

use jig_core::git::set_network_timeouts;

/// libgit2 defaults to waiting forever, which in the daemon means a push to a
/// server that accepts and then goes quiet stops auto-spawn permanently.
///
/// These are process-wide C globals, so this lives alone in its own test
/// binary module and asserts the value jig actually ships — running it twice,
/// or alongside anything else, sets the same numbers.
#[test]
fn jig_bounds_how_long_libgit2_waits_on_a_remote() {
    // SAFETY: the getters read the same C globals the setter writes. Nothing
    // else in this binary touches them.
    unsafe {
        let before = git2::opts::get_server_timeout_in_milliseconds().unwrap();
        assert_eq!(before, 0, "libgit2 should start with no timeout at all");

        set_network_timeouts();

        let connect = git2::opts::get_server_connect_timeout_in_milliseconds().unwrap();
        let idle = git2::opts::get_server_timeout_in_milliseconds().unwrap();

        assert_eq!(
            Duration::from_millis(connect as u64),
            Duration::from_secs(10)
        );
        assert_eq!(Duration::from_millis(idle as u64), Duration::from_secs(60));
    }
}
