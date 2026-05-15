# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.3.1 (2026-05-11)

<csr-id-e1dfd6b90ac3e80c6af9fd46fd1e19fd0207b3f5/>
<csr-id-6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02/>
<csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/>
<csr-id-6f958a8c78b08a0374f80d5d2ecd868e3bed8c68/>
<csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/>

### Chore

 - <csr-id-e1dfd6b90ac3e80c6af9fd46fd1e19fd0207b3f5/> bump version to 0.3.1
 - <csr-id-6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02/> bump version to 0.3.0
 - <csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/> bump version to 0.2.0

### New Features

 - <csr-id-5cff652c9698daa6a067086234c50c6abd884517/> jig — git worktree manager for parallel Claude Code sessions

### Bug Fixes

 - <csr-id-a6a59f1aad09365f570b64c283e11a0f4b281667/> query all PR states and add integration tests (KRO-143)
   get_pr_for_branch used state=open, making closed/merged PRs invisible
   and preventing cleanup. Changed to state=all and parse actual state from
   the response instead of hardcoding Open. Added integration tests against
   real closed PR #17 and merged PR #18 on krondor-corp/jig (gated behind
   #[ignore], run with cargo test -p jig-core -- --ignored).
 - <csr-id-50c054d030c0d8a42fa6f105c4771c606c94810d/> pass prompt via --allowed-tools= form and clear tracker after run
   Two related bugs in the triage path:
   
   1. `Agent::once` builds argv with `--allowed-tools <tools...>` (variadic),
   which swallows the prompt as another tool name. Claude exits with
   "Input must be provided either through stdin or as a prompt argument".
   Switch to the `--allowed-tools=value` form so the prompt remains a
   separate positional.
   
   2. `TriageActor::register` adds an entry to the tracker but no path
   removes it on success — only the stuck-timeout sweep (default 600s)
   clears completed triages. The `jig ps -gw` display surface keeps
   showing finished triages for up to 10 minutes. Remove the tracker
   entry immediately after the synchronous `run_single` returns.
 - <csr-id-cc11b783045bb5d4d897e5e7660557ced9c41cc5/> resolve clippy warnings in issues list and format pr create
 - <csr-id-9e3deddc1a6070d619d13874f63a6343b7e6ab02/> pass --head flag to gh pr create for git worktree compatibility
 - <csr-id-ba3ee31ba372af56cf1ed953128792b433deaf8c/> resolve clippy warnings for CI

### Style

 - <csr-id-6f958a8c78b08a0374f80d5d2ecd868e3bed8c68/> cargo fmt
 - <csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/> cargo fmt

## v0.3.0 (2026-05-10)

<csr-id-6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02/>
<csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/>
<csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/>

### Chore

 - <csr-id-6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02/> bump version to 0.3.0
 - <csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/> bump version to 0.2.0

### New Features

 - <csr-id-5cff652c9698daa6a067086234c50c6abd884517/> jig — git worktree manager for parallel Claude Code sessions

### Bug Fixes

 - <csr-id-50c054d030c0d8a42fa6f105c4771c606c94810d/> pass prompt via --allowed-tools= form and clear tracker after run
   Two related bugs in the triage path:
   
   1. `Agent::once` builds argv with `--allowed-tools <tools...>` (variadic),
   which swallows the prompt as another tool name. Claude exits with
   "Input must be provided either through stdin or as a prompt argument".
   Switch to the `--allowed-tools=value` form so the prompt remains a
   separate positional.
   
   2. `TriageActor::register` adds an entry to the tracker but no path
   removes it on success — only the stuck-timeout sweep (default 600s)
   clears completed triages. The `jig ps -gw` display surface keeps
   showing finished triages for up to 10 minutes. Remove the tracker
   entry immediately after the synchronous `run_single` returns.
 - <csr-id-cc11b783045bb5d4d897e5e7660557ced9c41cc5/> resolve clippy warnings in issues list and format pr create
 - <csr-id-9e3deddc1a6070d619d13874f63a6343b7e6ab02/> pass --head flag to gh pr create for git worktree compatibility
 - <csr-id-ba3ee31ba372af56cf1ed953128792b433deaf8c/> resolve clippy warnings for CI

### Style

 - <csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/> cargo fmt

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

## v0.4.0 (2026-05-15)

### Style

 - <csr-id-e7152ee81cd6d10e480735b1f263943db5a7bde9/> cargo fmt
 - <csr-id-6f958a8c78b08a0374f80d5d2ecd868e3bed8c68/> cargo fmt
 - <csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/> cargo fmt

### Refactor

 - <csr-id-0a996ced56110b176b3de348cbbd0f510b4873ac/> add generic Event<K> and consolidate event scaffolds
   Add `jig_core::Event<K>` (with `now()` constructor and flatten-roundtrip
   test) as the shared timestamped-event wrapper.  Add `ReducibleKind` trait
   with a blanket `impl<K: ReducibleKind> Reducible for Event<K>` so that
   type-aliased event types in jig-cli can satisfy `EventLog::reduce()` without
   violating Rust's orphan rules.
   
   Collapse the three independent concrete structs in jig-cli to type aliases:
   
     worker/events/schema.rs  →  pub type Event = jig_core::Event<EventKind>;
     daemon/events/schema.rs  →  pub type Event = jig_core::Event<EventKind>;
     notify/events.rs         →  pub type Notification = jig_core::Event<NotificationEvent>;
   
   Follow-on fixups:
   - Worker reducer: impl ReducibleKind for EventKind (receives ts + kind);
     replace five test-only Event::at(ts, kind) calls with Event { ts, kind }
     struct literals; drop the Event::event_type() / Event::at() wrappers.
   - Daemon reducer: impl ReducibleKind for EventKind; move started()/stopped()
     to free functions re-exported from events::mod; update two call sites in
     daemon/mod.rs.
   - Notify: drop the id UUID field and its generation in queue::emit(); remove
     Notification::to_json() wrapper, inline serde_json::to_string at the two
     call sites; add backward-compat roundtrip test for old JSONL lines with id.
   
   On-disk format is unchanged for worker/daemon.  Notify JSONL gains no id
   field on new writes; old lines with id deserialise cleanly (serde default).
 - <csr-id-2393dc403a09b4991400aa5bbeabd50bc96b95d3/> collapse GetIssue into ListIssues; document single source of truth
   Delete get_issue.rs and update the three call sites in the Linear client
   to use ListIssues { filter, first: 1 }.into_iter().next() instead.
   Add a canonical doc comment on ListIssues that enforces the one-query
   invariant for all issue field selection.

### Bug Fixes

 - <csr-id-28aca7958072029c21c92bd8782e6dc51a01b7fe/> check out remote branch when origin/<branch> already exists
   When `jig create <branch>` is called and `origin/<branch>` exists (e.g.
   a daemon spawned `feat/xyz` with a PR), the worktree now checks out that
   remote branch rather than forking a new branch from base. This means
   `jig create feat/xyz` gives you the real branch the daemon created, not a
   disconnected fork.
   
   Resolution order in add_worktree:
   1. Local branch exists → use it (unchanged)
   2. origin/<branch> exists → create local from remote, set upstream to origin/<branch>
   3. Neither → fork from base (unchanged)
   
   Also reverts the MissingRemote error variant added in the previous commit —
   the real fix is smarter resolution, not louder errors on missing remotes.
 - <csr-id-e732f6773fa0575835719ea68aeaac8219db33b8/> validate remote exists before creating worktree; drop HEAD fallback
   Add up-front remote validation in `jig create` so that a base branch like
   `origin/main` fails fast with a clear error message when the `origin` remote
   is not configured, rather than silently forking from HEAD.
   
   Also removes the silent HEAD fallback from `find_valid_start_point`: if
   neither the literal base branch nor its `origin/`-stripped variant resolves,
   the function now returns `BranchNotFound` instead of falling back to HEAD.
   This surfaces typos and missing remote refs that were previously invisible.
   
   New `Branch::remote_prefix()` returns the first `/`-delimited segment (or
   `None` for plain branch names). New `Repo::has_remote()` checks whether a
   named remote is configured.
 - <csr-id-595ecea0aeed045cb76d1ff5f0b93bdf0978ec24/> work around libgit2 ENOENT on missing .git/shallow
   libgit2 1.9.2 (via git2 0.20.4) surfaces a fatal "could not find
   .git/shallow to stat" error during worktree iteration on macOS even
   though file-absent is supposed to mean "not a shallow clone." Hit
   reproducibly via `jig rm` on non-shallow repos.
   
   We're already on the latest git2/libgit2-sys, so the fix is on our
   side: after Repo::open / Repo::discover, proactively create an empty
   .git/shallow in the common dir if missing. Per git's own semantics,
   an empty shallow file means "no shallow refs" — no behavior change
   for git itself, only avoids the libgit2 internal stat from blowing
   up downstream.
 - <csr-id-a6a59f1aad09365f570b64c283e11a0f4b281667/> query all PR states and add integration tests (KRO-143)
   get_pr_for_branch used state=open, making closed/merged PRs invisible
   and preventing cleanup. Changed to state=all and parse actual state from
   the response instead of hardcoding Open. Added integration tests against
   real closed PR #17 and merged PR #18 on krondor-corp/jig (gated behind
   #[ignore], run with cargo test -p jig-core -- --ignored).
 - <csr-id-50c054d030c0d8a42fa6f105c4771c606c94810d/> pass prompt via --allowed-tools= form and clear tracker after run
   Two related bugs in the triage path:
   
   1. `Agent::once` builds argv with `--allowed-tools <tools...>` (variadic),
      which swallows the prompt as another tool name. Claude exits with
      "Input must be provided either through stdin or as a prompt argument".
      Switch to the `--allowed-tools=value` form so the prompt remains a
      separate positional.
   
   2. `TriageActor::register` adds an entry to the tracker but no path
      removes it on success — only the stuck-timeout sweep (default 600s)
      clears completed triages. The `jig ps -gw` display surface keeps
      showing finished triages for up to 10 minutes. Remove the tracker
      entry immediately after the synchronous `run_single` returns.
 - <csr-id-cc11b783045bb5d4d897e5e7660557ced9c41cc5/> resolve clippy warnings in issues list and format pr create
 - <csr-id-9e3deddc1a6070d619d13874f63a6343b7e6ab02/> pass --head flag to gh pr create for git worktree compatibility
 - <csr-id-ba3ee31ba372af56cf1ed953128792b433deaf8c/> resolve clippy warnings for CI

### Documentation

 - <csr-id-a359f75431686c36f688a905ec9c353c50bf2929/> document branch resolution order; log when base is ignored
   Add tracing::info! in the remote-branch case of add_worktree so that the
   "ignoring configured base in favor of origin/<branch>" decision is
   observable in logs.
   
   Update the create command's --help text to describe the three-way resolution
   order (local → remote → fork from base), and note that --base is ignored
   when the branch already exists.

### Chore

 - <csr-id-c969fa2c98eb767452995a0d8989c6d96e073290/> bump version to 0.4.0
 - <csr-id-020bac48693bb36b95913cae6dc4cb5746ecc132/> bump default claude model from sonnet to opus
   Backend default at `Model::DEFAULT` is dead code in practice today (the
   jig.toml defaults at `context/repo.rs:94,113` shadow it on every real
   call site), so this has no runtime effect until KRO-152 consolidates
   those duplicates. Lands the intent now; behavior shift happens when the
   consolidation lands.
 - <csr-id-e1dfd6b90ac3e80c6af9fd46fd1e19fd0207b3f5/> bump version to 0.3.1
 - <csr-id-6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02/> bump version to 0.3.0
 - <csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/> bump version to 0.2.0

### New Features

 - <csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/> rewrite health check to validate repo setup and agent scaffolding
   Replace terminal-detection-focused health check with structured validation
   of system deps (git, tmux, claude), repository config (jig.toml, base
   branch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).
   Remove unused jq/gh dependency checks and dead required field. Exit
   non-zero when checks fail.
 - <csr-id-5cff652c9698daa6a067086234c50c6abd884517/> jig — git worktree manager for parallel Claude Code sessions

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 31 commits contributed to the release.
 - 20 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump version to 0.4.0 ([`c969fa2`](https://github.com/krondor-corp/jig/commit/c969fa2c98eb767452995a0d8989c6d96e073290))
    - Merge pull request #30 from krondor-corp/feature/kro-153-jig-create-must-validate-remote-exists-before-falling-back ([`b0f842d`](https://github.com/krondor-corp/jig/commit/b0f842d7cbfaeae96e1da03bae8061384fecbeec))
    - Document branch resolution order; log when base is ignored ([`a359f75`](https://github.com/krondor-corp/jig/commit/a359f75431686c36f688a905ec9c353c50bf2929))
    - Check out remote branch when origin/<branch> already exists ([`28aca79`](https://github.com/krondor-corp/jig/commit/28aca7958072029c21c92bd8782e6dc51a01b7fe))
    - Merge pull request #29 from krondor-corp/feature/kro-150-collapse-getissue-into-listissues-document-single-source-of ([`99640df`](https://github.com/krondor-corp/jig/commit/99640df5223b8eaf85d1c0bf896c2043d9317998))
    - Merge pull request #31 from krondor-corp/feature/kro-149-add-generic-eventk-in-jig-core-consolidate ([`5f96611`](https://github.com/krondor-corp/jig/commit/5f966116cf0960ce09f24db54a3b4733cf6fe4a2))
    - Add generic Event<K> and consolidate event scaffolds ([`0a996ce`](https://github.com/krondor-corp/jig/commit/0a996ced56110b176b3de348cbbd0f510b4873ac))
    - Cargo fmt ([`e7152ee`](https://github.com/krondor-corp/jig/commit/e7152ee81cd6d10e480735b1f263943db5a7bde9))
    - Validate remote exists before creating worktree; drop HEAD fallback ([`e732f67`](https://github.com/krondor-corp/jig/commit/e732f6773fa0575835719ea68aeaac8219db33b8))
    - Collapse GetIssue into ListIssues; document single source of truth ([`2393dc4`](https://github.com/krondor-corp/jig/commit/2393dc403a09b4991400aa5bbeabd50bc96b95d3))
    - Work around libgit2 ENOENT on missing .git/shallow ([`595ecea`](https://github.com/krondor-corp/jig/commit/595ecea0aeed045cb76d1ff5f0b93bdf0978ec24))
    - Bump default claude model from sonnet to opus ([`020bac4`](https://github.com/krondor-corp/jig/commit/020bac48693bb36b95913cae6dc4cb5746ecc132))
    - Merge pull request #21 from krondor-corp/release-automation ([`6e5e121`](https://github.com/krondor-corp/jig/commit/6e5e1218c422440dfe296c5a3aade37f7247f1d8))
    - Bump jig-core v0.3.1, jig-cli v0.3.1 ([`c834648`](https://github.com/krondor-corp/jig/commit/c83464870957aedcebc37cd4136472cd404c5537))
    - Bump version to 0.3.1 ([`e1dfd6b`](https://github.com/krondor-corp/jig/commit/e1dfd6b90ac3e80c6af9fd46fd1e19fd0207b3f5))
    - Cargo fmt ([`6f958a8`](https://github.com/krondor-corp/jig/commit/6f958a8c78b08a0374f80d5d2ecd868e3bed8c68))
    - Query all PR states and add integration tests (KRO-143) ([`a6a59f1`](https://github.com/krondor-corp/jig/commit/a6a59f1aad09365f570b64c283e11a0f4b281667))
    - Merge pull request #19 from krondor-corp/release-automation ([`eb8e28c`](https://github.com/krondor-corp/jig/commit/eb8e28c03456fcab820c9281ccbed24c9800feb5))
    - Bump jig-core v0.3.0, jig-cli v0.3.0 ([`ea44da2`](https://github.com/krondor-corp/jig/commit/ea44da2fd4eda8b11533f33b23f753eef5fdcdc0))
    - Bump version to 0.3.0 ([`6bd4cd5`](https://github.com/krondor-corp/jig/commit/6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02))
    - Merge pull request #14 from krondor-corp/fix/claude-allowed-tools-and-triage-tracker ([`4aa4eb7`](https://github.com/krondor-corp/jig/commit/4aa4eb704216a1c4967a90a25f5d82dffbea0d40))
    - Pass prompt via --allowed-tools= form and clear tracker after run ([`50c054d`](https://github.com/krondor-corp/jig/commit/50c054d030c0d8a42fa6f105c4771c606c94810d))
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

