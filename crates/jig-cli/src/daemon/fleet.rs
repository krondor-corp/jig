//! The fleet: this daemon on the private network, and its peers.
//!
//! Peers speak the protocol the unix socket does (`ipc`'s newline-delimited
//! JSON, one [`Request`] and one [`Response`] per connection), carried over a
//! [`Net`] stream. Two halves, like `ipc`:
//!
//! * **Server**: [`Fleet::spawn`] answers peers on the `Net` listener.
//!   Every connection [`Net::accept`] returns is already from a trusted peer;
//!   what each request may do is [`Fleet::answer`]'s call: reads always,
//!   commands only with `[fleet] net_commands = true`, `Shutdown` never.
//! * **Client**: [`Fleet::poll`] asks every peer for its frame, and
//!   [`Fleet::command`] sends a verb to the peer that owns the worker. The
//!   web view uses both, so any machine's page shows and drives the fleet.
//!
//! Nothing here goes through a peer to a third machine: a peer only ever
//! answers for itself. See `docs/fleet-spec.md`, "Topology".

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use jig_core::net::{Net, NetError, Node, NodeId, Stream};

use super::commands::{Command, WorkerKey};
use super::ipc::{self, DaemonStatus, IpcError, Request, Response};
use super::DaemonShared;

/// How long a status poll waits on a peer. Short: the web view polls every
/// few seconds, and one slow peer must not hold up the rest of the page.
const POLL_TIMEOUT: Duration = Duration::from_secs(3);

/// How long the listener waits after a failed `accept` before trying again.
const ACCEPT_BACKOFF: Duration = Duration::from_secs(1);

/// One peer's answer to a poll.
#[derive(Debug, Clone)]
pub struct PeerStatus {
    pub node: Node,
    /// Its frame: this poll's, or when [`Self::error`] is set the last one
    /// it gave, so a peer that drops out stays on the page, marked stale.
    /// `None` if it has never answered.
    pub frame: Option<DaemonStatus>,
    /// When [`Self::frame`] was had, in unix seconds.
    pub seen_at: Option<i64>,
    /// Why it did not answer this poll (down, refused, too slow).
    pub error: Option<String>,
}

/// This machine on the fleet network.
pub struct Fleet {
    net: Box<dyn Net>,
    /// What this machine is called: in refusals sent to peers.
    name: String,
    /// `[fleet] net_commands`: whether peers may run verbs here.
    commands: bool,
    /// Each peer's last good frame and when it came: what a peer that
    /// stops answering is shown as.
    seen: Mutex<HashMap<NodeId, (DaemonStatus, i64)>>,
}

impl Fleet {
    pub fn new(net: Box<dyn Net>, name: impl Into<String>, commands: bool) -> Self {
        Self {
            net,
            name: name.into(),
            commands,
            seen: Mutex::default(),
        }
    }

    // ── Client ──────────────────────────────────────────────────────

    /// Every peer's frame, polled in parallel. A peer that does not answer
    /// is in the list with its error (and its last frame, if it ever gave
    /// one), never left out: the page says it is unreachable rather than
    /// pretending it has no workers.
    pub fn poll(&self) -> Result<Vec<PeerStatus>, NetError> {
        let peers = self.net.peers()?;
        Ok(std::thread::scope(|scope| {
            let polls: Vec<_> = peers
                .into_iter()
                .map(|node| scope.spawn(move || self.poll_one(node)))
                .collect();
            polls
                .into_iter()
                .map(|poll| poll.join().expect("peer poll panicked"))
                .collect()
        }))
    }

    fn poll_one(&self, node: Node) -> PeerStatus {
        let answer = match self.request(&node.id, &Request::GetStatus, POLL_TIMEOUT) {
            Ok(Response::Status(status)) => Ok(*status),
            Ok(other) => Err(format!("expected status, got {other:?}")),
            Err(e) => Err(e.to_string()),
        };
        let mut seen = self.seen.lock().unwrap_or_else(|e| e.into_inner());
        match answer {
            Ok(frame) => {
                let now = chrono::Utc::now().timestamp();
                seen.insert(node.id.clone(), (frame.clone(), now));
                PeerStatus {
                    node,
                    frame: Some(frame),
                    seen_at: Some(now),
                    error: None,
                }
            }
            Err(error) => {
                let last = seen.get(&node.id).cloned();
                PeerStatus {
                    node,
                    frame: last.as_ref().map(|(frame, _)| frame.clone()),
                    seen_at: last.map(|(_, at)| at),
                    error: Some(error),
                }
            }
        }
    }

    /// Run `cmd` on `worker` on the peer `node`, which owns it. Answers as
    /// the local executor does: `Ok`, or `Error` with the peer's reason.
    pub fn command(&self, node: &NodeId, worker: WorkerKey, cmd: Command) -> Response {
        let request = cmd.request(worker);
        let timeout = request.client_timeout();
        match self.request(node, &request, timeout) {
            Ok(response) => response,
            Err(e) => Response::Error(format!("{node}: {e}")),
        }
    }

    /// One request to one peer.
    fn request(
        &self,
        node: &NodeId,
        request: &Request,
        timeout: Duration,
    ) -> Result<Response, IpcError> {
        let mut stream = self.net.dial(node).map_err(|e| match e {
            NetError::Io(io) => IpcError::Io(io),
            other => IpcError::Protocol(other.to_string()),
        })?;
        stream.set_timeout(Some(timeout))?;
        ipc::write_message(&mut stream, request)?;
        ipc::read_message(&mut stream)
    }

    // ── Server ──────────────────────────────────────────────────────

    /// Answer peers on the `Net` listener, on a thread of its own, until
    /// the process exits.
    ///
    /// `accept` blocks with no way to interrupt it, so this thread is never
    /// joined: a stopping daemon leaves it behind, and it ends with the
    /// process. A peer's request answers on a thread per connection, so a
    /// slow one (a verb, or a peer that went quiet) never holds up the next.
    pub fn spawn(self: Arc<Self>, shared: DaemonShared) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new()
            .name("jig-net".into())
            .spawn(move || self.listen(&shared))
            .expect("failed to spawn jig-net thread")
    }

    fn listen(self: &Arc<Self>, shared: &DaemonShared) {
        tracing::info!(net = %self.net.kind(), commands = self.commands, "fleet listener starting");
        let mut accepted = false;
        loop {
            match self.net.accept() {
                Ok((stream, from)) => {
                    accepted = true;
                    let fleet = Arc::clone(self);
                    let shared = shared.clone();
                    let spawned = std::thread::Builder::new()
                        .name("jig-peer".into())
                        .spawn(move || fleet.serve(stream, &from, &shared));
                    if let Err(e) = spawned {
                        tracing::warn!("peer request dropped: {}", e);
                    }
                }
                // The first `accept` binds: failing before anything ever
                // connected means there is no listener (a taken port, an
                // address this machine lacks). Polling peers still works.
                Err(e) if !accepted => {
                    tracing::warn!("fleet listener not started: {}", e);
                    return;
                }
                Err(e) => {
                    tracing::warn!("fleet accept failed: {}", e);
                    std::thread::sleep(ACCEPT_BACKOFF);
                }
            }
        }
    }

    /// Read one request from a peer, answer it, close.
    fn serve(&self, mut stream: Box<dyn Stream>, from: &NodeId, shared: &DaemonShared) {
        let served = (|| -> Result<(), IpcError> {
            stream.set_timeout(Some(ipc::SERVER_TIMEOUT))?;
            let request: Request = ipc::read_message(&mut stream)?;
            tracing::info!(peer = %from, ?request, "peer request");
            // A verb runs as long as it runs (`retry-setup` is a whole
            // `on_create`); the peer's own timeout bounds its wait.
            stream.set_timeout(None)?;
            let response = ipc::guarded(|| self.answer(request, shared));
            stream.set_timeout(Some(ipc::SERVER_TIMEOUT))?;
            ipc::write_message(&mut stream, &response)
        })();
        if let Err(e) = served {
            tracing::warn!(peer = %from, "peer request failed: {}", e);
        }
    }

    /// What a peer may have: the security invariant in `docs/fleet-spec.md`.
    fn answer(&self, request: Request, shared: &DaemonShared) -> Response {
        match request.into_command() {
            Ok((worker, cmd)) if self.commands => shared.run_command(worker, cmd),
            Ok(_) => Response::Error(format!(
                "{} does not take commands over the network (set `net_commands = true` under [fleet] there)",
                self.name
            )),
            Err(Request::Ping) => Response::Pong(shared.info()),
            Err(Request::GetStatus) => Response::Status(Box::new(shared.status())),
            Err(Request::Shutdown) => {
                Response::Error("shutdown is only taken on the local socket".into())
            }
            Err(other) => Response::Error(format!("{other:?} is not answered over the network")),
        }
    }
}
