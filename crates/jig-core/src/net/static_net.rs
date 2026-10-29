//! `static` backend for the [`Net`] trait — any private IP network.
//!
//! The operator brings the network (WireGuard, ZeroTier, Nebula, NetBird, a
//! LAN) and tells jig where to listen, who the peers are, and which source
//! addresses to trust. Identity is the caller's source address: sound on
//! WireGuard-family overlays, where an address can only be used by the holder
//! of its key; on a plain LAN only as good as the LAN.
//!
//! ```toml
//! [fleet]
//! net = "static"
//!
//! [fleet.static]
//! listen = "10.0.0.2:7473"
//! peers = [{ name = "laptop", addr = "10.0.0.3:7473" }]
//! allow = ["10.0.0.4"]   # trusted callers besides the peers
//! ```

use std::net::{IpAddr, SocketAddr, UdpSocket};

use serde::{Deserialize, Serialize};

use super::listener::{self, LazyListener};
use super::{Check, Net, NetError, NetHealth, NetKind, Node, NodeId, Stream};

/// `[fleet.static]` in the global config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticConfig {
    /// Where this machine's listener binds. Must be an address on the private
    /// network — never `0.0.0.0` on a machine with a public interface.
    pub listen: SocketAddr,
    /// The other jig machines.
    #[serde(default)]
    pub peers: Vec<StaticPeer>,
    /// Source addresses trusted to call in, besides the peers' own.
    #[serde(default)]
    pub allow: Vec<IpAddr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticPeer {
    pub name: String,
    pub addr: SocketAddr,
}

/// A peer's node id is its address without the port: that is what a
/// connection from it carries.
fn node_id(addr: &SocketAddr) -> NodeId {
    addr.ip().into()
}

/// The `static` backend.
pub struct StaticNet {
    name: String,
    config: StaticConfig,
    listener: LazyListener,
}

impl StaticNet {
    /// `name` is what this machine calls itself — the fleet machine name.
    pub fn new(name: impl Into<String>, config: StaticConfig) -> Self {
        Self {
            name: name.into(),
            config,
            listener: LazyListener::default(),
        }
    }

    /// Bind the listener if it is not yet bound, and say where it is.
    /// Useful when `listen` asks for port 0.
    pub fn listen(&self) -> Result<SocketAddr, NetError> {
        Ok(self.listener.get(|| Ok(self.config.listen))?.1)
    }

    /// Whether a connection from `ip` is from someone this machine trusts.
    pub fn trusts(&self, ip: IpAddr) -> bool {
        self.config.allow.contains(&ip) || self.config.peers.iter().any(|p| p.addr.ip() == ip)
    }
}

impl Net for StaticNet {
    fn kind(&self) -> NetKind {
        NetKind::Static
    }

    fn local(&self) -> Result<Node, NetError> {
        Ok(Node {
            name: self.name.clone(),
            id: node_id(&self.config.listen),
        })
    }

    fn peers(&self) -> Result<Vec<Node>, NetError> {
        Ok(self
            .config
            .peers
            .iter()
            .map(|p| Node {
                name: p.name.clone(),
                id: node_id(&p.addr),
            })
            .collect())
    }

    fn accept(&self) -> Result<(Box<dyn Stream>, NodeId), NetError> {
        self.listener.accept(
            || Ok(self.config.listen),
            |ip| Ok(self.trusts(ip).then(|| ip.into())),
        )
    }

    fn dial(&self, peer: &NodeId) -> Result<Box<dyn Stream>, NetError> {
        let addr = self
            .config
            .peers
            .iter()
            .find(|p| &node_id(&p.addr) == peer)
            .ok_or_else(|| NetError::UnknownPeer(peer.clone()))?
            .addr;
        listener::connect(addr)
    }

    fn health(&self) -> NetHealth {
        let ip = self.config.listen.ip();
        let connected = if ip.is_unspecified() {
            Check::Warn(format!(
                "listen = {} binds every interface — use the private address",
                self.config.listen
            ))
        } else if is_local(ip) {
            Check::Ok(format!("{ip} is on this machine"))
        } else {
            Check::Fail(format!("{ip} is not an address of this machine"))
        };

        let peers = self.config.peers.len();
        let allowed = self.config.allow.len();
        let enrolled = if peers + allowed == 0 {
            Check::Warn("no peers or allowed addresses in [fleet.static]".into())
        } else {
            Check::Ok(format!(
                "{peers} peer{}, {allowed} more allowed",
                if peers == 1 { "" } else { "s" }
            ))
        };

        NetHealth {
            installed: Check::Ok("built in".into()),
            connected,
            enrolled,
        }
    }
}

/// Whether `ip` is assigned to an interface here. Binding a UDP socket to it
/// succeeds only if so; the socket is closed at once and never receives.
fn is_local(ip: IpAddr) -> bool {
    UdpSocket::bind((ip, 0)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> StaticConfig {
        toml::from_str(
            r#"
listen = "10.0.0.2:7473"
peers = [{ name = "laptop", addr = "10.0.0.3:7473" }]
allow = ["10.0.0.9"]
"#,
        )
        .unwrap()
    }

    #[test]
    fn config_reads_from_toml() {
        let config = config();
        assert_eq!(config.listen, "10.0.0.2:7473".parse().unwrap());
        assert_eq!(config.peers[0].name, "laptop");
        assert_eq!(config.allow, vec!["10.0.0.9".parse::<IpAddr>().unwrap()]);
    }

    #[test]
    fn peers_and_allowlisted_addresses_are_trusted() {
        let net = StaticNet::new("devbox", config());
        assert!(net.trusts("10.0.0.3".parse().unwrap()));
        assert!(net.trusts("10.0.0.9".parse().unwrap()));
        assert!(!net.trusts("10.0.0.5".parse().unwrap()));
    }

    #[test]
    fn peers_are_named_and_addressed_by_ip() {
        let net = StaticNet::new("devbox", config());
        assert_eq!(
            net.peers().unwrap(),
            vec![Node {
                name: "laptop".into(),
                id: NodeId::new("10.0.0.3")
            }]
        );
        assert_eq!(net.local().unwrap().name, "devbox");
        assert_eq!(net.local().unwrap().id, NodeId::new("10.0.0.2"));
    }

    #[test]
    fn dialing_a_stranger_is_an_error() {
        let net = StaticNet::new("devbox", config());
        let err = net.dial(&NodeId::new("10.0.0.5")).err().unwrap();
        assert!(matches!(err, NetError::UnknownPeer(_)), "{err:?}");
    }

    #[test]
    fn health_flags_an_address_that_is_not_ours() {
        // TEST-NET-1: never assigned to a real interface.
        let mut config = config();
        config.listen = "192.0.2.1:7473".parse().unwrap();
        let health = StaticNet::new("devbox", config).health();
        assert!(health.connected.is_fail(), "{health:?}");
        assert!(!health.ok());
    }

    #[test]
    fn health_passes_on_loopback() {
        let mut config = config();
        config.listen = "127.0.0.1:7473".parse().unwrap();
        let health = StaticNet::new("devbox", config).health();
        assert!(health.ok(), "{health:?}");
    }

    #[test]
    fn health_warns_about_an_empty_trust_list() {
        let config = StaticConfig {
            listen: "127.0.0.1:7473".parse().unwrap(),
            peers: vec![],
            allow: vec![],
        };
        let health = StaticNet::new("devbox", config).health();
        assert!(matches!(health.enrolled, Check::Warn(_)), "{health:?}");
    }
}
