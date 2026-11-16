//! The private network a fleet of jig machines talks over.
//!
//! jig needs four things from a network: somewhere to listen, a list of peers,
//! who is calling, and whether the network is up. [`Net`] asks for exactly
//! those, at the level of authenticated streams, so address-based networks
//! (`static`) and identity-aware ones (`tailscale`) fit behind one trait.
//! Chosen per machine like the mux: `[fleet] net = "tailscale" | "static"`.
//!
//! Nothing in this module binds a socket until [`Net::accept`] is called —
//! enabling a listener is the daemon's decision, not a side effect of
//! building a backend. See `docs/fleet-spec.md`.

mod listener;
pub mod static_net;
pub mod tailscale;

pub use static_net::{StaticConfig, StaticNet, StaticPeer};
pub use tailscale::TailscaleNet;

use std::fmt;
use std::io::{Read, Write};
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::exec::ExecError;

/// The port a fleet listener uses when none is configured.
pub const DEFAULT_PORT: u16 = 7473;

/// How long [`Net::dial`] waits for a peer to answer the connection.
pub const DIAL_TIMEOUT: Duration = Duration::from_secs(10);

/// A byte stream to or from a peer.
pub trait Stream: Read + Write + Send {}

impl<T: Read + Write + Send> Stream for T {}

/// Who a machine is on the network.
///
/// Opaque outside its backend: a tailscale stable node id, or a `static`
/// peer's address. Stable for as long as the machine stays enrolled.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<IpAddr> for NodeId {
    fn from(ip: IpAddr) -> Self {
        Self(ip.to_string())
    }
}

/// A machine on the network: what people call it and who it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub id: NodeId,
}

/// The pluggable private network.
///
/// Every connection [`Net::accept`] hands back is from a trusted peer —
/// tagged `tag:jig` on tailscale, allowlisted on `static`. Anyone else is
/// dropped inside `accept`, so callers never see them.
pub trait Net: Send + Sync {
    /// Which backend this is.
    fn kind(&self) -> NetKind;
    /// This machine: its name and node id on the network.
    fn local(&self) -> Result<Node, NetError>;
    /// The other jig machines this one can see.
    fn peers(&self) -> Result<Vec<Node>, NetError>;
    /// The next incoming connection from a trusted peer, and who it is from.
    /// Binds the listener on first call.
    fn accept(&self) -> Result<(Box<dyn Stream>, NodeId), NetError>;
    /// Open a connection to a peer.
    fn dial(&self, peer: &NodeId) -> Result<Box<dyn Stream>, NetError>;
    /// Installed? Connected? This machine enrolled? For `jig health`.
    fn health(&self) -> NetHealth;
}

/// Which network backend a machine's fleet uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetKind {
    /// Tailscale (or Headscale): discovery by `tag:jig`, identity by `whois`.
    Tailscale,
    /// Any private IP network: listen address, peer list, allowlist.
    Static,
}

impl fmt::Display for NetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            NetKind::Tailscale => "tailscale",
            NetKind::Static => "static",
        })
    }
}

impl FromStr for NetKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tailscale" => Ok(NetKind::Tailscale),
            "static" => Ok(NetKind::Static),
            other => Err(format!(
                "unknown net backend '{other}' (expected tailscale or static)"
            )),
        }
    }
}

/// One line of a health report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    Ok(String),
    Warn(String),
    Fail(String),
}

impl Check {
    pub fn is_fail(&self) -> bool {
        matches!(self, Check::Fail(_))
    }

    pub fn detail(&self) -> &str {
        match self {
            Check::Ok(d) | Check::Warn(d) | Check::Fail(d) => d,
        }
    }
}

/// What `jig health` reports about the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetHealth {
    /// Is the backend available on this machine?
    pub installed: Check,
    /// Is this machine on the network?
    pub connected: Check,
    /// Will peers trust this machine, and does it know whom to trust?
    /// Tagged `tag:jig` on tailscale; peers and allowlist on `static`.
    pub enrolled: Check,
}

impl NetHealth {
    pub fn ok(&self) -> bool {
        ![&self.installed, &self.connected, &self.enrolled]
            .iter()
            .any(|c| c.is_fail())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error(transparent)]
    Exec(#[from] ExecError),
    #[error("{command} failed: {detail}")]
    Command { command: String, detail: String },
    #[error("could not parse {command} output: {source}")]
    Parse {
        command: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("not connected: {0}")]
    NotConnected(String),
    #[error("no jig peer {0}")]
    UnknownPeer(NodeId),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn net_kind_reads_from_config() {
        #[derive(Deserialize)]
        struct Cfg {
            net: NetKind,
        }
        let cfg: Cfg = toml::from_str(r#"net = "tailscale""#).unwrap();
        assert_eq!(cfg.net, NetKind::Tailscale);
        let cfg: Cfg = toml::from_str(r#"net = "static""#).unwrap();
        assert_eq!(cfg.net, NetKind::Static);
        assert!(toml::from_str::<Cfg>(r#"net = "iroh""#).is_err());
    }

    #[test]
    fn net_kind_round_trips_through_its_name() {
        for kind in [NetKind::Tailscale, NetKind::Static] {
            assert_eq!(kind.to_string().parse::<NetKind>().unwrap(), kind);
        }
        assert!("zerotier".parse::<NetKind>().is_err());
    }

    #[test]
    fn health_fails_only_on_a_failed_check() {
        let health = NetHealth {
            installed: Check::Ok("built in".into()),
            connected: Check::Ok("up".into()),
            enrolled: Check::Warn("no peers".into()),
        };
        assert!(health.ok());
        let health = NetHealth {
            connected: Check::Fail("down".into()),
            ..health
        };
        assert!(!health.ok());
    }
}
