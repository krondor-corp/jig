# jig fleet spec — one view of all work, everywhere

Status: accepted direction (2026-08-31), networking revised 2026-10-22. Phase 1
of this spec — the daemon-to-service refactor — shipped in v0.9.0; it lives in
`crates/jig-cli/src/daemon/` and its plan doc has been retired.

The 2026-10-22 revision replaces the hub-and-spoke iroh transport with a peer
mesh over a **pluggable private network** (`Net`), chosen per machine the way
`mux` is. Tailscale is the first backend; iroh becomes a later backend rather
than the only one. The web view moves from stretch goal to core.

## The problem

Work runs across multiple machines — auto-spawned by jig on the remote dev server,
hand-created worktrees locally — and there is no single place to answer:

1. What issues are in the queue?
2. What is the current review state?
3. Which agents are *genuinely* stuck?
4. Which setups failed, and why?
5. How do I prune, nudge, or switch the model of a task on another machine?

herdr solves the *terminal* half of this (persistent PTYs, attach from anywhere,
`herdr --remote` as a client over remote sessions). It cannot solve the *work* half:
herdr knows a pane went idle; it does not know whether that means "done and merged,"
"done but no PR," "CI red," "setup never finished," or "waiting on review."

## Identity

**jig is the delivery loop — issue → worktree → agent → PR → merge — and its control
plane.** Not a multiplexer, not a terminal UI, not a network.

- **Mux-agnostic by design.** The `Mux` trait (`crates/jig-core/src/mux/`) has two
  first-class backends chosen per device in config: `tmux` and `herdr`.
  Capabilities degrade honestly: `agent_state() == None` on tmux means the monitor
  judges from the event log alone.
- **Network-agnostic by the same design.** The `Net` trait (below) is chosen per
  device in config. jig never runs a public endpoint and never implements login:
  reachability and identity come from a private network the operator already
  trusts.
- **Terminal viewing/attach is delegated**, always: `jig attach` locally,
  `herdr --remote` across machines. jig builds no terminal UI, ever.
- **The fleet view is a *work* view** — branches, pipeline state, setup state,
  PR/CI/review health, issue linkage, stuck-ness — joined across every machine,
  as `jig ps --fleet` and as a web page on the private network.

## Why the distributed problem is small

The questions decompose by data source:

| Question | Source of truth | Distribution needed |
|----------|----------------|---------------------|
| Issue queue | Linear (cloud) | none — each daemon polls it for its repos (as today) |
| Review state | GitHub (cloud) | none — each daemon polls it for its workers (`daemon/checks.rs`) |
| Genuinely stuck / setup failed | per-machine: event log + mux `agent_state` | **read**: peers ask each other for status |
| Prune / nudge / switch model | the machine owning the worker | **command**: sent to the owning peer |

Only the last two rows cross machines, and both are request/response against
the machine that owns the data. There is no replicated state and no aggregator
that has to be up for anything to work.

## Topology: a peer mesh that polls

```
     ┌──────────┐        ┌──────────┐        ┌──────────┐
     │ dev srv  │◄──────►│  laptop  │◄──────►│  box N   │   jig daemons, one per machine
     │ daemon   │        │  daemon  │        │  daemon  │   · local monitor loop, unchanged
     └────▲─────┘        └────▲─────┘        └────▲─────┘   · answer Status for own workers
          └───────────────────┴───────────────────┘         · execute commands for own workers
                    private network (Net)                    · serve the fleet view
                          ▲
                          │ browser on any enrolled device (phone included,
                          │ for IP-based backends)
```

- **Every daemon is a peer.** It answers `Status` for its own workers over the
  private network and executes commands for its own workers. Nothing else.
- **Any peer can render the fleet.** `jig ps --fleet` (or the web page) asks
  `Net::peers()` who is out there, requests `Status` from each, and joins the
  results with a `MACHINE` column. A peer that doesn't answer is rendered stale,
  never guessed at.
- **Commands go to the owner.** `jig kill --machine laptop X` connects to
  `laptop` and asks it; the owner checks the caller (`Net::identify`) before
  acting.
- **No hub.** Polling a handful of machines every few seconds is cheap. If
  cross-machine coordination is ever needed (queue assignment, Phase 6), a
  coordinator is a role a peer takes on, not a new deployment.

The wire protocol is the existing one in `daemon/ipc.rs` — newline-delimited
JSON, one `Request` in and one `Response` out per connection — carried over a
`Net` stream instead of only the unix socket. Local commands keep using the
unix socket directly.

## Networking: the `Net` trait

jig needs exactly four things from a network: somewhere to listen, a list of
peers, who is calling, and whether the network is up. `Net` asks for those and
nothing more, at the level of authenticated streams so that address-based
networks and key-based ones fit behind the same trait:

```rust
pub trait Net {
    /// This machine: its name and node id on the network.
    fn local(&self) -> Result<Node, NetError>;
    /// The other jig machines this one can see.
    fn peers(&self) -> Result<Vec<Node>, NetError>;
    /// The next incoming connection and who it is from.
    fn accept(&self) -> Result<(Box<dyn Stream>, NodeId), NetError>;
    /// Open a connection to a peer.
    fn dial(&self, peer: &NodeId) -> Result<Box<dyn Stream>, NetError>;
    /// Installed? Connected? This machine enrolled? For `jig health`.
    fn health(&self) -> NetHealth;
}
```

Configured per machine, like `mux`:

```toml
# ~/.config/jig/config.toml
[fleet]
net = "tailscale"        # "tailscale" | "static" | (later) "iroh"
```

### Backends

| Backend | `peers` | `accept` identity | Operator does once | jig can own |
|---------|---------|-------------------|--------------------|-------------|
| `tailscale` | `tailscale status --json`, filtered to `tag:jig` | `tailscale whois <ip>` on the TCP peer | install Tailscale | `jig fleet join <authkey>` runs `tailscale up --authkey … --advertise-tags=tag:jig`; discovery; health |
| `tailscale` + Headscale | same | same | install Tailscale; run Headscale on the dev server | same, pointed at the operator's control server |
| `static` | listed in config | source address checked against an allowlist | bring your own WireGuard / ZeroTier / Nebula / NetBird / LAN | binding, peer list, allowlist |
| `iroh` (later) | enrolled node keys | the QUIC peer's node key | nothing — embedded in the binary | everything, `jig fleet join <ticket>` |

- **`tailscale` is the first backend.** Like the `tmux` and `herdr` backends it
  shells out through `Exec` — no new dependencies, no async runtime. It gives
  automatic discovery by tag, per-connection identity, and a fleet page any
  enrolled device's browser can open, phone included.
- **`static` is the escape hatch** for any private IP network. Address-based
  identity is sound on WireGuard-family networks, where an address can only be
  used by the holder of its key; on a plain LAN it is only as good as the LAN.
- **Headscale needs no backend of its own** — it is Tailscale's control server,
  self-hosted, and clients are the normal `tailscale` client.
- **`iroh` is the most plug-and-play** (nothing to install) but a browser cannot
  join it, so the web view needs an IP backend, and its tokio would reach past
  the one thread the web view keeps it in today. It is designed for, not built
  first. `zim-peer`
  (krondor-corp/zim) remains the reference for its accept policy and peer
  registry.

What jig can never own on `tailscale` is the login itself — proving a machine
belongs to the operator. A reusable, pre-tagged auth key reduces that to one
command per machine.

### Security invariant

Every request arriving over `Net` is checked against the caller's identity
before it is answered:

- **Reads** (`Ping`, `Status`) — any trusted peer. On `tailscale`, trusted means
  tagged `tag:jig` on the operator's tailnet (tailnet ACLs restrict the jig port
  to that tag as well); on `static`, the configured allowlist; on `iroh`, the
  enrolled keys.
- **Commands** (`kill`, `nudge`, `prune`, `retry-setup`, `model`) — trusted
  peers only, refused (not queued) otherwise. Over `Net`, commands can be enabled
  separately from reads, behind the same check.
- The `Net` listener is **off by default**; enabling `[fleet]` is opt-in per
  machine. Nothing ever binds a public interface.

## Setup state is fleet state

The fleet view can only show what the event log records in structured form. A
failed `on_create` must therefore be data, not a string:

- `on_create` output is kept in `setup.log` beside the worker's `events.jsonl`,
  and its failure reason carries exit status, an output tail, and the log path
  (KRO-236).
- Setup becomes a first-class part of `WorkerState` and `WorkerSnapshot` —
  phase, duration, and on failure the tail and log path — so `ps`, the fleet
  view, and notifications all render it from the same field.
- Failed workers stay visible in `ps` until pruned; `retry-setup` re-runs the
  hook in the existing worktree. `setup.log` itself is fetched from the owning
  peer on demand; only the tail travels in `Status`.

## Model switching

`jig model <branch> <model> [--machine <name>]`:

- **v0 (local):** the agent session is live in a mux window — send `/model
  <model>` through `Mux::send_message`. Record the change as an event so `ps`
  shows the current model.
- **v1 (routed):** same command from any peer, sent to the owning machine.
- **v2 (cold switch):** for a worker that must restart, the owning daemon kills
  the window and re-runs `Agent::resume` with the new `--model`. Requires
  persisting model + session id on the worker record.

The same routing carries the rest of the control verbs: `kill`, `nudge`,
`prune`, `retry-setup`, `spawn --machine`.

## The fleet view

`jig ps --fleet`: one table joining every peer's `Status` —
`MACHINE · WORKER · STATE · SETUP · AGENT (live mux state) · MODEL · PR · CI · REVIEW · ISSUE`.
"Genuinely stuck" and "setup failed" are first-class filters.

The web view serves the same table over HTTP on the `Net` listen address (IP
backends only), with a failed-setups panel and buttons for the command verbs. It
renders work state only — attach links can deep-link to `herdr --remote`
targets, but terminals never render in jig. It is reachable only from enrolled
devices; there is no public page and no login.

## Phases

1. **Daemon as a service** — ✅ shipped in v0.9.0: unix-socket IPC,
   `WorkerSnapshot`, `jig daemon start/stop/status/logs/install/uninstall`,
   `service-manager`, `mux` selection, per-tick mux snapshot batching,
   `jig health` herdr detection.
2. **Local control verbs + state fusion** — `jig model` v0; monitor uses
   `agent_state` where present: never nudge `blocked` (notify: "needs approval —
   `jig attach X`"), suppress idle-nudges while `working`, unchanged when `None`.
3. **Local control plane** — everything the fleet needs, built and useful on
   one machine first, so networking later only changes who can reach it:
   - **Machine identity** — every daemon has a stable machine name and node id,
     carried in `Ping`/`Status` and rendered by `ps`. Done in the daemon: the
     name is `[fleet] name` (default: hostname), the node id a UUID minted once
     into `~/.config/jig/node_id` (`crates/jig-cli/src/context/machine.rs`);
     both ride on `DaemonInfo.machine` and show in `jig daemon status`.
   - **Setup as state** — KRO-236 (setup log, real failure reasons in
     notifications), then structured setup state on `WorkerState`/`WorkerSnapshot`,
     failed workers visible in `ps`, notification dedupe across daemon restarts.
     Done locally: `Worker` records each phase (`worktree → copy_files →
     on_create → agent_launch`) as `SetupStarted`/`SetupFinished` events,
     which reduce to `WorkerState.setup` (`SetupState`:
     `Pending | Running | Ok | Failed`) and travel in `WorkerSnapshot`;
     `ps` renders it in STATE (`on_create 42s`, `failed: on_create (exit 1)`)
     and keeps failed and still-setting-up workers listed. One-shot alerts
     (`needs_intervention`, `pr_opened`) write a `Notified { kind }` event and
     are skipped once it is in the log.
   - **Daemon commands** — `kill`, `nudge` (with free text), `prune`,
     `retry-setup`, `model` as IPC `Request`s the daemon executes, so the CLI and
     the web view drive the same code path. Done: each verb is addressed by
     worker key (`repo/branch`) and answers `Ok` or `Error(reason)`; the one
     executor is `crates/jig-cli/src/daemon/commands.rs`, which the CLI also
     runs in-process when no daemon answers. Verbs run on a thread per request
     (never the listener's or the tick loop's), one at a time per worker, and
     each leaves a `command` event in the worker's log. `prune` refuses a
     worker whose window is still open; `retry-setup` only takes a worker whose
     setup failed (any phase, the agent launch included), per `WorkerState.setup`.
   - **Local web view** — the daemon serves the work view and command buttons
     over HTTP on `127.0.0.1`. Hardened against browser-borne requests
     (Host-header check, no state change on GET, a per-session token).
     Done: `[fleet] web = true` (`web_port`, default 7477) binds
     `127.0.0.1` only. The server is axum (`crates/jig-cli/src/daemon/web/`),
     run as a `runtime::Service` (zim's trait) by a `runtime::Supervisor` on
     one `jig-web` thread with a current-thread tokio runtime. The rest of
     the daemon stays std threads; handlers make their blocking daemon calls
     on tokio's blocking pool, and a stopping daemon gives in-flight requests
     10s before abandoning them. It serves a JSON API under `/api/v0`
     defined by the `jig-api` crate in zim-api's pattern: an `ApiRequest`
     trait, a module per subject and a file per request (`session`,
     `status`, `workers/{kill,nudge,prune,retry-setup,model,setup-log}`),
     each exporting the `PATH` the daemon mounts its handler on, and a
     generic `Client` that the Yew app (`crates/jig-cli/web`, built by
     Trunk, embedded with `rust-embed`) and the daemon tests both call. Rows
     come from the same `worker_columns` `ps` renders. The app adds PR
     links, a failed-setups panel (the tail, and the whole `setup.log` on
     request, read only from a path the daemon recorded) and a button per
     verb. Verbs need the session token in `X-Jig-Token`, which
     `Client::with_token` sends, and run through
     `DaemonShared::run_command`. The URL rides on `DaemonInfo.web`; `jig
     web` opens it and `jig daemon status` prints it.
4. **Net** — the `Net` trait and `[fleet] net` config with `static` and
   `tailscale` backends; `jig health` checks; `jig fleet join`. Done: the
   trait and both backends live in `crates/jig-core/src/net/`, built from
   `FleetConfig::net` (`[fleet] net`, `net_port`, `[fleet.static]`). `accept`
   drops untrusted callers itself (no `tag:jig` on `whois`, or not on the
   static allowlist / peer list), so callers only ever see trusted peers;
   nothing binds until `accept` is first called, and nothing calls it yet.
   `jig health` shows a `Net` section (installed, connected, enrolled);
   `jig fleet join <authkey> [--login-server URL]` runs `tailscale up`.
   Still to come: the daemon's
   protocol and web view served on the `Net` listener, gated by `accept`
   identity; `jig ps --fleet` and a fleet web view that poll every peer;
   commands routed to the owning peer.
5. **Queue across machines** — assigning queued issues to machines. There is no
   per-repo worker cap to build on (`max_concurrent_workers` was removed), so a
   per-machine limit, if wanted, is designed here. `jig spawn --machine <name>`
   routed like any command.
6. **`iroh` backend** — when zero-install enrollment is worth an async runtime.

## Risks

| Risk | Mitigation |
|------|------------|
| Scope: this is a small distributed system | No replicated state, no hub, one existing protocol; phases 2–3 are useful on one machine if 4+ stall |
| Command channel = remote execution | Off by default, private network only, every request checked against `Net` identity, commands refused for untrusted callers, commands enabled separately from reads |
| Vendor dependency on Tailscale | `Net` is a trait: Headscale needs no code, `static` covers any private network, `iroh` stays designed-for |
| Address-based identity on `static` | Documented as only as strong as the underlying network; recommended only on WireGuard-family overlays |
| Stale peers misleading the fleet view | A peer that doesn't answer is rendered stale with its `last_seen`; the view never fabricates liveness |
| herdr pre-1.0 CLI drift (backend dep) | Typed fail-loud parsing, version guard, tmux backend fully supported |
| Two views drift (local `ps` vs. fleet) | Both render from the same `WorkerSnapshot` type; fleet adds columns, never diverges in derivation |

## Non-goals

- A terminal UI or terminal fleet view (herdr's product; `herdr --remote` covers it).
- A public endpoint, accounts, or login of any kind — identity is the private network's.
- Running or reimplementing a VPN — jig uses one through `Net`, never ships one.
- Making herdr (or tmux), or Tailscale (or iroh), the sole backend — per-device choice is the point.
- Writing mux `agent_state` into the event log (it's a sensor; the log stays the
  pipeline source of truth).
- General-purpose task queue for non-jig work.
- Multi-user fleets — this is one operator's machines. Revisit if that changes.
