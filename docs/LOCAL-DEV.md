# Local Dev and E2E

Two ways to run real jig daemons on one machine without touching your own
jig: a hermetic test suite with a pass/fail verdict, and an interactive
fleet you can drive by hand.

## Hermetic: `make e2e`

The integration tests in `crates/jig-cli/tests/{daemon,cli}` start real
`jig daemon` processes through `common::Sandbox`: temp `XDG_CONFIG_HOME`
and `XDG_RUNTIME_DIR`, temp git repos, free ports. They run in parallel
and in CI. `tests/daemon/fleet.rs` meshes two daemons over loopback and
speaks the peer protocol to them the way a peer would.

New daemon features get a test here first. Bind port 0 and ask the daemon
where it landed, never a fixed port.

To see a failed setup without starting an agent, give a node a
failing `on_create` in its scratch repo's `jig.toml`, then spawn into it.
Setup fails before `claude` would launch.

## Interactive: `bin/dev`

```bash
./bin/dev up                         # build; start alice + bob
./bin/dev jig alice spawn hello -c "say hi, then stop"
./bin/dev jig alice                  # alice's TUI
./bin/dev attach alice               # alice's tmux server (her workers)
./bin/dev status                     # daemon status + ps, every node
./bin/dev down                       # stop daemons and tmux servers
./bin/dev clean                      # ...and delete .dev/
```

Each node lives under `.dev/<node>/` with everything jig reads or writes:

| What | Where | How |
|------|-------|-----|
| config, state, logs, repo registry | `config/jig/` | `XDG_CONFIG_HOME` |
| daemon socket, pid file | `run/jig/` | `XDG_RUNTIME_DIR` |
| tmux server (workers) | `tmux/` | `TMUX_TMPDIR`, `JIG_MUX=tmux`, `TMUX` unset |
| agent hooks from `jig init` | `home/.claude/` | `HOME` for `init` only |
| a repo to spawn into | `repo/` | |

Workers run the real `claude` under your real `HOME`, so they use your
login and cost real tokens. Their hooks report to the node, because the
node's tmux server carries its `XDG_CONFIG_HOME`.

`config.toml` is regenerated on every `up`, and `up` restarts running
daemons so they pick up the new config and build.

**Nodes and ports** come from `confit.toml` ([confit](https://confit.krondor.org)
is required for `bin/dev`). confit gives each worktree a slot: 0 on `main`,
otherwise the lowest free 1-9. Each port is band + lane + slot, so several
worktrees can each run a fleet at once without collisions; `./bin/dev
ports` prints this worktree's. To add a node, give it a net lane under
`[ports.services]`, then a line in `[dev.nodes]`.

Make targets: `make dev` (up), `make dev-kill` (stop daemons and tmux
servers), `make dev-clean` (also delete `.dev/`), `make rust-clean`
(`cargo clean`), `make clean` (both).

### Fleet networking: `up --net`

`--net` adds a `[fleet.static]` mesh (see `docs/fleet-spec.md`): each node
listens on its own loopback address, with the other nodes as peers.

```toml
[fleet]
net = "static"

[fleet.static]
listen = "127.0.0.11:<port>"
peers = [{ name = "bob", addr = "127.0.0.12:<port>" }]
```

(plus `net_commands = true` under `[fleet]`)

With the mesh up, `./bin/dev jig alice ps --fleet` prints every node's
workers. `--net` sets `net_commands = true` on every node, so any node can
run verbs on another; a real machine leaves it off unless it wants peers to
drive it. A node lists issues only for repos with `[issues.linear]` in
their `jig.toml`, through a Linear profile in that node's own config, so
the scratch repos have none until you give a node both.

The static backend identifies a peer by source IP alone, so nodes on one
machine need distinct addresses. Linux routes all of `127/8`. macOS only
has `127.0.0.1`: `./bin/dev net-setup` adds the `lo0` aliases (sudo,
until reboot). Builds without the `Net` trait ignore these keys.
`./bin/dev health` shows each node's net checks.

Tailscale (`net = "tailscale"`, `jig fleet join`) needs a real tailnet
and isn't covered by either harness.
