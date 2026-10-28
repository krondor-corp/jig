---
description: Run jig end to end — the hermetic daemon tests, or an interactive fleet of isolated local daemons (web view, workers, static net mesh). Use to verify daemon, web view, or fleet changes in a real process, not just unit tests.
allowed-tools:
  - Bash(make e2e)
  - Bash(./bin/dev)
  - Bash(./bin/dev *)
  - Bash(cargo test *)
  - Bash(curl *)
  - Read
  - Grep
  - Glob
---

Read `docs/LOCAL-DEV.md` first.

## Verdict: hermetic

Run `make e2e`. It starts real daemons in temp sandboxes on free ports and
can run alongside other worktrees. For one area, run
`cargo test -p jig-cli --test daemon <filter>`, e.g. `web`.

## Inspection: the dev fleet

1. Run `./bin/dev ports` and use the ports it prints. Never hardcode them,
   because other worktrees may be running fleets of their own.
2. Run `./bin/dev up`, or `up --net` for the `[fleet.static]` mesh.
   - On macOS, `--net` needs `./bin/dev net-setup`. That uses sudo, so ask
     the user to run it.
3. Drive the nodes with `./bin/dev jig <node> <args>`, and read their state
   with `./bin/dev status` and `./bin/dev logs <node>`.
4. To check the web view, `curl http://127.0.0.1:<web port>/`. A wrong
   `Host` header must get 403, and a `POST /command` needs the
   `jig-token` meta from the page.
5. Finish with `./bin/dev down`.

Don't run `jig spawn` without the user's go-ahead. Workers run the real
`claude` under the user's login and cost real tokens.
