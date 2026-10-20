//! Driving the `jig` binary end to end.

#[path = "../common/mod.rs"]
mod common;

mod attach;
mod basics;
mod commit;
mod config;
mod exit;
mod init;
mod multi_repo;
mod notify;
mod on_create;
mod startup;
mod worktree;
