//! The daemon: its socket, its single-instance claim, and its actors.

#[path = "../common/mod.rs"]
mod common;

mod actors;
mod ipc;
mod pidfile;
