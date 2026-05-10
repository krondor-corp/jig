# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.3.0 (2026-05-10)

### Chore

 - <csr-id-71fe92a04f995aa0878eca05fbab642f17f2695f/> bump version to 0.3.0
 - <csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/> bump version to 0.2.0

### New Features

 - <csr-id-5cff652c9698daa6a067086234c50c6abd884517/> jig — git worktree manager for parallel Claude Code sessions

### Bug Fixes

 - <csr-id-cc11b783045bb5d4d897e5e7660557ced9c41cc5/> resolve clippy warnings in issues list and format pr create
 - <csr-id-9e3deddc1a6070d619d13874f63a6343b7e6ab02/> pass --head flag to gh pr create for git worktree compatibility
 - <csr-id-ba3ee31ba372af56cf1ed953128792b433deaf8c/> resolve clippy warnings for CI

### Style

 - <csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/> cargo fmt

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump version to 0.3.0 ([`71fe92a`](https://github.com/krondor-corp/jig/commit/71fe92a04f995aa0878eca05fbab642f17f2695f))
    - Merge pull request #7 from krondor-corp/feature/kro-137-test-verify-daemon-spawn-for-jig ([`fd5f56c`](https://github.com/krondor-corp/jig/commit/fd5f56cf36f050394e49120842eaca2527c47c7c))
    - Resolve clippy warnings in issues list and format pr create ([`cc11b78`](https://github.com/krondor-corp/jig/commit/cc11b783045bb5d4d897e5e7660557ced9c41cc5))
    - Pass --head flag to gh pr create for git worktree compatibility ([`9e3dedd`](https://github.com/krondor-corp/jig/commit/9e3deddc1a6070d619d13874f63a6343b7e6ab02))
    - Resolve clippy warnings for CI ([`ba3ee31`](https://github.com/krondor-corp/jig/commit/ba3ee31ba372af56cf1ed953128792b433deaf8c))
    - Merge pull request #2 from krondor-corp/release-automation ([`80abbce`](https://github.com/krondor-corp/jig/commit/80abbcee7ebe9c83aa89b557859ea7ae36fa9b31))
    - Bump jig-core v0.2.0, jig-cli v0.2.0 ([`678f85a`](https://github.com/krondor-corp/jig/commit/678f85ac899e871014b67a93ec4b26fe33693465))
    - Bump version to 0.2.0 ([`9f12c30`](https://github.com/krondor-corp/jig/commit/9f12c306cfcfb1eea60707d32360eb89479993a9))
    - Cargo fmt ([`54d326b`](https://github.com/krondor-corp/jig/commit/54d326b275eebbc3ff783d8ecbabb0b2bb28a3db))
    - Jig — git worktree manager for parallel Claude Code sessions ([`5cff652`](https://github.com/krondor-corp/jig/commit/5cff652c9698daa6a067086234c50c6abd884517))
</details>

## v0.2.0 (2026-05-10)

<csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/>
<csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/>

### Chore

 - <csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/> bump version to 0.2.0

### New Features

 - <csr-id-5cff652c9698daa6a067086234c50c6abd884517/> jig — git worktree manager for parallel Claude Code sessions

### Style

 - <csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/> cargo fmt

## v0.5.0 (2026-02-13)

<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>

### Chore

 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### New Features

 - <csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/> add worktree.copy for gitignored files
   Adds `worktree.copy` config to copy gitignored files (like .env)
   to new worktrees:
   
   ```toml
   [worktree]
   copy = [".env", ".env.local"]
   ```
   
   Files are copied after worktree creation, before on_create hook runs.
 - <csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/> add worktree config to jig.toml
   jig.toml now supports worktree configuration:
   - `worktree.base` — base branch for new worktrees (overrides global)
   - `worktree.on_create` — command to run after worktree creation
 - <csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/> improve adapter architecture and audit templates
   Adapter improvements:
   - Add AgentType enum for compile-time safe matching
   - Rename template to PROJECT.md (agent-agnostic name)
   - Dynamic audit prompt uses adapter.project_file and adapter.skills_dir
   - Validate agent is installed before init (warns if not in PATH)
   - Fix settings.json schema URL
   
   Template improvements:
   - Fix settings.json to use correct schemastore.org URL
   - Add WebFetch, WebSearch, mcp__*, jig:* to default permissions
   - Update review skill to check jig-specific docs and skills
   - Update issues skill to reference issues/README.md
 - <csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/> add agent-agnostic adapter architecture
   - Add adapter module with AgentAdapter struct for pluggable agent support
   - jig init now requires agent argument: `jig init claude`
   - jig.toml stores agent type in [agent] section
   - spawn command uses adapter to build agent-specific commands
   - Move settings.json to templates/adapters/claude-code/
   
   This architecture allows future support for other agents (cursor, etc.)
   by adding new adapter constants.
 - <csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/> rewrite health check to validate repo setup and agent scaffolding
   Replace terminal-detection-focused health check with structured validation
   of system deps (git, tmux, claude), repository config (jig.toml, base
   branch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).
   Remove unused jq/gh dependency checks and dead required field. Exit
   non-zero when checks fail.

## v0.4.0 (2026-02-12)

### New Features

 - <csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/> rewrite health check to validate repo setup and agent scaffolding
   Replace terminal-detection-focused health check with structured validation
   of system deps (git, tmux, claude), repository config (jig.toml, base
   branch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).
   Remove unused jq/gh dependency checks and dead required field. Exit
   non-zero when checks fail.

