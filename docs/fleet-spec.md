# jig fleet spec — one view of all work, everywhere

Status: accepted direction (2026-08-31). Supersedes the earlier "all-in on herdr"
framing. Companion doc: [daemon-service-plan.md](./daemon-service-plan.md) — the
daemon-to-service refactor is Phase 1 of this spec.

## The problem

Work runs across multiple machines — auto-spawned by jig on the remote dev server,
hand-created worktrees locally — and there is no single place to answer:

1. What issues are in the queue?
2. What is the current review state?
3. Which agents are *genuinely* stuck?
4. How do I switch the model a task is running on?

herdr solves the *terminal* half of this (persistent PTYs, attach from anywhere,
`herdr --remote` as a client over remote sessions). It cannot solve the *work* half:
herdr knows a pane went idle; it does not know whether that means "done and merged,"
"done but no PR," "CI red," or "waiting on review."

## Identity

**jig is the delivery loop — issue → worktree → agent → PR → merge — and its control
plane.** Not a multiplexer, not a terminal UI.

- **Mux-agnostic by design.** The `Mux` trait (`crates/jig-core/src/mux/`) has two
  first-class backends chosen per device in config: `tmux` (ubiquitous, fine for
  throwaway boxes) and `herdr` (persistent PTYs, remote attach, live
  `agent_state: idle|working|blocked|done|unknown`). Capabilities degrade honestly:
  `agent_state() == None` on tmux means the monitor judges from the event log alone —
  today's behavior. Neither backend is deprecated.
- **Terminal viewing/attach is delegated**, always: `jig attach` locally,
  `herdr --remote` across machines. jig builds no terminal UI, ever.
- **The fleet view is a *work* view**: branches, pipeline state, PR/CI/review health,
  issue linkage, stuck-ness — joined across every machine.

## Why the distributed problem is small

The four questions decompose by data source:

| Question | Source of truth | Distribution needed |
|----------|----------------|---------------------|
| Issue queue | Linear (cloud) | none — hub polls it (jig already does: `issues`, triage) |
| Review state | GitHub (cloud) | none — hub polls it (jig already does: `daemon/checks.rs`) |
| Genuinely stuck | per-machine: event log + mux `agent_state` | **state up**: small worker snapshots |
| Switch model / kill / nudge | the machine owning the worker | **commands down**: routed to that machine |

Only rows 3–4 cross machines. The sync fabric is worker snapshots flowing up and
commands flowing down — a handful of typed message kinds, not state replication.
`WorkerSnapshot` already exists in [daemon-service-plan.md](./daemon-service-plan.md)
as the IPC projection; the network frames are the same types plus a machine identity.

## Topology

```
            Linear ──┐            ┌── GitHub
                     ▼            ▼
               ┌──────────────────────┐
               │  jig hub             │  persistent deployment (dev server)
               │  = jig daemon +      │  · polls Linear/GitHub (as today)
               │    aggregator role   │  · holds fleet state: machine → workers
               │                      │  · routes commands to owning daemon
               └──▲───────▲───────▲───┘
        snapshots │       │       │ commands
               ┌──┴──┐ ┌──┴──┐ ┌──┴──┐
               │ dev │ │ lap │ │ box │   jig daemons (one per machine)
               │ srv │ │ top │ │  N  │   · local monitor loop, unchanged
               └──┬──┘ └──┬──┘ └──┬──┘   · forward snapshots, execute commands
                  │       │       │
               tmux/herdr per device      terminals: herdr --remote for eyes-on
```

- **hub** = the same jig daemon binary with an aggregator role enabled, running as a
  user service on the always-on dev server. Machines connect *outward* to it (laptop
  behind NAT never needs to be reachable).
- **daemon** = [daemon-service-plan.md](./daemon-service-plan.md) as written, plus a
  forwarder: publish `WorkerSnapshot`s on change (and heartbeat), subscribe for
  commands addressed to this machine.
- **clients** = thin: `jig ps --fleet` (any machine) asks the hub; `jig model`,
  `jig kill --machine`, etc. submit commands to the hub for routing. Local commands
  keep talking to the local daemon socket directly.

## Transport: typed frames, pluggable carrier, iroh for the network

Design the protocol once, transport-agnostic: the same serde frames
(`Snapshot`, `Heartbeat`, `Command`, `Ack`) over:

1. **Local**: the unix socket from daemon-service-plan (newline-delimited JSON).
2. **Network**: [iroh](https://iroh.computer) — QUIC with hole-punching + relays,
   node identity = keypairs. Chosen because:
   - The laptop roams and NATs; iroh needs no port-forwarding and no reachable hub IP.
   - The command channel is remote code execution by construction; iroh's keypair
     identity gives a natural allowlist ("my machines"), e2e encrypted by default.
   - zim (krondor-corp/zim, built on iroh-blobs) has working patterns to reuse:
     `zim-peer`'s accept policy, peer registry, and wire-protocol shape.

   Alternatives considered and rejected:
   - *HTTPS + tokens to a reachable hub*: fails the roaming-laptop case for
     downstream commands (polling) and makes the hub a public endpoint.
   - *Tailscale/WireGuard mesh + plain HTTP*: only moves the connectivity problem —
     the command channel still needs app-level identity for the allowlist (trusting
     the network is not acceptable for remote execution), so the keypair machinery
     gets built regardless, plus a third-party control plane and a login on every
     box. With iroh the transport and the auth model are the same keypair, and the
     operational patterns (endpoint setup, accept policy, peer registry, relays) are
     already solved in `zim-peer`.

   iroh is the settled choice. It still lives in one transport crate behind the
   frame types — that's separation of concerns, not a planned escape hatch.

Security invariant: the hub and daemons accept frames **only** from allowlisted node
keys (`jig fleet trust <node-id>` to enroll a machine). A command frame is refused,
not queued, for unknown keys. No open enrollment, ever.

## Model switching

`jig model <branch> <model> [--machine <name>]`:

- **v0 (local, ships with Phase 2):** the agent session is live in a mux window —
  send `/model <model>` through `Mux::send_message`. claude applies it in-session.
  Record the change as an event so `ps` shows the current model.
- **v1 (routed):** same command from any machine; hub routes to the owning daemon.
- **v2 (cold switch):** for a worker that must restart (agent doesn't support live
  switch, or session wedged): daemon kills the window and re-runs `Agent::resume`
  with the new `--model` flag. Requires persisting model + session id on the worker
  record (`Agent::from_config` already takes the model; resume already re-launches
  with context).

Same routing carries the rest of the control verbs: `kill`, `nudge`, `spawn --machine`.

## The fleet view

`jig ps --fleet` (later `-g` grows a `MACHINE` column): one table joining
hub-polled Linear/GitHub state with per-machine snapshots —
`MACHINE · WORKER · STATE · AGENT (live mux state) · MODEL · PR · CI · REVIEW · ISSUE`.
"Genuinely stuck" is a first-class filter: pipeline says working, agent state
idle/blocked past threshold, no commits — the existing nudge-ladder heuristics, now
visible in one place instead of per-box.

Stretch, explicitly after everything else: the hub serves a read-only web view of the
same table (axum, the zim `daemon/api` pattern). It renders work state only — attach
links can deep-link to `herdr --remote` targets, but terminals never render in jig.

## Phases

1. **Daemon as a service** — execute
   [daemon-service-plan.md](./daemon-service-plan.md) (unix-socket IPC,
   `WorkerSnapshot`, `jig daemon start/stop/status/install/uninstall`,
   `service-manager`, user-level launchd/systemd). Plus, landed prerequisites hardened:
   `mux` selection in `jig.toml` (env `JIG_MUX` as override), per-tick mux snapshot
   batching (herdr's CLI is 3 calls/worker/tick today — too chatty for always-on),
   `jig health` herdr detection.
2. **Local control verbs + state fusion** — `jig model` v0; monitor uses
   `agent_state` where present: never nudge `blocked` (notify: "needs approval —
   `jig attach X`"), suppress idle-nudges while `working`, unchanged when `None`.
3. **Fleet fabric** — transport crate (frames over unix socket first, iroh second,
   `zim-peer` as reference), hub role flag, `jig fleet trust`, snapshot forwarding,
   command routing, `jig ps --fleet`.
4. **Queue across machines** — auto-spawn assignment: hub assigns queued issues to
   machines by config (max workers per machine already exists as
   `max_concurrent_workers`); `jig spawn --machine <name>` routed like any command.
5. **Web view** (stretch) — read-only axum table on the hub.

## Risks

| Risk | Mitigation |
|------|------------|
| Scope: this is a small distributed system | The protocol is 4 frame kinds; hub is the same binary/role-flag; phases 1–2 are useful standalone if 3 stalls |
| Command channel = remote execution | Keypair allowlist only, explicit `fleet trust`, refuse unknown keys, no auto-enrollment |
| iroh operational surface (relays, keys) | Patterns already proven in zim-peer; isolated in one transport crate behind the frame types |
| Stale snapshots misleading the fleet view | Heartbeats + `last_seen` rendered honestly (dim stale machines); hub never fabricates liveness |
| herdr pre-1.0 CLI drift (backend dep) | Typed fail-loud parsing, version guard, tmux backend fully supported forever under this identity |
| Two views drift (local `ps` vs. fleet) | Both render from the same `WorkerSnapshot` type; fleet adds columns, never diverges in derivation |

## Non-goals

- A terminal UI or terminal fleet view (herdr's product; `herdr --remote` covers it).
- Making herdr (or tmux) the sole backend — per-device choice is the point.
- Writing mux `agent_state` into the event log (it's a sensor; the log stays the
  pipeline source of truth).
- General-purpose task queue for non-jig work.
- Multi-user fleets — this is one operator's machines (one keyring). Revisit if that
  changes.
