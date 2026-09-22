//! jig — git worktree manager for parallel Claude Code sessions.
//!
//! The binary in `main.rs` is a thin shell over this library, so tests can
//! drive jig's internals directly instead of only through the CLI.

pub mod cli;
pub mod context;
pub mod daemon;
pub mod hooks;
pub mod notify;
pub mod prompts;
pub mod terminal;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod worker;
