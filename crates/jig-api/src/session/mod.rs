//! `/api/v0/session`: what the app needs before it can send commands.

pub mod get;

pub use get::{Session, SessionRequest};
