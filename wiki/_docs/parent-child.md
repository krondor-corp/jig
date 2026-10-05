---
title: Parent-Child Issues
slug: parent-child
date: 2025-05-09
---

Parent issues are for grouping. jig gives them no special lifecycle: no integration branch, no status changes, and no gate on their children.

## How the daemon treats them

- **Parents are never spawned.** An issue with at least one sub-issue is skipped by auto-spawn.
- **Children spawn like any other issue.** A child spawns once it's Planned, matches `auto_spawn_labels`, passes the provider's filters, and has no open `blocked-by` dependencies. The parent's status, assignee and branch don't matter.
- **Children branch from the repo's base branch** and `jig pr` targets that base, the same as any standalone issue.
- **The parent shows up as context.** A child worker's prompt includes the parent issue's title and description.

The parent's status is yours to manage in Linear.

## Ordering children

Use `--blocked-by` when one child needs another's work merged first:

```bash
jig issues create "Auth system overhaul" -p high
jig issues create "Add JWT token generation" --parent AUTH-1
jig issues create "Add auth middleware" --parent AUTH-1 --blocked-by AUTH-2
```

`AUTH-3` waits until `AUTH-2` is Complete, then branches from base and picks up the merged change.

## Requirements

- **Linear provider.** Parent/child relations come from Linear's API.
