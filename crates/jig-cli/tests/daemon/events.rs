//! The daemon's lifecycle log.

use std::io::Write;

use crate::common::Sandbox;
use jig_cli::context::log::Logs;
use jig_cli::daemon::events;

/// A line in a shape this build doesn't write — seen on a real machine —
/// must not cost the rest of the history, or `log_startup` resets the file.
#[test]
fn a_foreign_line_mid_log_still_reduces() {
    let sandbox = Sandbox::new();
    let paths = sandbox.paths();
    let log = Logs::new(&paths).daemon_events();

    log.append(&events::started(None)).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&log.path)
        .unwrap();
    writeln!(file, r#"{{"event":"Started","ts":1756700000,"pid":1}}"#).unwrap();
    drop(file);
    log.append(&events::stopped("signal")).unwrap();

    let state = log.reduce().unwrap();
    assert_eq!(state.pid, Some(std::process::id()));
    assert_eq!(state.stop_reason.as_deref(), Some("signal"));
    assert!(!state.previous_run_crashed());
    assert_eq!(log.read_all().unwrap().len(), 2);
}
