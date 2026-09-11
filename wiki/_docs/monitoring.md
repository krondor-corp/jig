---
title: Monitoring
slug: monitoring
date: 2025-05-09
---

Monitor all your workers from the terminal with `jig ps`. A single background daemon (`jig daemon start`) does the supervising — monitoring workers, nudging stuck agents, tracking PR health — and `jig ps -gw` is the live dashboard onto it.

With no daemon running, `jig ps` still works: it drives one in-process, exactly as it always did.

## The dashboard

```bash
jig ps -gw
```

This starts the global live watch display — all workers across all repos, updating in real time:

```text
jig ps --watch — 4 workers  (every 2s · daemon pid 72706)

WORKER              STATE    COMMITS  PR     HEALTH  ISSUE
● jwt-auth          running        2  -      -       ENG-123
● pagination        running        0  -      -       ENG-124
● test-coverage     draft          3  #42    ci      ENG-125
● error-pages       review         5  #43    ok      ENG-126

                                              [l]ogs  [q]uit
```

The header says where the frames come from: `daemon pid N` when a daemon is
answering, `hosting the daemon` when this view started one because none was
running.

### Columns

| Column | Description |
|--------|-------------|
| **WORKER** | Name with a mux status dot: `●` running, `○` exited, `✗` not found — colored red/green/yellow on herdr when it also reports the agent as blocked/working/idle |
| **STATE** | Derived worker status from the event stream |
| **COMMITS** | Commits ahead of base branch (`*` = uncommitted changes) |
| **PR** | PR number if one exists |
| **HEALTH** | PR check results: `ok`, problem names in red, `-` if no PR |
| **ISSUE** | Linked issue reference |

### At a glance

- **Which agents are active** — `running` means tool use is flowing
- **Who's stuck** — `stalled` means silence for 5+ minutes, the daemon will nudge
- **Draft vs review** — `draft` means agent is still working; `review` means ready for human review
- **PR health** — `ci` means checks failing, `conflicts` means merge conflicts
- **Progress** — Commit count shows how far along each worker is

### Log view

Press `l` in watch mode to see daemon activity:

```text
[14:32:05] tick: 3 workers, 1 action, 1 nudge, 0 errors
[14:32:05]   myrepo/jwt-auth PR: ok
[14:32:05]   myrepo/test-coverage PR: ci, conflicts
[14:32:35] tick: 3 workers, 0 actions, 0 nudges, 0 errors
```

Press `t` or `l` again to switch back. Press `q` to quit.

## The daemon

Every 30 seconds, the daemon fetches repos, scans event logs to derive worker state, discovers PRs via GitHub, and dispatches actions (nudges, notifications, cleanup).

The daemon uses background actor threads for blocking I/O: syncing repos, querying GitHub, polling for spawnable issues, creating worktrees, pruning merged workers, and delivering nudges through the configured mux backend.

### Running it

```bash
jig daemon start     # run it in the foreground (ctrl-c to stop)
jig daemon stop      # ask the running one to shut down
jig daemon status    # is it alive, ticking, and unstuck?
```

There is **one daemon per user**, always global — it watches every tracked
repo. A second `jig daemon start` fails with `daemon already running (pid N)`
rather than starting a rival that would fight over the same worktrees.

It binds a unix socket at `$XDG_RUNTIME_DIR/jig/daemon.sock` (falling back to
`~/.config/jig/state/daemon.sock`) and claims `daemon.pid` beside it. Both are
removed on a clean exit; after a crash the next start finds them stale and
takes them over, so there is nothing to clean up by hand.

`jig ps` and `jig ps -gw` are clients of that socket. They render what the
daemon reports instead of running a tick loop of their own, so you can have as
many dashboards open as you like. When no daemon is listening, `jig ps -gw`
starts one inline for the life of the view (and, in global mode, takes the
socket so `jig daemon status` can see it) — which is how jig worked before the
daemon had a socket.

### Checking on the daemon

`jig daemon status` asks the daemon over the socket — an answer *is* the proof
of life. Useful when workers seem to have stopped being nudged or spawned:

```text
✓ daemon running  pid 72706 · up 3h12m · v0.5.2
  → last tick 1s ago (every 2s)
  → log ~/.config/jig/state/logs/20260910T195104Z-daemon.log
  → jig-monitor  last finished 1s ago
  → jig-sync     last finished 1m40s ago
  → jig-spawn    busy 45s (last finished 2m ago)
```

It reports one of:

- **running** — answering and ticking on schedule
- **stalled** — answering, but the tick loop has stopped (the listener runs on
  its own thread, so a wedged tick still gets a reply)
- **not running** — nothing is listening on the socket

It also flags any actor that has been busy for over 10 minutes (e.g. a `git fetch` hung on auth), since the tick loop keeps running while a wedged actor silently skips its work. The command exits non-zero unless the daemon is healthy.

`jig daemon logs` prints the daemon's own log; `-f` follows it (and moves to the new log when the daemon restarts), `-n` sets how many lines, `--path` prints the file path.

## Nudges

When agents get stuck, the daemon intervenes by sending keystrokes through the mux backend (tmux or herdr). Each nudge type has an independent counter and escalates after `max_nudges` (default 3) to a notification instead.

### Nudge types

| Type | Trigger | Action |
|------|---------|--------|
| **idle** | Worker stalled or idle, no PR | Asks for status update, pushes toward committing |
| **stuck** | Worker waiting (interactive prompt) | Sends auto-approve keystroke, then message |
| **ci** | CI failing on open PR | Lists failing checks, tells agent to fix |
| **conflict** | Merge conflicts on PR | Tells agent to rebase and resolve |
| **review** | Unresolved review comments | Tells agent to address feedback |
| **bad-commits** | Non-conventional commits | Lists bad commits, tells agent to reword |

### Escalation

After `max_nudges` of the same type, the daemon stops nudging and fires a notification — alerting you that the worker needs human attention. This prevents infinite loops where an agent keeps failing at the same thing.

### Draft-only nudging

Nudges only fire for **draft PRs**. Once a PR is marked ready for review, the daemon backs off — the human is in control. Health problems still appear in the HEALTH column for visibility.

## Auto-cleanup

When a PR is merged or closed, the daemon automatically:

- Kills the worker's window (tmux window or herdr tab)
- Removes the worktree and event logs
- Emits a `Terminal` event

Pruning skips worktrees with uncommitted changes (logs a warning). On startup, the daemon scans for PRs merged/closed while it was offline and prunes stale workers.

Configure cleanup behavior:

```toml
[github]
auto_cleanup_merged = true       # default: kill workers when PR merges
auto_cleanup_closed = false      # kill workers when PR closed without merge
```

## Configuration

### Health thresholds

```toml
# ~/.config/jig/config.toml (global) or jig.toml (per-repo)

[health]
silence_threshold_seconds = 300  # 5 minutes before "stalled"
max_nudges = 3                   # per nudge type before escalation
```

### Per-type nudge config

```toml
# jig.toml

[health.nudge.idle]
max = 5
cooldown_seconds = 600

[health.nudge.ci]
max = 2
cooldown_seconds = 180
```

Available types: `idle`, `stuck`, `ci`, `conflict`, `review`, `bad_commits`. Resolution: per-type repo → repo defaults → global → hardcoded defaults.

### Custom nudge templates

Override any built-in template by placing files in `.jig/templates/`:

```text
.jig/templates/
├── nudge-idle.hbs
├── nudge-ci.hbs
└── spawn-preamble.hbs
```

Templates use Handlebars and always receive `nudge_count`, `max_nudges`, and `is_final_nudge`.

## The event system

Every worker has a JSONL event log at `~/.config/jig/state/events/<repo>-<worker>/events.jsonl`. Events are appended by git hooks and the daemon.

| Event | Source | Meaning |
|-------|--------|---------|
| `Spawn` | `jig spawn` | Worker created |
| `ToolUseStart` / `ToolUseEnd` | Agent hooks | Tool use activity |
| `Commit` | post-commit hook | Code committed |
| `Push` | post-commit hook | Code pushed |
| `PrOpened` | Daemon | PR discovered for branch |
| `Notification` | Agent | Hit an interactive prompt |
| `Stop` | Agent exit | Session ended |
| `Nudge` | Daemon | Nudge delivered |
| `Terminal` | Daemon | Worker cleaned up |

Worker state is derived by replaying the event stream — there's no mutable state database.

## Quick reference

```bash
jig ps                   # Status snapshot
jig ps -w                # Watch mode (current repo)
jig ps -gw               # Global watch — all repos, live dashboard
jig daemon start         # Run the daemon (one per user, watches every repo)
jig daemon stop          # Stop the running daemon
jig daemon status        # Is the daemon alive and ticking?
jig daemon logs -f       # Follow the daemon's log
```
