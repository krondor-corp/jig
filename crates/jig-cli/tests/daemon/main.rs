//! The daemon: its socket, its single-instance claim, and its actors.

#[path = "../common/mod.rs"]
mod common;

mod actors;
mod commands;
mod events;
mod ipc;
mod pidfile;
mod web;
