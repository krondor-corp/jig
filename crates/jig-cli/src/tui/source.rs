//! Where the TUI's data comes from, and where its commands go: background
//! threads that report back to the event loop as [`Msg`]s.
//!
//! The TUI is a client, like `jig ps`. This machine's workers come from its
//! daemon over the unix socket, or straight off disk when none is running
//! (`ps`'s own [`Frame`]); every peer's come from the fleet network
//! ([`Fleet::poll`], as `jig ps --fleet` asks). A verb runs where its
//! worker lives: on this machine through the daemon (or in-process without
//! one, as the CLI verbs do), on a peer through [`Fleet::command`].

use std::sync::Arc;
use std::time::Duration;

use flume::{Receiver, RecvTimeoutError, Sender};

use jig_core::net::NodeId;

use super::toast::Toast;
use crate::cli::commands::ps::{daemon_frame, Frame};
use crate::context::{AppCtx, Ctx};
use crate::daemon::commands::{self, Command, WorkerKey};
use crate::daemon::fleet::Fleet;
use crate::daemon::ipc::{self, IpcError, Response};
use crate::worker::events::WorkerState;

/// What a background thread tells the event loop.
pub enum Msg {
    /// A fresh look at every machine.
    Fleet(FleetFrame),
    /// What a command did.
    Outcome(Toast),
}

/// Every machine's workers, this one first.
#[derive(Debug, Clone, Default)]
pub struct FleetFrame {
    pub machines: Vec<MachineFrame>,
    /// Why the peers could not be listed (the network is down, say).
    pub peers_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MachineFrame {
    pub name: String,
    /// Its node id on the fleet network; `None` for this machine.
    pub node: Option<NodeId>,
    pub workers: Vec<WorkerState>,
    /// Why it did not answer this time. Its workers, if any, are the last
    /// it gave.
    pub error: Option<String>,
    /// When its workers were had, for a machine that stopped answering.
    pub seen_at: Option<i64>,
    /// Whether this machine's daemon is running (only ever false for this
    /// machine: a peer is its daemon).
    pub daemon: bool,
}

impl MachineFrame {
    pub fn is_local(&self) -> bool {
        self.node.is_none()
    }

    /// Answering now, as opposed to stale or never seen.
    pub fn is_up(&self) -> bool {
        self.error.is_none()
    }
}

/// Look at every machine every `interval`, and at once whenever something
/// arrives on `refresh`, until the event loop hangs up.
pub fn spawn_poller(
    ctx: Ctx,
    fleet: Option<Arc<Fleet>>,
    name: String,
    interval: Duration,
    refresh: Receiver<()>,
    tx: Sender<Msg>,
) {
    std::thread::Builder::new()
        .name("jig-tui-poll".into())
        .spawn(move || loop {
            let frame = look(&ctx, fleet.as_deref(), &name);
            if tx.send(Msg::Fleet(frame)).is_err() {
                return;
            }
            match refresh.recv_timeout(interval) {
                Ok(()) | Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
        })
        .expect("failed to spawn jig-tui-poll thread");
}

/// This machine, then every peer.
fn look(ctx: &Ctx, fleet: Option<&Fleet>, name: &str) -> FleetFrame {
    let (local, daemon) = match daemon_frame(&ctx.paths, None) {
        Some(frame) => (frame, true),
        None => (Frame::from_disk(ctx, None), false),
    };
    let mut machines = vec![MachineFrame {
        name: name.to_string(),
        node: None,
        workers: local.workers,
        error: None,
        seen_at: None,
        daemon,
    }];
    let peers_error = match fleet.map(Fleet::poll) {
        None => None,
        Some(Ok(peers)) => {
            machines.extend(peers.into_iter().map(|peer| {
                MachineFrame {
                    name: peer.node.name,
                    node: Some(peer.node.id),
                    workers: peer
                        .frame
                        .map(|f| f.workers.iter().map(|w| w.to_display_state()).collect())
                        .unwrap_or_default(),
                    error: peer.error,
                    seen_at: peer.seen_at,
                    daemon: true,
                }
            }));
            None
        }
        Some(Err(e)) => Some(e.to_string()),
    };
    FleetFrame {
        machines,
        peers_error,
    }
}

/// Run `cmd` on `key`'s worker, on `machine` (`None`: this one), on a
/// thread of its own: a verb can take as long as `on_create` does. The
/// outcome comes back as a toast, then a refresh.
pub fn run_command(
    app: Arc<AppCtx>,
    fleet: Option<Arc<Fleet>>,
    machine: Option<NodeId>,
    key: WorkerKey,
    cmd: Command,
    tx: Sender<Msg>,
    refresh: Sender<()>,
) {
    std::thread::Builder::new()
        .name("jig-tui-command".into())
        .spawn(move || {
            let what = format!("{} {key}", cmd.name());
            let result = match &machine {
                None => local(&app, key, cmd),
                Some(node) => match &fleet {
                    Some(fleet) => match fleet.command(node, key, cmd) {
                        Response::Ok => Ok(()),
                        Response::Error(reason) => Err(reason),
                        other => Err(format!("unexpected answer: {other:?}")),
                    },
                    None => Err("this machine has no fleet network".into()),
                },
            };
            let toast = match result {
                Ok(()) => Toast::success(format!("{what}: done")),
                Err(reason) => Toast::failure(format!("{what}: {reason}")),
            };
            let _ = tx.send(Msg::Outcome(toast));
            let _ = refresh.send(());
        })
        .expect("failed to spawn jig-tui-command thread");
}

/// Through this machine's daemon, or in this process when none answers,
/// as the CLI verbs do.
fn local(app: &AppCtx, key: WorkerKey, cmd: Command) -> Result<(), String> {
    match ipc::command(&app.paths, key.clone(), cmd.clone()) {
        Ok(true) => Ok(()),
        Ok(false) => commands::execute(app, &key, &cmd).map_err(|e| e.to_string()),
        Err(IpcError::Refused(reason)) => Err(reason),
        Err(e) => Err(e.to_string()),
    }
}
