//! `tailscale` backend for the [`Net`] trait.
//!
//! Shells out to the `tailscale` CLI through [`Exec`] — no new dependencies,
//! no async runtime. Works unchanged against Headscale, which is only a
//! different control server for the same client.
//!
//! - **peers** — `tailscale status --json`, filtered to nodes tagged
//!   [`JIG_TAG`]. Tags are set by the tailnet admin (or a pre-tagged auth
//!   key), so a node cannot make itself a jig peer.
//! - **identity** — `tailscale whois --json <ip>` on the caller's address.
//! - **listen** — `tailscale ip -4`, so the listener is only ever on the
//!   tailnet interface.
//!
//! Responses are parsed into typed structs; an unexpected shape is an error,
//! not a silent default.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::process::Command;

use serde::Deserialize;

use super::listener::{self, LazyListener};
use super::{Check, Net, NetError, NetHealth, NetKind, Node, NodeId, Stream};
use crate::exec::{Exec, ExecError, Timeout};

/// The ACL tag that makes a tailnet node a jig peer.
pub const JIG_TAG: &str = "tag:jig";

const JOIN_HINT: &str = "run `jig fleet join <authkey>`";

// ── CLI response shapes ─────────────────────────────────────────────

/// `tailscale status --json`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Status {
    backend_state: String,
    #[serde(rename = "Self")]
    me: Option<PeerStatus>,
    #[serde(default)]
    peer: Option<HashMap<String, PeerStatus>>,
    current_tailnet: Option<Tailnet>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PeerStatus {
    #[serde(rename = "ID")]
    id: String,
    host_name: String,
    #[serde(rename = "DNSName", default)]
    dns_name: String,
    #[serde(rename = "TailscaleIPs", default)]
    tailscale_ips: Option<Vec<IpAddr>>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Tailnet {
    name: String,
}

/// `tailscale whois --json <ip>`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WhoIs {
    node: WhoIsNode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WhoIsNode {
    #[serde(rename = "StableID")]
    stable_id: String,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

fn has_jig_tag(tags: &Option<Vec<String>>) -> bool {
    tags.iter().flatten().any(|t| t == JIG_TAG)
}

impl PeerStatus {
    fn is_jig(&self) -> bool {
        has_jig_tag(&self.tags)
    }

    /// The MagicDNS name's first label — unique on the tailnet, unlike the
    /// OS hostname — else the hostname.
    fn name(&self) -> String {
        self.dns_name
            .split('.')
            .next()
            .filter(|n| !n.is_empty())
            .unwrap_or(&self.host_name)
            .to_string()
    }

    fn node(&self) -> Node {
        Node {
            name: self.name(),
            id: NodeId::new(&self.id),
        }
    }

    /// Where to reach it: IPv4 first, which every client routes.
    fn addr(&self) -> Option<IpAddr> {
        let ips = self.tailscale_ips.as_deref().unwrap_or_default();
        ips.iter()
            .find(|ip| ip.is_ipv4())
            .or_else(|| ips.first())
            .copied()
    }
}

impl Status {
    fn parse(json: &str) -> Result<Self, NetError> {
        serde_json::from_str(json).map_err(|source| NetError::Parse {
            command: "tailscale status".into(),
            source,
        })
    }

    fn running(&self) -> bool {
        self.backend_state == "Running"
    }

    fn me(&self) -> Result<&PeerStatus, NetError> {
        self.me
            .as_ref()
            .filter(|_| self.running())
            .ok_or_else(|| NetError::NotConnected(format!("tailscale is {}", self.backend_state)))
    }

    /// Tagged peers, sorted by name.
    fn jig_peers(&self) -> Vec<&PeerStatus> {
        let mut peers: Vec<_> = self
            .peer
            .iter()
            .flat_map(|p| p.values())
            .filter(|p| p.is_jig())
            .collect();
        peers.sort_by_key(|p| p.name());
        peers
    }

    /// `connected` and `enrolled`, for [`NetHealth`].
    fn assess(&self) -> (Check, Check) {
        let me = match self.me() {
            Ok(me) => me,
            Err(_) => {
                let hint = match self.backend_state.as_str() {
                    "Stopped" => "run `tailscale up`",
                    _ => JOIN_HINT,
                };
                return (
                    Check::Fail(format!("{} — {hint}", self.backend_state)),
                    Check::Fail("not on a tailnet".into()),
                );
            }
        };

        let ip = me.addr().map(|ip| ip.to_string()).unwrap_or_default();
        let connected = match &self.current_tailnet {
            Some(t) => Check::Ok(format!("{ip} on {}", t.name)),
            None => Check::Ok(ip),
        };

        let peers = self.jig_peers().len();
        let enrolled = if me.is_jig() {
            Check::Ok(format!(
                "{JIG_TAG}, {peers} jig peer{}",
                if peers == 1 { "" } else { "s" }
            ))
        } else {
            Check::Fail(format!(
                "not tagged {JIG_TAG} — {JOIN_HINT} with a {JIG_TAG} key"
            ))
        };
        (connected, enrolled)
    }
}

impl WhoIs {
    fn parse(json: &str) -> Result<Self, NetError> {
        serde_json::from_str(json).map_err(|source| NetError::Parse {
            command: "tailscale whois".into(),
            source,
        })
    }

    /// The caller's node id, if it is a jig peer.
    fn jig_node(&self) -> Option<NodeId> {
        has_jig_tag(&self.node.tags).then(|| NodeId::new(&self.node.stable_id))
    }
}

// ── Backend ─────────────────────────────────────────────────────────

/// The `tailscale` backend.
pub struct TailscaleNet {
    port: u16,
    listener: LazyListener,
}

impl TailscaleNet {
    /// `port` is where jig listens and dials on every tailnet node.
    pub fn new(port: u16) -> Self {
        Self {
            port,
            listener: LazyListener::default(),
        }
    }

    /// Enroll this machine: `tailscale up` with a pre-authorized key that
    /// advertises [`JIG_TAG`]. `login_server` points at a Headscale.
    pub fn join(authkey: &str, login_server: Option<&str>) -> Result<(), NetError> {
        let mut exec = tailscale()
            .arg("up")
            .arg(format!("--authkey={authkey}"))
            .arg(format!("--advertise-tags={JIG_TAG}"))
            .labeled("tailscale up")
            .timeout(Timeout::NETWORK);
        if let Some(server) = login_server {
            exec = exec.arg(format!("--login-server={server}"));
        }
        let out = exec.run()?;
        if out.success() {
            Ok(())
        } else {
            Err(NetError::Command {
                command: "tailscale up".into(),
                detail: out.failure(),
            })
        }
    }

    fn status(&self) -> Result<Status, NetError> {
        Status::parse(&run(&["status", "--json"])?)
    }

    fn listen_addr(&self) -> Result<SocketAddr, NetError> {
        let out = run(&["ip", "-4"])?;
        let ip: IpAddr = out
            .lines()
            .next()
            .and_then(|l| l.trim().parse().ok())
            .ok_or_else(|| NetError::NotConnected("tailscale has no IPv4 address".into()))?;
        Ok(SocketAddr::new(ip, self.port))
    }

    fn whois(ip: IpAddr) -> Result<Option<NodeId>, NetError> {
        Ok(WhoIs::parse(&run(&["whois", "--json", &ip.to_string()])?)?.jig_node())
    }
}

impl Net for TailscaleNet {
    fn kind(&self) -> NetKind {
        NetKind::Tailscale
    }

    fn local(&self) -> Result<Node, NetError> {
        Ok(self.status()?.me()?.node())
    }

    fn peers(&self) -> Result<Vec<Node>, NetError> {
        let status = self.status()?;
        status.me()?;
        Ok(status
            .jig_peers()
            .into_iter()
            .map(PeerStatus::node)
            .collect())
    }

    fn accept(&self) -> Result<(Box<dyn Stream>, NodeId), NetError> {
        self.listener.accept(|| self.listen_addr(), Self::whois)
    }

    fn dial(&self, peer: &NodeId) -> Result<Box<dyn Stream>, NetError> {
        let status = self.status()?;
        let ip = status
            .jig_peers()
            .into_iter()
            .find(|p| p.id == peer.as_str())
            .and_then(PeerStatus::addr)
            .ok_or_else(|| NetError::UnknownPeer(peer.clone()))?;
        listener::connect(SocketAddr::new(ip, self.port))
    }

    fn health(&self) -> NetHealth {
        let installed = match tailscale().arg("version").timeout(Timeout::QUICK).run() {
            Ok(out) if out.success() => Check::Ok(out.stdout.lines().next().unwrap_or("").into()),
            Ok(out) => Check::Fail(out.failure()),
            Err(ExecError::Spawn { .. }) => {
                return NetHealth {
                    installed: Check::Fail("not installed — https://tailscale.com/download".into()),
                    connected: Check::Warn("skipped".into()),
                    enrolled: Check::Warn("skipped".into()),
                }
            }
            Err(e) => Check::Fail(e.to_string()),
        };
        let (connected, enrolled) = match self.status() {
            Ok(status) => status.assess(),
            Err(e) => (Check::Fail(e.to_string()), Check::Warn("skipped".into())),
        };
        NetHealth {
            installed,
            connected,
            enrolled,
        }
    }
}

fn tailscale() -> Exec {
    Exec::command(Command::new("tailscale")).capturing()
}

/// Run a quick, local `tailscale` query and return its stdout.
fn run(args: &[&str]) -> Result<String, NetError> {
    let command = format!("tailscale {}", args[0]);
    let mut exec = tailscale().labeled(&command).timeout(Timeout::QUICK);
    for arg in args {
        exec = exec.arg(arg);
    }
    let out = exec.run()?;
    if out.success() {
        Ok(out.stdout)
    } else {
        Err(NetError::Command {
            command,
            detail: out.failure(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS: &str = include_str!("fixtures/tailscale_status.json");
    const NEEDS_LOGIN: &str = include_str!("fixtures/tailscale_status_needs_login.json");
    const WHOIS: &str = include_str!("fixtures/tailscale_whois.json");
    const WHOIS_UNTAGGED: &str = include_str!("fixtures/tailscale_whois_untagged.json");

    #[test]
    fn local_is_this_node() {
        let status = Status::parse(STATUS).unwrap();
        assert_eq!(
            status.me().unwrap().node(),
            Node {
                name: "devbox".into(),
                id: NodeId::new("nSELF11CNTRL"),
            }
        );
    }

    #[test]
    fn peers_are_only_the_tagged_nodes() {
        let status = Status::parse(STATUS).unwrap();
        let peers: Vec<Node> = status.jig_peers().into_iter().map(|p| p.node()).collect();
        assert_eq!(
            peers,
            vec![
                Node {
                    name: "build".into(),
                    id: NodeId::new("nBUILD4CNTRL"),
                },
                Node {
                    name: "laptop".into(),
                    id: NodeId::new("nLAPTOP2CNTRL"),
                },
            ]
        );
    }

    #[test]
    fn a_peer_is_dialed_on_its_ipv4_address() {
        let status = Status::parse(STATUS).unwrap();
        let laptop = status
            .jig_peers()
            .into_iter()
            .find(|p| p.id == "nLAPTOP2CNTRL")
            .unwrap();
        assert_eq!(laptop.addr(), Some("100.101.1.2".parse().unwrap()));
    }

    #[test]
    fn a_running_tagged_node_is_healthy() {
        let (connected, enrolled) = Status::parse(STATUS).unwrap().assess();
        assert_eq!(
            connected,
            Check::Ok("100.101.1.1 on operator@example.com".into())
        );
        assert_eq!(enrolled, Check::Ok("tag:jig, 2 jig peers".into()));
    }

    #[test]
    fn an_untagged_node_is_not_enrolled() {
        let json = STATUS.replacen(r#""Tags": ["tag:jig"],"#, "", 1);
        let (connected, enrolled) = Status::parse(&json).unwrap().assess();
        assert!(matches!(connected, Check::Ok(_)), "{connected:?}");
        assert!(enrolled.is_fail(), "{enrolled:?}");
        assert!(enrolled.detail().contains("jig fleet join"), "{enrolled:?}");
    }

    #[test]
    fn a_logged_out_node_has_no_peers_and_says_how_to_join() {
        let status = Status::parse(NEEDS_LOGIN).unwrap();
        assert!(status.jig_peers().is_empty());
        assert!(matches!(status.me(), Err(NetError::NotConnected(_))));

        let (connected, enrolled) = status.assess();
        assert!(connected.is_fail());
        assert!(connected.detail().contains("NeedsLogin"), "{connected:?}");
        assert!(
            connected.detail().contains("jig fleet join"),
            "{connected:?}"
        );
        assert!(enrolled.is_fail());
    }

    #[test]
    fn whois_identifies_a_tagged_caller() {
        let whois = WhoIs::parse(WHOIS).unwrap();
        assert_eq!(whois.jig_node(), Some(NodeId::new("nLAPTOP2CNTRL")));
    }

    #[test]
    fn whois_refuses_an_untagged_caller() {
        let whois = WhoIs::parse(WHOIS_UNTAGGED).unwrap();
        assert_eq!(whois.jig_node(), None);
    }

    #[test]
    fn unexpected_output_is_a_parse_error() {
        let err = Status::parse(r#"{"Peer": {}}"#).unwrap_err();
        assert!(matches!(err, NetError::Parse { .. }), "{err:?}");
        let err = WhoIs::parse("not json").unwrap_err();
        assert!(matches!(err, NetError::Parse { .. }), "{err:?}");
    }
}
