# Local Dev and E2E

Two ways to run real jig daemons on one machine without touching your own
jig: a hermetic test suite with a pass/fail verdict, and an interactive
fleet you can click through.

## Hermetic: `make e2e`

The integration tests in `crates/jig-cli/tests/{daemon,cli}` start real
`jig daemon` processes through `common::Sandbox`: temp `XDG_CONFIG_HOME`
and `XDG_RUNTIME_DIR`, temp git repos, free ports. They run in parallel
and in CI. `tests/daemon/web.rs` covers the web view end to end: `[fleet]
web` turns it on, `jig web` finds it, and its commands reach the daemon's
executor.

New daemon features get a test here first. Bind port 0 and ask the daemon
where it landed, never a fixed port.

## The web view

| Part | Where |
|------|-------|
| Wire types (`/api/v0` paths, request and response JSON) | `crates/jig-api` (serde only, so it compiles to wasm) |
| Server: axum on its own `jig-web` thread and tokio runtime | `crates/jig-cli/src/daemon/web/` |
| App: Yew, built by Trunk into `web/dist/`, embedded in jig | `crates/jig-cli/web/` (its own workspace) |

`make build-web` builds the app; it needs `trunk` and the `wasm32-unknown-unknown`
target. A plain `cargo build` without it still works, and the page then says
it isn't built. `./bin/dev up` builds both. A debug jig reads `web/dist/` from
disk on every request, so after `make build-web` you only need to reload the
page, not rebuild or restart jig.

The server's unit tests (`daemon/web/tests.rs`) drive the router in-process
with a stand-in daemon. They don't need the app built. Lint the app on its
own: `cd crates/jig-cli/web && cargo clippy --target wasm32-unknown-unknown`.

To see the failed-setups panel without starting an agent, give a node a
failing `on_create` in its scratch repo's `jig.toml`, then spawn into it.
Setup fails before `claude` would launch.

## Interactive: `bin/dev`

```bash
./bin/dev up                         # build; start alice + bob
./bin/dev jig alice spawn hello -c "say hi, then stop"
./bin/dev web alice                  # the web view
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
ports` prints this worktree's. To add a node, give it a web lane and a net
lane under `[ports.services]`, then a line in `[dev.nodes]` and in `[dev.net]`.

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

The static backend identifies a peer by source IP alone, so nodes on one
machine need distinct addresses. Linux routes all of `127/8`. macOS only
has `127.0.0.1`: `./bin/dev net-setup` adds the `lo0` aliases (sudo,
until reboot). Builds without the `Net` trait ignore these keys.
`./bin/dev health` shows each node's net checks.

Tailscale (`net = "tailscale"`, `jig fleet join`) needs a real tailnet
and isn't covered by either harness.
