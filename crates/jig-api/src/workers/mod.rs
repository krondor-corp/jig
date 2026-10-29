//! `/api/v0/workers/*`: the control verbs and a worker's setup log.
//!
//! The verbs are the CLI's (`jig kill`, `nudge`, `prune`, `retry-setup`,
//! `model`), and run through the same daemon executor. Each needs a
//! [`crate::Client::with_token`] client, and answers [`CommandOutcome`] or
//! refuses with the daemon's reason ([`crate::ApiError::Refused`]).

pub mod kill;
pub mod model;
pub mod nudge;
pub mod prune;
pub mod retry_setup;
pub mod setup_log;

pub use kill::KillRequest;
pub use model::ModelRequest;
pub use nudge::NudgeRequest;
pub use prune::PruneRequest;
pub use retry_setup::RetrySetupRequest;
pub use setup_log::{SetupLog, SetupLogRequest};

use serde::{Deserialize, Serialize};

/// A verb the daemon ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandOutcome {
    pub message: String,
}
