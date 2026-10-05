---
description: Discover and manage work items. Use to explore tasks before spawning workers or to track project progress.
allowed-tools:
  - Bash(jig:*)
  - Read
  - Glob
  - Grep
---

Discover and manage work items via the `jig issues` CLI.

## Actions

### List issues

```bash
jig issues
jig issues --status planned
jig issues --priority high
jig issues --category Backend
jig issues --label backend
```

### Show issue

```bash
jig issues <id>
```

### Create issue

```bash
jig issues create "Add verbose flag"
jig issues create "Fix crash on exit" --priority high --category bugs
jig issues create "Refactor auth" --label backend
```

### Create sub-issue (for epics)

```bash
jig issues create "Add JWT generation" --parent ENG-123 --priority high
jig issues create "Add auth middleware" --parent ENG-123 --blocked-by ENG-124
```

Parents are for grouping only. Sub-issues branch from and PR into the repo's base branch like any other issue; use `--blocked-by` to order them.

### Update issue

```bash
jig issues update <id> --body "## Findings..." --append
jig issues update <id> --parent ENG-123
jig issues update <id> --remove-parent
```

### Update status

```bash
jig issues status <id> backlog
jig issues status <id> planned
```

### Complete issue

```bash
jig issues complete <id>
```

### Manage dependencies

```bash
# Add a blocker
jig issues update <id> --blocked-by <blocker-id>

# Add multiple blockers at once
jig issues update <id> --blocked-by dep-a,dep-b

# Remove a blocker
jig issues update <id> --remove-blocked-by <blocker-id>
```

Dependencies control spawn order — a child won't spawn until all its blockers are Complete.
