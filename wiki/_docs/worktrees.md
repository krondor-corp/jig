---
title: Worktrees
slug: worktrees
date: 2025-05-09
---

Create isolated worktrees for parallel development — each gets its own branch and working directory.

## Create

```bash
jig create feature-auth        # Create worktree
jig create feature-auth -o     # Create and cd into it
```

The `-o` flag can be placed anywhere:

```bash
jig -o create feature-auth
jig create -o feature-auth
```

## List

```bash
jig list                       # Worktrees in current repo
jig list -g                    # Global — all repos
jig list --all                 # Include the base repo
```

## Open

```bash
jig open feature-auth          # cd into an existing worktree
```

## Remove

```bash
jig remove feature-auth        # Remove exact match
jig remove 'test*'             # Glob pattern
jig remove 'feature/*'         # Remove all under feature/
jig remove feature-auth -f     # Force remove
```

## Exit

```bash
jig exit                       # Remove current worktree and cd to repo root
```

## Home

```bash
jig home                       # cd to base repository root
```

## Nested paths

Branch names with slashes create nested directories:

```bash
jig create feature/auth/oauth -o
# Creates .jig/feature/auth/oauth/
```

## Branch resolution

`jig create <branch>` resolves the starting commit in this order:

1. **Local branch exists** — checks it out as-is.
2. **`origin/<branch>` exists** — creates a local branch from the remote and tracks it. The configured base is ignored. This is the common case when an agent already pushed the branch (e.g. via `jig spawn`).
3. **Neither** — forks a new branch from the configured base (`[worktree] base` in `jig.toml`, or `--base`).

So `jig create feat/xyz` will automatically check out a daemon-created branch if it exists on origin, rather than creating a disconnected fork.

```bash
# Agent pushed feat/xyz — you get the real branch, not a fork
jig create feat/xyz -o

# Branch doesn't exist anywhere — forks from origin/main (default base)
jig create my-new-feature -o

# Override base for a genuinely new branch
jig create my-feature --base origin/develop -o
```

## How it works

Worktrees are stored in `.jig/` inside your repo (auto-added to `.git/info/exclude`):

```text
my-repo/
├── .jig/
│   ├── feature-a/
│   ├── feature-b/
│   └── feature/
│       └── auth/
│           └── oauth/
├── src/
└── ...
```

Each worktree is a full checkout of your repo on its own branch. Changes in one worktree don't affect others until you merge.

## Parallel sessions

Each worktree is independent. Run multiple agent sessions side by side:

Terminal 1:
```bash
jig create feature-auth -o
your-agent
```

Terminal 2:
```bash
jig create fix-bug-123 -o
your-agent
```

Both work independently with their own branches. For managed parallel sessions with monitoring and daemon supervision, use `jig spawn` instead — see [Spawning](/docs/spawning/).
