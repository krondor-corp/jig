# herdr spec — going all in

Status: accepted (2026-08-31). Decision record + execution plan. The backend itself is
shipped; this spec covers making herdr the primary runtime and what gets built on it.
Companion doc: [daemon-service-plan.md](./daemon-service-plan.md) (detailed IPC design for
Phase 3).

## Decision

**herdr is jig's primary mux backend. tmux is demoted to a frozen compatibility
fallback.** All new orchestration features may assume herdr capabilities (live agent
state, persistent PTYs, remote attach); the tmux path keeps working for existing flows
but receives no new features and its consumers must tolerate `agent_state() == None`.

Why all-in rather than dual-first-class:

- **Agent state is the correct input to jig's core loop.** The monitor's job is deciding
  when a worker needs a nudge, an answer, or an escalation. On tmux it is guessing from
  hook events and `pane_current_command`; a worker stuck at a pre-hook dialog (trust
  prompt, settings warning, permission approval) looks idle and gets nudged into a
  dialog box. herdr classifies every pane `idle | working | blocked | done | unknown`
  via lifecycle hooks + screen-manifest detection. Validated live: herdr flagged a
  spawned worker `blocked` at claude's settings dialog — invisible to the tmux path.
- **The remote story only exists on herdr.** Daemon-owned PTYs survive disconnect,
  lid-close, and SSH drops; `herdr --remote <ssh-target>` reattaches from anywhere.
  tmux gives detach-survival on one machine and nothing else.
- **Dual first-class backends tax every feature forever** — every capability gated on
  `Option`, every test doubled. Freezing tmux caps that cost at zero.
- **The dependency shape doesn't change.** jig already requires an external binary
  (tmux). herdr is one curl install, Apache-2.0, with large adoption.

The hedge: herdr is pre-1.0 and we don't control it. The `Mux` trait
(`crates/jig-core/src/mux/mod.rs`) stays the seam — the herdr backend was built behind
it in an afternoon, and tmux stays compiled and selectable behind it. **tmux is deleted
only when both hold: herdr ≥ 1.0 with a stable CLI/socket contract, and one full
release cycle with no herdr-backend regressions.** Until then, all-in means "designed
for herdr", not "herdr-only".

## Current state (shipped)

- `HerdrMux` (`crates/jig-core/src/mux/herdr.rs`): full `Mux` impl over the `herdr`
  CLI. Mapping: jig group `jig-<repo>` → herdr **workspace** (by label); window/branch
  → **tab** (label verbatim); liveness via `pane process-info`; typed serde parsing of
  the CLI's JSON, fail-loud on shape changes.
- `Mux::agent_state(name) -> Option<AgentState>` — default `None`; herdr implements it
  from tab `agent_status`. `jig ps` renders it (dot: red=blocked, green=working,
  yellow=idle/done); monitor stores it on `WorkerState::mux_agent_state`.
- Backend selection via factory fns (`mux::for_repo` etc.) reading `JIG_MUX`; no call
  site names a concrete backend.
- Validated end-to-end on herdr 0.8.2: spawn, ps, monitor nudges delivered through
  herdr, kill, nuke, worktree removal. Fixed en route: one-shot `jig ps` raced the
  async monitor pass (now waits on `monitor.is_pending()`).
- CLI-only integration, deliberately: herdr's socket protocol is still migrating
  pre-1.0; the CLI is the stable contract. Flags we depend on: `workspace
  create/list/close`, `tab create/list/close/focus`, `pane
  list/get/process-info/send-text/send-keys/run`, `status`.

## Target architecture

```
laptop ──ssh / herdr --remote──▶ worker box
                                  ├─ herdr server  (owns PTYs: workspace per repo,
                                  │                 tab per worker branch)
                                  ├─ jig daemon    (user service: monitor/nudge/spawn/
                                  │                 triage; IPC socket, thin clients)
                                  └─ worktrees     (.jig/<branch>)
```

herdr owns terminals; jig owns orchestration (worktrees, lifecycle, event log, nudges,
GitHub/Linear, PRs). Both are OS user services; reboot recovery = launchd/systemd
restarts both, herdr restores layout, jig's orphan recovery (`daemon/mod.rs`) re-drives
agents whose processes died with the PTY owner.

**Invariant: jig's event log remains the source of truth for pipeline state**
(spawned → working → PR → done). `agent_state` is a live terminal observation — a
sensor fused in the monitor, never written into the event log as if it were a hook
event.

## Plan

### Phase 1 — flip the default

1. `mux = "herdr" | "tmux"` in `jig.toml` (global + per-repo override), **default
   `herdr`**; `JIG_MUX` env var stays as a debug override. Fallback rule: if herdr is
   not installed, warn once and use tmux — existing users keep working.
2. `jig init` / `jig health`: detect herdr binary + running server (`herdr status`),
   offer the install one-liner, fail health with a fix when `mux = "herdr"` and the
   server is down. Version guard: warn outside the tested range (`>= 0.8.2`).
3. **Snapshot batching**: today each `HerdrMux` query re-lists workspaces/tabs/panes
   (≈3 CLI calls per worker per tick). Add a per-tick `MuxSnapshot` (one `workspace
   list` + `tab list` + `pane list`) that all workers resolve against. Required before
   the always-on daemon.
4. UX sweep: spawn/kill/attach messages currently say "tmux window" — make them
   backend-aware; `jig attach` execs the herdr client with the right tab focused.

### Phase 2 — state fusion in the monitor

Monitor rules, in priority order:

1. **`blocked`** → never nudge (text lands in a dialog). Notify instead ("worker X
   needs approval — `jig attach X`"), pause the idle-nudge clock. Never auto-answer
   dialogs.
2. **`working`** → suppress idle nudges even when the event log is quiet (hooks lag
   long tool calls); reset the stuck-timer.
3. **`idle` / `done`** → corroborates the event log; existing nudge ladder applies.
4. **`None` / `unknown`** → today's behavior, unchanged (this is the whole tmux path).

`jig ps` gains a distinct `AGENT` column (live terminal state) alongside `STATE`
(pipeline state).

### Phase 3 — daemon becomes a service

Execute [daemon-service-plan.md](./daemon-service-plan.md) (unix-socket IPC,
`WorkerSnapshot`, `jig daemon start/stop/status/install/uninstall`, `service-manager`
crate, user-level launchd/systemd — same shape as zim's `zim daemon service`, see
krondor-corp/zim `crates/zim/src/cli/ops/daemon/`). Amendments now that herdr is
primary:

- Monitor uses Phase 1 snapshot batching; IPC `Status` includes `agent_state`.
- `jig daemon status` reports mux backend + herdr server health beside PID/uptime.
- `jig daemon install` offers to install herdr's service too (paired services on a
  worker box).

### Phase 4 — remote host story

Mostly documentation + polish once 1–3 land:

- Provision a box: install jig + herdr, `jig daemon service install`, herdr service
  install, clone repos, `jig init`.
- Drive from the laptop: `ssh box jig spawn ...`, `ssh box jig ps` (thin IPC client —
  instant), `herdr --remote box` for the full TUI.
- Stretch: `jig attach --remote <host> <branch>` wraps `herdr --remote` with the tab
  focused.

### Phase 5 — retire tmux

When the deletion criteria in **Decision** hold: remove `tmux.rs`, the fallback rule,
and the `Option` on `agent_state`. Until then tmux code is frozen — bugfixes only.

## Risks

| Risk | Mitigation |
|------|------------|
| herdr pre-1.0 CLI drift | Version guard; typed parsing fails loud; CLI-only (no socket coupling); tmux fallback stays compiled until 1.0 + one clean cycle |
| Upstream direction we don't control | `Mux` trait seam — worst case we fork or re-point the backend; Apache-2.0 permits both |
| Install friction for new users | Auto-detect + one-liner in `jig init`/`health`; silent tmux fallback keeps them unblocked |
| CLI latency in the tick loop | Snapshot batching (Phase 1.3); 10s per-command timeout already in `HerdrMux::run` |
| False `blocked` from screen detection | herdr matches known dialog UI only; fusion only suppresses nudges or notifies — never auto-answers |
| tmux muscle memory / user preference | tmux stays selectable via config through Phase 5; herdr supports tmux-style prefix keys |

## Testing

- Unit: key translation, JSON fixtures from real herdr 0.8.x output, snapshot
  resolution.
- Integration (gated on `herdr` in PATH; always a named test session per herdr's own
  guidance, never the user's default): window round-trip, `agent_state` transitions,
  monitor blocked-suppression.
- Compat gate: full suite green with no herdr installed (tmux fallback path).

## Non-goals

- Replacing jig's event log with herdr state (sensor, not source of truth).
- herdr socket API integration (revisit at herdr 1.0).
- Publishing a jig plugin to herdr's marketplace (tracked separately).
- Windows (herdr supports it; out of scope here).
