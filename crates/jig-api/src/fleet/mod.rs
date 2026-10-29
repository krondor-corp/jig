//! `/api/v0/fleet`: the work view for every machine on the fleet network.
//!
//! The daemon serving the page answers for itself and polls each peer `Net`
//! lists, so any machine's page shows the whole fleet. A worker's verbs go
//! back to the daemon that served the page, carrying the owning machine's
//! [`Machine::node`]; that daemon forwards them to the owner.

pub mod get;

pub use get::FleetRequest;

use serde::{Deserialize, Serialize};

use crate::Status;

/// Every machine, this one first.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Fleet {
    pub machines: Vec<Machine>,
    /// Why the peers could not be listed (the network is down, say), when
    /// they could not. This machine is still in [`Self::machines`].
    pub peers_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Machine {
    /// What the fleet calls it.
    pub name: String,
    /// Its node id on the network: the `machine` a verb request names to
    /// reach it. `None` for the machine serving the page.
    pub node: Option<String>,
    /// Its frame, when it answered.
    pub status: Option<Status>,
    /// Why it did not answer, when it did not.
    pub error: Option<String>,
}

impl Machine {
    /// Whether this is the machine serving the page.
    pub fn is_local(&self) -> bool {
        self.node.is_none()
    }
}
