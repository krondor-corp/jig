# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.3.1 (2026-05-11)

<csr-id-e1dfd6b90ac3e80c6af9fd46fd1e19fd0207b3f5/>
<csr-id-6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02/>
<csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/>
<csr-id-20f50084325dd8efff3ae0484c07027230dc1b89/>
<csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/>

### Chore

 - <csr-id-e1dfd6b90ac3e80c6af9fd46fd1e19fd0207b3f5/> bump version to 0.3.1
 - <csr-id-6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02/> bump version to 0.3.0
 - <csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/> bump version to 0.2.0

### New Features

 - <csr-id-7f3e4bae3f9bd9f5a1a813d8d80e0d1652a1814d/> add gh CLI binary and auth checks to System section
   Adds `gh` binary detection and `gh auth status` verification to the
   System section of `jig health`, so users are warned before `jig pr`
   fails due to a missing or unauthenticated `gh` CLI.
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
 - <csr-id-61c8ad3c0fc9e3ef6a47c356e8a880ee1f05afce/> keep from_cwd registry minimal; persist as side-effect
   The previous approach replaced from_cwd's in-memory registry with the
   full global one, which misunderstands the intent: from_cwd is a
   single-repo context and its registry should stay that way.
 - <csr-id-50715c855e00c63600c75031fd6203388aec8e04/> persist repo to disk on every from_cwd call
 - <csr-id-bc6271b116b9ae8ac6599896019f71dc408dadd7/> load jig.local.toml standalone when jig.toml is absent
   Previously JigToml::load returned None immediately if jig.toml did not
   exist, silently ignoring any jig.local.toml present in the repo. Repos
   with only a local config were treated as unconfigured, causing the daemon
   to skip them with no diagnostic.
   
   Now if jig.toml is absent but jig.local.toml exists, the local file is
   parsed as the full standalone config (local_only = true). The overlay
   merge path is unchanged when both files are present.
   
   Also adds JigToml::local_only_exists and a health-check warning (not
   failure) that surfaces the no-base-file situation to the user.
 - <csr-id-c751d2c2df7283e8f787781202c8cef9d1499dbf/> emit PrMerged/PrClosed events from process_pr_report
   The merged/closed branch of process_pr_report was emitting PrOpened (or
   nothing) instead of the terminal events that drive worker cleanup.
   Because no producer ever wrote PrMerged or PrClosed, the reducer could
   never transition a worker to Merged/Failed status, so the recovery prune
   never received a PruneTarget and worktrees/tmux windows were never
   cleaned up.
   
   Split the combined arm into two separate arms that emit the matching
   event variants. Guard with !state.status.is_terminal() to avoid
   re-appending on every subsequent poll once the worker has already
   reached a terminal state.
 - <csr-id-adbce583457c3b46622874c795e8b3b366b5491a/> remove unused import and use struct init syntax in tests
 - <csr-id-cc11b783045bb5d4d897e5e7660557ced9c41cc5/> resolve clippy warnings in issues list and format pr create
 - <csr-id-9e3deddc1a6070d619d13874f63a6343b7e6ab02/> pass --head flag to gh pr create for git worktree compatibility
 - <csr-id-ba3ee31ba372af56cf1ed953128792b433deaf8c/> resolve clippy warnings for CI

### Style

 - <csr-id-20f50084325dd8efff3ae0484c07027230dc1b89/> format assert in test
 - <csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/> cargo fmt

## v0.3.0 (2026-05-10)

<csr-id-6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02/>
<csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/>
<csr-id-20f50084325dd8efff3ae0484c07027230dc1b89/>
<csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/>

### Chore

 - <csr-id-6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02/> bump version to 0.3.0
 - <csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/> bump version to 0.2.0

### New Features

 - <csr-id-7f3e4bae3f9bd9f5a1a813d8d80e0d1652a1814d/> add gh CLI binary and auth checks to System section
   Adds `gh` binary detection and `gh auth status` verification to the
   System section of `jig health`, so users are warned before `jig pr`
   fails due to a missing or unauthenticated `gh` CLI.
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
 - <csr-id-61c8ad3c0fc9e3ef6a47c356e8a880ee1f05afce/> keep from_cwd registry minimal; persist as side-effect
   The previous approach replaced from_cwd's in-memory registry with the
   full global one, which misunderstands the intent: from_cwd is a
   single-repo context and its registry should stay that way.
 - <csr-id-50715c855e00c63600c75031fd6203388aec8e04/> persist repo to disk on every from_cwd call
 - <csr-id-bc6271b116b9ae8ac6599896019f71dc408dadd7/> load jig.local.toml standalone when jig.toml is absent
   Previously JigToml::load returned None immediately if jig.toml did not
   exist, silently ignoring any jig.local.toml present in the repo. Repos
   with only a local config were treated as unconfigured, causing the daemon
   to skip them with no diagnostic.
   
   Now if jig.toml is absent but jig.local.toml exists, the local file is
   parsed as the full standalone config (local_only = true). The overlay
   merge path is unchanged when both files are present.
   
   Also adds JigToml::local_only_exists and a health-check warning (not
   failure) that surfaces the no-base-file situation to the user.
 - <csr-id-c751d2c2df7283e8f787781202c8cef9d1499dbf/> emit PrMerged/PrClosed events from process_pr_report
   The merged/closed branch of process_pr_report was emitting PrOpened (or
   nothing) instead of the terminal events that drive worker cleanup.
   Because no producer ever wrote PrMerged or PrClosed, the reducer could
   never transition a worker to Merged/Failed status, so the recovery prune
   never received a PruneTarget and worktrees/tmux windows were never
   cleaned up.
   
   Split the combined arm into two separate arms that emit the matching
   event variants. Guard with !state.status.is_terminal() to avoid
   re-appending on every subsequent poll once the worker has already
   reached a terminal state.
 - <csr-id-adbce583457c3b46622874c795e8b3b366b5491a/> remove unused import and use struct init syntax in tests
 - <csr-id-cc11b783045bb5d4d897e5e7660557ced9c41cc5/> resolve clippy warnings in issues list and format pr create
 - <csr-id-9e3deddc1a6070d619d13874f63a6343b7e6ab02/> pass --head flag to gh pr create for git worktree compatibility
 - <csr-id-ba3ee31ba372af56cf1ed953128792b433deaf8c/> resolve clippy warnings for CI

### Style

 - <csr-id-20f50084325dd8efff3ae0484c07027230dc1b89/> format assert in test
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

## v1.12.0 (2026-04-13)

<csr-id-b02f6b583825d0551ff5d334ce5794542b1e212d/>
<csr-id-2bd4d0698a970edb157ecc52fcfc0c2830b375f5/>
<csr-id-e087a62731a90f8aec9897ef658e113a85c0c9f4/>
<csr-id-29da5be67a92fbb7e2cbd7674cb824ca17afb8d5/>
<csr-id-a28704f7e124ad3c35734469ff285d265fc7ccc9/>
<csr-id-e63634542688c53115dac2f70254224545dcb4c8/>
<csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/>
<csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/>
<csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/>
<csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/>
<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-506098c0047a2a342ca56cac5757a3e11e70cc7c/>
<csr-id-966b95cc52fc563eb2d4cc5762e7cea1edd8119b/>
<csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>
<csr-id-9acbc64d601db75393903d07daca6ae2c16a1b54/>
<csr-id-3cba2cc100dff6c024c7ba47767447008a6932d1/>

### Chore

 - <csr-id-b02f6b583825d0551ff5d334ce5794542b1e212d/> bump version to 1.12.0
 - <csr-id-2bd4d0698a970edb157ecc52fcfc0c2830b375f5/> bump version to 1.11.1
 - <csr-id-e087a62731a90f8aec9897ef658e113a85c0c9f4/> bump version to 1.11.0
 - <csr-id-29da5be67a92fbb7e2cbd7674cb824ca17afb8d5/> bump version to 1.10.0
 - <csr-id-a28704f7e124ad3c35734469ff285d265fc7ccc9/> bump version to 1.9.0
 - <csr-id-e63634542688c53115dac2f70254224545dcb4c8/> bump version to 1.8.0
 - <csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/> bump version to 1.7.1
 - <csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/> bump version to 1.7.0
 - <csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/> bump version to 1.6.0
 - <csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/> bump version to 1.5.0
 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

 - <csr-id-82baa06149e54a9f4bb819f1a01d609849b11f7f/> resolve parent branch for manual-child path, add docs
   - CLI `jig spawn --issue <child>` now auto-resolves the parent branch
   as the base when the issue has a parent with a branch_name, matching
   the daemon spawn behavior.
   - Add docs/parent-child.md covering the parent-as-integrator model,
   auto vs manual decision framework, step-by-step manual flow, PR base
   resolution, non-goals, and file provider limitations.
   - Add integration tests for the manual-child lifecycle (issue creation
   with --parent, detail display, multiple children, worktree creation,
   status transitions preserving parent field).
   - Update docs/index.md to reference the new doc.
 - <csr-id-7b31c4330c5d88b0295fc469c61fab43bf89ab3b/> assignee support and additive label edits on jig issues update
   Adds three user-facing improvements to `jig issues update` and fixes a
   silent no-op bug in label resolution:
   
   - `--assignee "me"|<user-id>` for Linear: plumbed through
   `LinearClient::update_issue` -> `LinearProvider::update_issue` ->
   `run_update`. "me" resolves via `client.viewer_id()`, matching the
   existing create-path semantics. Rejected for the file provider.
   - `--add-label` / `--remove-label` for additive edits. Reads the issue's
   current label set, applies the delta, and passes the resulting set
   through the existing replace-style mutation. Rejects combining these
   with the replace-style `--label`.
   - `Detail` view now prints a `Labels:` line so `jig issues <id>` can be
   used to audit labels before editing.
 - <csr-id-04879baac1e42348935db971f34ebb93707c0ca4/> add jig notify CLI with doctor, test, tail, send subcommands
   Add `jig notify` subcommand tree for inspecting, testing, and sending
   notifications. Also add `emit_strict()` to Notifier that surfaces exec
   hook errors (with captured stderr) instead of swallowing them.
   
   - `doctor`: prints resolved NotifyConfig, queue status, and surfaces
   TOML parse errors that would otherwise silently fall back to defaults
   - `test`: emits synthetic NeedsIntervention through full pipeline
   - `tail [-n N]`: prints last N events from the JSONL queue
   - `send <kind>`: agent-facing command to emit real notification events
   - `emit_strict()`: returns Err on hook failure with captured stderr,
   used by CLI commands; daemon continues using best-effort `emit()`
 - <csr-id-b48f685a3e6f45a8a13d038fdbed9ef67c8ecda1/> update skills and templates for review system
   Add automated review instructions to SPAWN_PREAMBLE so spawned workers
   know how to respond to review findings. Create review-respond skill
   template and register it in jig init. The .jig/ directory is already
   gitignored by ensure_gitignored().
 - <csr-id-89f85a84297ae29f2cc8d10d5e62bc9678469c0a/> default `issues create` status to backlog
   Without an explicit `-s`, new issues now land in Backlog instead of
   the provider's default (which, for Linear workspaces with triage
   enabled, is Triage — causing the daemon to immediately pick them up
   for auto-triage). Callers can still override with `-s triage` or any
   other status.
 - <csr-id-62e42a7110ef329596f47e3b721ac66380c9560c/> create with initial status atomically
   Adds `-s/--status` to `jig issues create` so issues land directly in
   the target workflow state. Previously the only way to create an issue
   in a non-default state was create-then-update, which left a race
   window where a daemon could observe the issue in its default state
   (e.g. Triage) and trigger auto-workflows before the follow-up status
   update landed.
   
   - linear_client: extract `resolve_state_id` helper used by both
   `update_issue_status` and `create_issue`; thread an
   `initial_status: Option<&IssueStatus>` through `create_issue` and
   inject it as `stateId` in the IssueCreateInput mutation — atomic.
   - linear_provider / file_provider: plumb `initial_status` through.
   File provider applies it after template rendering via the existing
   `replace_field` helper, overriding the template's default
   `Status: Planned`.
   - CLI: new `-s/--status` flag on `issues create`, validated up front.
 - <csr-id-9b04cce3cb51ec70b6756bc839d410f8eafb6528/> add jig pr command for automatic draft PR creation
   Pushes current branch and creates a draft PR with automatic base branch
   resolution. When running inside a jig worktree with a parent issue, the
   PR targets the parent issue's branch. Otherwise falls back to the repo's
   base branch.
 - <csr-id-fdd129ec304b2b17a11bf64ca6df0832a2c8b388/> add -P short flag for --parent on issues create/update
   The --parent flag was already implemented but lacked the -P short form
   specified in the issue. Add short = 'P' to both create and update
   subcommands for ergonomic CLI usage.
 - <csr-id-058be99b3d376ce22f135948837cecc5f817ac14/> add `jig review submit` and `jig review respond` CLI commands
   Add subcommands for agents to write validated review files into a
   worktree's `.jig/reviews/` directory. `submit` reads review markdown
   from stdin, validates via Review::from_markdown(), and writes to the
   next numbered file. `respond` reads response markdown from stdin,
   validates the referenced review exists, and writes the response file.
   The existing `jig review <name>` behavior moves to `jig review show`.
 - <csr-id-1f5b4f39301dc456c75f01a67e76da6f876f64ee/> add parent fields to Issue struct and Linear GraphQL
   Replace the single `parent: Option<(String, String)>` tuple with
   five dedicated fields: parent_id, parent_branch_name, parent_status,
   parent_title, and parent_body. Expand the Linear GraphQL queries to
   fetch description, branchName, and state from the parent issue.
 - <csr-id-bad95206b0394331917c7a577743b4f00cb37474/> triage worker output to Linear
   Add Triage and Backlog variants to IssueStatus with full Linear
   workflow state mapping. Add --append flag to `jig issues update` so
   triage workers can append investigation findings to existing issue
   descriptions. Add daemon triage completion verification that emits
   NeedsIntervention when a triage worker exits but the issue remains
   in Triage status.
 - <csr-id-49cb0ad36b35c6b494732d02f36c0187995b4db9/> add --parent flag to create/update for sub-issue relations
   Add `--parent` flag to `jig issues create` and `jig issues update` to
   link issues as sub-issues of a parent. Add `--remove-parent` flag to
   `jig issues update` to clear the relation.
   
   For Linear provider, uses the `parentId` field on issueCreate/issueUpdate
   mutations. For file provider, stores as a `**Parent:**` frontmatter field.
   
   Parent info is displayed in the single-issue detail view and interactive
   pager when set.
 - <csr-id-ef585aedc2a758b0e7e238aa3bf523196bb28a7a/> add dependency management via CLI
   Add --blocked-by and --remove-blocked-by flags to `jig issues update`
   for creating and removing issue dependency relations. Works with both
   file-based and Linear providers. Single issue view now displays
   dependencies.
 - <csr-id-11a94ff3771fda2aaec8fd036e9a92966cf9688b/> add `jig issues update` command for editing existing issues
   Add a new `update` subcommand to `jig issues` that supports updating
   an existing issue's title, body, priority, labels, and category for
   both file-based and Linear providers.
   
   - LinearClient: add UPDATE_ISSUE_MUTATION and update_issue() method
   - LinearProvider: add update_issue() delegating to client
   - FileProvider: add update_issue() with replace_title/replace_body helpers
   - CLI: add Update variant to IssuesCommand with --title, --body,
   --priority, --label, --category flags
   - Integration tests: 6 new tests covering title, priority, labels,
   body, multiple fields, and no-fields error case
   - Unit tests: 4 new tests for replace_title, replace_body,
   update_issue_title_and_priority, update_issue_labels
 - <csr-id-f313d4c1e1dba5e00cfcc1127eda9adb0f12d599/> move issues to InProgress on spawn to prevent duplicates
   Add `update_status` to the `IssueProvider` trait and call it in both
   spawn code paths (daemon auto-spawn and CLI `jig spawn`) to transition
   issues to InProgress immediately after a worker is launched. This
   prevents `list_spawnable()` from re-picking the same issue across tick
   cycles, since it filters for Planned status only.
 - <csr-id-b9bca75b2c6e59bcbd84c173a3e92b7717362baf/> improve create UX and add docs/tests
   - Make --category optional (defaults to "features" for file provider,
   omitted for Linear to avoid passing a meaningless project name)
   - Document `jig issues create` in Linear integration docs and issues skill
   - Add integration tests for default category, stdin body, and labels
 - <csr-id-e4445e544c26244b44b9051d72d5f34cd0c87da3/> support Linear provider for `jig issues create`
   Wire up `jig issues create` to dispatch through LinearProvider when
   provider = "linear" is configured. Adds a createIssue GraphQL mutation
   to LinearClient with helper queries to resolve team, label, and project
   IDs. The CLI gains a --body flag (inline text or "-" for stdin) that
   works with both file and Linear providers.
 - <csr-id-99f0311db0fb1fcac156aa2f03e8b04bfdd75199/> add jig.local.toml overlay and fix stderr color detection
   Add deep-merge support for jig.local.toml (gitignored) on top of
   jig.toml, allowing machine-specific overrides without touching the
   committed config. Tables merge recursively; scalars and arrays from the
   local file win. `jig init` auto-adds jig.local.toml to .gitignore.
   
   Revamp `jig config` display with source attribution showing where each
   setting originates (jig.toml, jig.local.toml, global config, or default).
   
   Fix colored crate TTY detection: override based on stderr since all jig
   output goes to stderr, preventing silent color suppression when stdout
   is piped.
 - <csr-id-0f9a72d8d53c686ff305a11c961c37083f26a47d/> add Create event and Created status for bare worktrees
   Distinguishes `jig create` worktrees from `jig spawn` workers via a
   positive Create event rather than relying on empty event logs. The daemon
   discovers Created workers (they appear in `jig list`) but takes no
   actions on them — no auto-resume, no nudges, no recovery.
 - <csr-id-205483042ca48167984c4076b0d01bbd81a00f2f/> derive spawn worktree name from issue, use repo base branch in nudges
   - Make `name` optional in `jig spawn` when `--issue` is provided; derive
   worktree name from the issue's branch_name or ID
   - Extract Linear identifiers from branch-format strings (e.g.
   `feature/aut-5044-refactor-foo` → `AUT-5044`)
   - Move derive_worker_name/sanitize_worker_name to shared issues::naming module
   - Pass repo's actual base branch to conflict and bad-commits nudge templates
   instead of hardcoding origin/main
   - Simplify resume to reuse launch(), remove build_resume_command
   - Skip recovery for Initializing workers (still running on-create hooks)
 - <csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/> add commit-msg hook for conventional commit validation
   Adds a commit-msg git hook that validates commit messages against the
   conventional commits spec using the existing parser and jig.toml config.
   Closes the conventional-commits-validation issue.
 - <csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/> add conventional commit validation and examples
   Add parser, configurable validator, and CLI commands:
   - `jig commit validate` validates HEAD, specific revs, stdin, or files
   - `jig commit examples` shows conventional commit reference
   - Configurable via `[commits]` section in jig.toml
   - 16 unit tests + 12 integration tests
 - <csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/> add issue lifecycle commands (create, status, complete, stats)
   Add CLI subcommands to `jig issues` for managing issue state:
   - `jig issues create` creates issues from templates with title, priority, category, labels
   - `jig issues status <id> --status <new>` updates frontmatter status in file issues
   - `jig issues complete <id>` marks issues as complete, with optional --delete
   - `jig issues stats` shows breakdown by status and priority
   
   For Linear issues, status changes go through the API (not file editing).
   Backwards-compatible: `jig issues` with no subcommand still lists/browses.
   Also adds "in-progress" (hyphenated) to loose status parsing.
 - <csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/> add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limiting
   Add three display features to make ps -w a proper dashboard:
   - Nudge cooldown countdown in NUDGE column (e.g. "2/3 (3m12s)")
   - Nudge messages rendered below worker table when delivered
   - Global sync/poll timer footer alongside keybinding hints
   
   Fix two bugs discovered during implementation:
   - Preserve draft PR status on GitHub API errors so nudges aren't
   silently suppressed for known-draft PRs
   - Throttle GitHub API requests to once per 60s per worker, aligned
   with gh --cache 60s TTL, reducing API pressure ~30x
 - <csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/> add daemon crash recovery and worker resume
   - Install SIGTERM/SIGINT handler for graceful daemon shutdown
   - Record Started/Stopped lifecycle events in daemon.jsonl
   - Detect unclean shutdown on next startup (missing Stopped event)
   - Auto-recover orphaned workers on daemon startup via Worktree::resume()
   - Add `jig resume <name>` CLI command for manual worker recovery
   - Detect dead tmux windows during steady-state ticks and auto-resume
   - Add `auto_recover` global config option (default: true) for opt-out
   - Wire Action::Restart to use recovery::try_resume_worker()
   - Add integration tests for jig resume command
 - <csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/> show nudge state in jig ps output
   Add NUDGE column between STATE and COMMITS in the ps table.
   Displays nudge count as count/max (e.g. 2/3), with color coding:
   grey dash for zero, yellow for in-progress, red for exhausted.
   
   Adds max_nudges field to WorkerDisplayInfo so the UI can render
   the denominator from the resolved health config.
 - <csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/> add per-repo nudge configuration in jig.toml
   Add [health] section to jig.toml supporting per-repo overrides of
   silence_threshold_seconds and max_nudges, plus per-nudge-type
   [health.nudge.<type>] sections with independent max and
   cooldown_seconds settings.
   
   Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]
   > global config > defaults. When cooldown_seconds is not set, falls
   back to silence_threshold_seconds.
   
   - Add RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structs
   - Add ResolvedNudgeConfig with resolver for per-type config
   - Thread resolved config through nudge classify, dispatch, and execute
   - Apply per-type cooldown to both idle/stalled nudges and PR nudges
   - Display effective nudge config in `jig config show`
   - Fixes PR nudge burst bug by enforcing per-type cooldowns
 - <csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/> add jig home command to print base repo root
   Adds `jig home` (alias `jig h`) that prints the base repository root
   path, enabling `cd $(jig home)` navigation from worktrees.
 - <csr-id-df0a3be811b27f8afce047bd088cad410d09e081/> communicate worker initialization state and on-create failures
   Add Initializing event type and worker status to make the worker
   lifecycle visible during setup. When the daemon auto-spawns a worker,
   it now registers the worker as Initializing before running the
   on-create hook, then transitions to Spawned on success or Failed on
   hook failure.
 - <csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/> show auto-spawn config in `jig config` and add `jig issues --auto`
   Display auto-spawn settings (enabled, auto-start, max workers, poll
   interval, spawn labels) in `jig config show` with source attribution.
   Add `--auto` flag to `jig issues` to filter to only daemon-eligible
   auto-spawn candidates using the existing `list_spawnable` method.
 - <csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/> use checkmark instead of asterisk for AUTO column
   Change the AUTO column indicator in `jig issues` from `*` to `✓`
   for better readability.
 - <csr-id-bd1a1faeca5a7634224bef836154791819b4903b/> use checkmark instead of asterisk for AUTO column
   Change the AUTO column indicator in `jig issues` from `*` to `✓`
   for better readability.
 - <csr-id-462f05eaf29929899631125c733738cd8f93e558/> move auto-spawn to background thread to keep ps -w responsive
   The on-create hook (e.g. pnpm install) was running synchronously on
   the tick thread, freezing the ps --watch UI for the entire duration.
   Introduces a spawn_actor following the same pattern as prune_actor,
   issue_actor, etc. The tick now sends spawnable issues to the background
   thread and drains results on the next tick.
   
   Also adds:
   - Spawning worker names shown below the ps table during setup
   - WorkerStatus::Initializing variant for future use
   - spawn_labels config in jig.toml
   - Three new issues (config-show-auto-spawn, worker-initializing-state,
   auto-column-checkmark)
 - <csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/> add labels field for issue tagging and filtering
   Add `labels: Vec<String>` to Issue and IssueFilter types. Linear
   provider now passes all label names through from GraphQL (auto field
   derivation unchanged). File provider parses `**Labels:**` comma-separated
   frontmatter. CLI gains `--label/-l` flag for filtering (all must match).
   Shell completions updated for bash, zsh, and fish.
 - <csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/> add shared UI module with consistent formatting and --plain flag
   Expand ui.rs into a centralized formatting module with:
   - Status symbol constants (✓, →, ✗, !)
   - Formatted output helpers (success, progress, failure, warning, detail, header)
   - Color helpers (highlight, bold, dim) that respect plain mode
   - Table builder helper (new_table) for consistent table creation
   - Global --plain flag for scriptable output (no colors, no decorations)
   - Error display with cause chain formatting
   
   Migrate all 20 command files from inline colored::Colorize calls to
   shared ui:: helpers. Add --plain support to list, repos, and issues
   commands with tab-separated output for piping.
 - <csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/> add AUTO column to `jig issues` table output
   Show a green dot indicator for issues tagged for auto-spawn, making it
   visible at a glance whether file-provider Auto flag or Linear jig-auto
   label is set.
 - <csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/> support -g/--global flag for attaching from anywhere
   Add run_global implementation to the Attach command so users can attach
   to a worktree from outside the owning repo using `jig attach <name> -g`.
   Resolves the owning repo via GlobalCtx::repo_for_worktree.
 - <csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/> jig init --audit launches agent in tmux to populate docs
   --audit now spawns the configured agent in a jig-init:<repo> tmux
   session with the audit prompt instead of just printing instructions.
   --backup enhances the prompt to reference .backup/ files. --audit
   accepts an optional string for extra instructions.
 - <csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/> block auto-spawn on unresolved dependencies
   Add `is_spawnable_with_deps()` to IssueProvider trait that checks all
   depends_on entries resolve to Complete before allowing spawn. Applied in
   both FileProvider and LinearProvider's list_spawnable(). Also adds
   --blocked/--unblocked flags to `jig issues` CLI for filtering.
 - <csr-id-057e8dc3675610e75e826910d051774f32f63cee/> group workers by repo in `jig ps -g` output
   Add `repo` field to `WorkerDisplayInfo` and render grouped tables with
   bold repo headers when running in global mode (`jig ps -g` / `jig ps -gw`).
   Local `jig ps` output is unchanged.
 - <csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/> daemon periodically prunes stale worktrees
   Workers in terminal state (merged/archived/failed) with dead tmux
   sessions now get their git worktrees, event logs, and global state
   entries cleaned up automatically. Prune runs every 120s during watch
   mode. Pruned workers are reported in the tick status and log view.
   
   Also includes snake_case fixes for auto-spawn-filtering ticket.
 - <csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/> default table view for `jig ls` and pretty grouped `jig ls -g`
   - `jig ls` now shows a table with name, branch, and commits ahead
   - `jig ls -g` shows tables grouped by repo with bold headers
   - Add `--plain/-p` flag for bare name output (old behavior)
   - Shell completions use `--plain` and fall back to `-gp` outside a repo
   - Branch column only shown when it differs from worktree name
 - <csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/> show draft vs review state, document PR nudge behavior
   Workers with draft PRs now show "draft" (blue) in the STATE column
   instead of "review" (cyan). This makes it visually clear which workers
   will receive PR nudges (draft) vs which are in human review (non-draft).
   
   Add PR Nudges section to daemon docs explaining the draft/non-draft
   nudge policy and what each health check means.
 - <csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/> unify daemon/ps tick loops and add log toggle to watch mode
   Extract run_with() callback API from daemon so ps --watch shares the
   same setup code path instead of duplicating Daemon/Notifier/TmuxClient
   construction. The callback controls inter-tick delay and can signal
   stop, which enables keypress handling during the sleep window.
   
   Add log view toggle to watch mode: press 'l' to see timestamped daemon
   activity (nudges fired, PR check results, errors), 't' to switch back
   to the table, 'q' to quit cleanly. Uses crossterm raw mode with 100ms
   poll intervals for responsive input.
   
   Also allows spawned workers to transition to stalled (previously
   Spawned status was excluded from silence detection).
 - <csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/> surface PR health in ps --watch display
   Add a HEALTH column to the watch table showing per-worker PR check
   results (ci, conflicts, reviews, commits) so problems are visible at a
   glance without needing RUST_LOG=debug. Upgrade silent debug-level PR
   errors to info-level logging.
 - <csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/> add --base flag to spawn and create for custom branch base
   Allow overriding the default base branch (from jig.toml) per-command
   with --base/-b. Includes shell completions for branch names across
   bash, zsh, and fish. Also fixes spawn status message to show the
   actual base branch used instead of the current branch.
 - <csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/> wire issues into spawn pipeline with --issue flag
   Add `jig spawn --issue <id>` to resolve file-based issues and use their
   body as Claude context. Thread issue_ref through the full pipeline:
   spawn CLI → register() → Spawn event → WorkerState reducer → daemon
   workers.json → ps watch table.
   
   Also adds:
   - `jig issues` CLI command with --ids flag for scripting
   - IssuesConfig in jig.toml for configurable issues directory
   - ISSUE column in ps --watch table (shortened last path segment)
   - Shell completions for --issue in bash, zsh, and fish
   - issue_ref tests in reducer and daemon roundtrip
 - <csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/> add watch mode to ps command for live dashboard
   `jig ps --watch` clears and refreshes the worker table every 2s.
   Shows enriched state from event logs alongside tmux status:
   - TMUX column (●/○/✗) for session liveness
   - STATE column from event-derived WorkerStatus
   - NUDGES count and PR number from event log
   - Configurable interval: `jig ps -w 5` for 5s refresh
 - <csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/> add daemon loop to orchestrate event-driven pipeline
   The missing conductor: `jig daemon` runs a periodic loop that:
   - Discovers workers by scanning event log directories
   - Replays events to derive current WorkerState per worker
   - Compares old vs new state to dispatch actions
   - Executes nudges via tmux and notifications via hooks
   - Persists state to workers.json between ticks
   
   Supports --once for single-pass mode and --interval for tuning.
 - <csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/> add git hook management (install, uninstall, handlers)
   Implements the git-hooks epic (tickets 0-4):
   - Hook wrapper templates that chain jig logic with user hooks
   - Registry tracking installed hooks at jig-hooks.json
   - Idempotent init with backup/restore of existing hooks
   - Post-commit/merge handlers that emit events to worker logs
   - Uninstall with rollback to original user hooks
 - <csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/> expand WorkerStatus with event-driven states
   Add Idle, WaitingInput, Stalled variants. Make all variants unit types
   (remove associated data from WaitingReview/Failed). Add needs_attention(),
   is_active(), is_terminal(), from_legacy() methods. Snake_case serialization.
 - <csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/> add event log format and Claude Code hooks
   Implement event-system tickets 1 and 2:
   - Event schema with typed EventType enum and flat JSONL serialization
   - EventLog append-only reader/writer with per-worker JSONL files
   - Claude Code hook templates (PostToolUse, Notification, Stop)
   - `jig hooks install-claude` CLI command to install hooks to ~/.claude/hooks/
 - <csr-id-82c654ab1137ec963121638f6741617c59ee0c04/> add global state infrastructure for cross-repo aggregation
   Introduces ~/.config/jig/ directory structure with structured TOML config,
   aggregated JSON worker state, and event log directories for the event-driven
   pipeline. Ensures global dirs are created at CLI startup.
 - <csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/> introduce RepoContext and thread repo state through all operations
   Derive repo_root, worktrees_dir, git_common_dir, base_branch, and
   session_name once at startup via RepoContext::from_cwd(), eliminating
   redundant git subprocess calls (e.g. spawn called get_base_repo() 8x).
   OpContext now holds Option<RepoContext>, and all jig-core functions
   accept &RepoContext instead of re-deriving from cwd. Also adds repo
   registry for global mode auto-registration, removes dead spawn::kill(),
   and updates docs/patterns/issue status.
 - <csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/> implement smart jig update command
   Rewrite update command to:
   - Detect installation method (script, cargo, source, unknown)
   - Check latest version from GitHub releases API
   - Auto-update for script installations (~/.local/bin)
   - Prompt dev builds to install release binaries
   - Offer cleanup of old cargo bin after source build updates
   - Add --force flag to skip version check
 - <csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/> prettify jig ps with Op pattern and comfy-table
   Introduce the Op trait to separate command logic from presentation.
   Rewrite `jig ps` as the first adopter: ops return typed data, Display
   impls own all formatting via comfy-table with terminal-width-aware
   column layout and color-coded status indicators.
   
   - Add Op trait in crates/jig-cli/src/op.rs
   - Rewrite ps command with PsOutput, PsError, and Op impl
   - Add comfy-table dependency for dynamic table rendering
   - Update main.rs dispatch to use Op::execute()
   - Add docs/ui/STDOUT-FORMATTING.md documenting the pattern
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
 - <csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/> restructure issue tracking with categories and templates
   - Add directory-based issue organization (epics/, features/, bugs/, chores/)
   - Add issue templates (_templates/): standalone.md, epic-index.md, ticket.md
   - Create plan-and-execute epic for orchestration vision
   - Update issues/README.md with comprehensive documentation
   - Update /issues skill for new directory structure
   - Remove old flat issue files and _template.md
   - Add .backup/ to .gitignore
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
 - <csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/> improve backup, audit prompt, and review skill
   - Backup now copies files to .backup/ directory preserving path structure
   - Audit prompt is detailed and opinionated about what to fill in each doc
   - Review skill now checks for documentation and skills updates
 - <csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/> upgrade jig init scaffolding to language-agnostic skeletons
   - Move issue-tracking.md to issues/README.md, fix "wt" → "jig"
   - Rename skills/jig → skills/spawn for consistency
   - Remove name: field from skill frontmatter
   - Add skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.md
   - Expand docs/index.md as documentation hub
   - Make CLAUDE.md template a skeleton with guidance comments
   - Upgrade settings.json: add $schema, ask tier for destructive ops, better secret patterns
   - Add issues/_template.md ticket template
 - <csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/> add Claude Code skills and simplify permissions
   - Add skills for check, draft, issues, review, and spawn commands
   - Simplify .claude/settings.json using wildcard permissions
   - Add jig.toml with spawn auto-configuration
   - Fix formatting in init.rs
 - <csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/> use actual templates for jig init instead of bare-bones placeholders
   - Embed templates from templates/ directory using include_str!
   - Add all 5 skills: check, draft, issues, review, spawn
   - Expand permissions to cover tools used by skills
   - Set spawn.auto = true by default
   - Use exec() on Unix for --audit flag (full terminal control)
   
   The init command now creates a complete scaffolding that matches
   the documentation, instead of empty placeholder comments.
 - <csr-id-3a78670c102178f25db9dc4020b534370fc36f84/> add --audit flag to init command that launches Claude interactively
   Uses exec() on Unix to replace the current process with Claude Code,
   giving it full terminal control for interactive documentation audit.
 - <csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/> add shell-setup command and fix shell completions
   - Add `jig shell-setup` command to automatically configure shell integration
   - Detects user's shell from $SHELL
   - Finds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)
   - Adds eval line with markers for easy identification
   - Places integration after PATH setup when possible
   - Supports --dry-run flag to preview changes
   
   - Rewrite shell completions with dynamic worktree completion
   - `jig open/attach/review/merge/kill/status <TAB>` shows actual worktrees
   - Context-aware completions for all subcommands
   - Simplified zsh completion using _arguments -C
   
   - Update docs/usage/shell-integration.md
   - Add quick setup section for shell-setup command
   - Add troubleshooting section for common issues
   - Remove stale `sc` alias references (legacy from "scribe" name)
 - <csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/> rewrite health check to validate repo setup and agent scaffolding
   Replace terminal-detection-focused health check with structured validation
   of system deps (git, tmux, claude), repository config (jig.toml, base
   branch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).
   Remove unused jq/gh dependency checks and dead required field. Exit
   non-zero when checks fail.
 - <csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/> add shell completions for bash, zsh, and fish
   Shell completions are now emitted alongside the shell wrapper function
   in `jig shell-init`. Completions cover all subcommands, aliases,
   per-command flags, nested config subcommands, and dynamic worktree
   name completion via `command jig list`.

### Bug Fixes

 - <csr-id-0d1cfe04abda4fd7954bfde322d2b1366475d847/> resolve merge conflicts with main
   Merge upstream triage collection logic from issue_actor, wire SpawnKind
   through triageable/spawnable paths, align TriageConfig defaults (enabled=false,
   timeout i64), and add model field to merged triage config.
 - <csr-id-f76e8862f12c02b572270e4c90133ba5e254dcdf/> fix ParentIssue destructuring after rebase on main
 - <csr-id-65decdaa1e53d186bf2e16bfe0232bac6a84d1a5/> preserve `jig review <name>` backward compatibility
   Make the review subcommand optional so `jig review <name>` continues
   to work as shorthand for `jig review show <name>`, avoiding a breaking
   change to the existing CLI surface.
 - <csr-id-3b4465812927c74693a63577f395f5a64334e614/> fix large_enum_variant clippy errors
   Box CoreIssue in IssuesOutput::Detail to reduce variant size.
   Add #[allow(clippy::large_enum_variant)] to macro-generated Command
   and OpOutput enums where boxing isn't practical.
 - <csr-id-df2888d12a60c5b9b41828d79bcc865d82304f8b/> allow large_enum_variant in Command enum
   The Issues variant grew past clippy's threshold after adding
   blocked_by/remove_blocked_by Vec<String> fields. Boxing is not
   worth the complexity for a CLI entry point enum.
 - <csr-id-3c50caba9d28dcdb33d8a8811b65a3d8335f7188/> restore original provider helper methods in issues.rs
   Reverts unintended refactoring that replaced repo.issue_provider(),
   repo.linear_provider(), repo.file_provider(), and repo.jig_toml
   with explicit JigToml::load() / issues::make_* calls throughout
   the issues command. Only run_update needs explicit construction.
 - <csr-id-6d4388a490b70bb3401aa84bf936c2cfb9239e3a/> cherry-pick bug fixes from review experiment branch
   - Skip PENDING draft reviews in get_reviews() to prevent false nudges
   while a reviewer is still writing comments
   - Add enforce_styling() to worker tables so comfy-table renders ANSI
   colors regardless of terminal detection
   - Suppress review nudges after dev pushes (dev_pushed_after_reviews
   helper) — if the developer already pushed commits after the latest
   review feedback, the ball is in the reviewer's court
   - Add head_sha field to PrStateInfo for commit-aware operations
   - Make gh_api() pub(crate) to support the new helper
 - <csr-id-47d683b540b09d52770902df1a3d47e501372ba9/> consolidate spawn codepaths to ensure update_status in watch mode
   The watch-mode daemon (spawn_actor::spawn_single) was missing the
   update_status(InProgress) call after spawning, causing issues to be
   re-spawned on every tick. This was because three near-duplicate spawn
   codepaths existed (CLI, daemon blocking, daemon watch) and only two
   had the status update.
   
   Extract spawn_worker_for_issue() and update_issue_status() into the
   shared spawn module so all three codepaths use a single authoritative
   implementation. This eliminates ~120 lines of duplication and makes it
   impossible to miss a step in any codepath.
 - <csr-id-c5b52f59055c0f95498ef657685a18405bf6b515/> consolidate spawn codepaths to ensure update_status in watch mode
   The watch-mode daemon (spawn_actor::spawn_single) was missing the
   update_status(InProgress) call after spawning, causing issues to be
   re-spawned on every tick. This was because three near-duplicate spawn
   codepaths existed (CLI, daemon blocking, daemon watch) and only two
   had the status update.
   
   Extract spawn_worker_for_issue() and update_issue_status() into the
   shared spawn module so all three codepaths use a single authoritative
   implementation. This eliminates ~120 lines of duplication and makes it
   impossible to miss a step in any codepath.
 - <csr-id-0e71bb2f733c8d97cffe7dc2c1826c3aa61da2a9/> support -g/--global flag and show provider-specific details
   `jig config` and `jig config -g` now work outside a git repo by showing
   global config. Issues section displays Linear profile details (profile,
   team, projects, assignee, labels) when provider is "linear", and directory
   when provider is "file".
 - <csr-id-8015b3097d63862324524ac9e88e2bd411982839/> add .jig/ to .gitignore during init
   Previously .jig/ was only added to .git/info/exclude which is
   local to each clone. This meant hooks and state could leak into
   commits in fresh clones or CI. Now `jig init` adds .jig/ alongside
   jig.local.toml in .gitignore so the ignore is shared across clones.
 - <csr-id-847c9ba0e63c6aeb5a46fead6529eca9b8976465/> use `jig -g init` instead of broken `--global` flag on init
   The `--global` flag conflicts with the top-level `-g/--global` CLI flag.
   Implement `run_global()` on Init so `jig -g init` scaffolds the global
   config correctly.
 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter
   - Relation mapping was inverted: "blocks" → "is_blocked_by" so
   dependency checking now correctly identifies blocked issues
   - Remove unused SearchData type
   
   Improve issues command UX:
   - Hide completed issues by default (use --all to include them)
   - Interactive mode (-i) uses alternate screen buffer like less/git diff
   - Add ui::with_alternate_screen() reusable helper
   - Interactive mode: scrolling, auto indicator, title truncation, G/g nav
 - <csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/> accept trailing args in git hook subcommands
   Git passes arguments to hooks (e.g. post-merge receives a squash flag
   "0" or "1"), which the hook wrapper forwards via "$@". The CLI
   subcommands rejected these unexpected args. Add trailing_var_arg to
   PostCommit, PostMerge, and PreCommit to accept and ignore them.
 - <csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/> output cd command from jig home instead of bare path
   Matches the pattern used by `jig open` and `jig exit` — outputs
   `cd '/path'` to stdout for shell eval, not just the path.
 - <csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/> recover from stale git worktree registrations on spawn and prune
   When a worktree directory is removed but git still tracks the entry,
   `git worktree add` fails with "missing but already registered". Now
   create_worktree runs `git worktree prune` first, and prune_actor
   handles the missing-directory case instead of skipping cleanup.
   
   Also extracts prune_actor into its own module and adds urgent issue
   to replace git CLI shelling with git2.
 - <csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/> add issues command to shell completions
   The `issues` command was missing from all three shells' command lists
   and had no flag/argument completions. Adds command entry, issue ID
   positional completions, and flag completions (status, priority,
   category, interactive, ids) for bash, zsh, and fish.
 - <csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/> use if-let instead of unwrap to satisfy clippy
 - <csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/> daemon PR discovery, tmux targeting, and nudge delivery
   - Add proactive PR discovery: daemon queries GitHub for open PRs on
   worker branches when pr_url is unknown, emits PrOpened events to
   make state durable across restarts
   - Create per-repo GitHub clients via registry path lookup instead of
   ambient remote detection (fixes multi-repo daemon)
   - Extract real branch name from spawn events for tmux window lookup
   (spawn creates windows with slashes, e.g. feature/foo, not dashes)
   - Run all four PR checks (CI, conflicts, reviews, commits) on open PRs
   - Nudge on every tick, not just state transitions, so polling daemon
   retries delivery until max_nudges
   - Collapse multiline nudge templates to single line before tmux send
   to prevent premature submission in TUIs
   - Fix tracing init: RUST_LOG now properly overrides default warn level
   - Add stderr tick summary in continuous daemon mode for visibility
   without RUST_LOG
   - Add debug logging for tmux window misses and notification pipeline
 - <csr-id-378031a0afe019f57edc9bae469bf8168e05de29/> register Claude hooks in settings.json, add kill --all and nuke
   Claude Code hooks were installed as scripts but never registered in
   ~/.claude/settings.json, so they never fired. Now jig init registers
   them properly. Also fixes: hook templates read JSON from stdin (not
   env vars), spawned workers no longer nudged as stalled, event logs
   reset on respawn, row ordering stabilized in ps --watch, kill/unregister
   cleans up event logs, and nuke command added for full repo cleanup.
 - <csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/> address review findings and wire up event pipeline end-to-end
   Fix 6 issues from code review: UTF-8 safe truncate, stable status
   serialization via as_str/from_legacy, stuck nudge sends message after
   auto-approve, notification errors logged, branch names URL-encoded,
   tmux commands check exit status.
   
   Wire up missing pipeline links: jig spawn emits Spawn event, jig init
   auto-installs git+Claude hooks (idempotent on re-run), ps --watch runs
   daemon tick on each refresh for integrated orchestration.
   
   Add docs/daemon.md with background service setup for launchd, systemd,
   OpenRC, and generic nohup.
 - <csr-id-a41b92cb77141469539658c133da79f79f714452/> remove unnecessary return statement
 - <csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/> make --audit print command instead of trying to launch claude
   Spawning claude programmatically was causing terminal issues and hangs.
   Now --audit just prints the command for the user to run manually.
 - <csr-id-196774225c8eba52fdb9382f98418ecf82c48567/> prevent shell-setup from corrupting shell config files
   The previous byte-slicing approach in find_path_line_end() calculated
   offsets incorrectly because lines() strips newlines but the code assumed
   +1 byte per line. This could corrupt or truncate config files.

### Other

 - <csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/> fmt
 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-506098c0047a2a342ca56cac5757a3e11e70cc7c/> wrap parent fields in ParentIssue struct
   Address review feedback: replace five flat parent_* fields on Issue
   with a single `parent: Option<ParentIssue>` struct for cleaner API.
 - <csr-id-966b95cc52fc563eb2d4cc5762e7cea1edd8119b/> derive issue provider from RepoContext
   Add jig_toml and global_config fields to RepoContext, loaded once during
   construction. Add convenience methods (issue_provider, issue_provider_with_ref,
   file_provider, linear_provider) that eliminate the repeated
   GlobalConfig::load() + JigToml::load() + make_provider() pattern across
   all call sites.
   
   Updated call sites:
   - daemon/issue_actor: uses RepoContext::from_path() per repo
   - core/spawn: update_issue_status uses RepoContext
   - cli/spawn: uses repo.issue_provider()
   - cli/issues: all subcommands use RepoContext methods
 - <csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/> consolidate auto-spawn into single `auto_spawn_labels` field
   Replace three redundant knobs (`spawn.auto`, `spawn.auto_spawn`,
   `issues.spawn_labels`) with a single `issues.auto_spawn_labels` field.
   Semantics: None = disabled, Some([]) = all issues, Some(["x"]) = filtered.
 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### Test

 - <csr-id-9acbc64d601db75393903d07daca6ae2c16a1b54/> add integration tests for --parent and --remove-parent flags
   Cover the parent flag lifecycle that previously had zero test coverage:
   - create_with_parent: verify --parent injects **Parent:** field
   - update_set_parent: verify --parent adds parent to existing issue
   - update_remove_parent: verify --remove-parent strips the field
   - detail_shows_parent: verify detail view includes parent info
 - <csr-id-3cba2cc100dff6c024c7ba47767447008a6932d1/> add integration tests for automated review pipeline
   Adds 13 integration tests covering the full review lifecycle:
   - Single review cycles (approve and changes_requested)
   - Multi-round convergence (submit → respond → submit)
   - Three-round review cycle with responses at each stage
   - Review count correctly excludes response files
   - Review file structure at max_rounds boundary
   - Response content preserved for next review round
   - Config opt-in/opt-out behavior
   - Review files written to worktree cwd, not repo root
   - CLI validation (missing header, invalid status markers, missing --review flag)
   
   Includes manual test plan as module-level documentation.

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/
   - State file moved to .jig/.state/state.json
   - Auto-migration from .worktrees/ layout on first load
   - jig kill/unregister now removes workers from state entirely
   (instead of archiving them)
   - jig ps auto-cleans stale workers whose tmux windows are gone
   - Hidden directories (.state) are skipped when listing worktrees
   - .jig/.state/ added to .gitignore, .jig/ added to git exclude

## v1.11.1 (2026-03-29)

<csr-id-2bd4d0698a970edb157ecc52fcfc0c2830b375f5/>
<csr-id-e087a62731a90f8aec9897ef658e113a85c0c9f4/>
<csr-id-29da5be67a92fbb7e2cbd7674cb824ca17afb8d5/>
<csr-id-a28704f7e124ad3c35734469ff285d265fc7ccc9/>
<csr-id-e63634542688c53115dac2f70254224545dcb4c8/>
<csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/>
<csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/>
<csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/>
<csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/>
<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-966b95cc52fc563eb2d4cc5762e7cea1edd8119b/>
<csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-2bd4d0698a970edb157ecc52fcfc0c2830b375f5/> bump version to 1.11.1
 - <csr-id-e087a62731a90f8aec9897ef658e113a85c0c9f4/> bump version to 1.11.0
 - <csr-id-29da5be67a92fbb7e2cbd7674cb824ca17afb8d5/> bump version to 1.10.0
 - <csr-id-a28704f7e124ad3c35734469ff285d265fc7ccc9/> bump version to 1.9.0
 - <csr-id-e63634542688c53115dac2f70254224545dcb4c8/> bump version to 1.8.0
 - <csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/> bump version to 1.7.1
 - <csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/> bump version to 1.7.0
 - <csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/> bump version to 1.6.0
 - <csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/> bump version to 1.5.0
 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

 - <csr-id-11a94ff3771fda2aaec8fd036e9a92966cf9688b/> add `jig issues update` command for editing existing issues
   Add a new `update` subcommand to `jig issues` that supports updating
   an existing issue's title, body, priority, labels, and category for
   both file-based and Linear providers.
   
   - LinearClient: add UPDATE_ISSUE_MUTATION and update_issue() method
   - LinearProvider: add update_issue() delegating to client
   - FileProvider: add update_issue() with replace_title/replace_body helpers
   - CLI: add Update variant to IssuesCommand with --title, --body,
   --priority, --label, --category flags
   - Integration tests: 6 new tests covering title, priority, labels,
   body, multiple fields, and no-fields error case
   - Unit tests: 4 new tests for replace_title, replace_body,
   update_issue_title_and_priority, update_issue_labels
 - <csr-id-f313d4c1e1dba5e00cfcc1127eda9adb0f12d599/> move issues to InProgress on spawn to prevent duplicates
   Add `update_status` to the `IssueProvider` trait and call it in both
   spawn code paths (daemon auto-spawn and CLI `jig spawn`) to transition
   issues to InProgress immediately after a worker is launched. This
   prevents `list_spawnable()` from re-picking the same issue across tick
   cycles, since it filters for Planned status only.
 - <csr-id-b9bca75b2c6e59bcbd84c173a3e92b7717362baf/> improve create UX and add docs/tests
   - Make --category optional (defaults to "features" for file provider,
   omitted for Linear to avoid passing a meaningless project name)
   - Document `jig issues create` in Linear integration docs and issues skill
   - Add integration tests for default category, stdin body, and labels
 - <csr-id-e4445e544c26244b44b9051d72d5f34cd0c87da3/> support Linear provider for `jig issues create`
   Wire up `jig issues create` to dispatch through LinearProvider when
   provider = "linear" is configured. Adds a createIssue GraphQL mutation
   to LinearClient with helper queries to resolve team, label, and project
   IDs. The CLI gains a --body flag (inline text or "-" for stdin) that
   works with both file and Linear providers.
 - <csr-id-99f0311db0fb1fcac156aa2f03e8b04bfdd75199/> add jig.local.toml overlay and fix stderr color detection
   Add deep-merge support for jig.local.toml (gitignored) on top of
   jig.toml, allowing machine-specific overrides without touching the
   committed config. Tables merge recursively; scalars and arrays from the
   local file win. `jig init` auto-adds jig.local.toml to .gitignore.
   
   Revamp `jig config` display with source attribution showing where each
   setting originates (jig.toml, jig.local.toml, global config, or default).
   
   Fix colored crate TTY detection: override based on stderr since all jig
   output goes to stderr, preventing silent color suppression when stdout
   is piped.
 - <csr-id-0f9a72d8d53c686ff305a11c961c37083f26a47d/> add Create event and Created status for bare worktrees
   Distinguishes `jig create` worktrees from `jig spawn` workers via a
   positive Create event rather than relying on empty event logs. The daemon
   discovers Created workers (they appear in `jig list`) but takes no
   actions on them — no auto-resume, no nudges, no recovery.
 - <csr-id-205483042ca48167984c4076b0d01bbd81a00f2f/> derive spawn worktree name from issue, use repo base branch in nudges
   - Make `name` optional in `jig spawn` when `--issue` is provided; derive
   worktree name from the issue's branch_name or ID
   - Extract Linear identifiers from branch-format strings (e.g.
   `feature/aut-5044-refactor-foo` → `AUT-5044`)
   - Move derive_worker_name/sanitize_worker_name to shared issues::naming module
   - Pass repo's actual base branch to conflict and bad-commits nudge templates
   instead of hardcoding origin/main
   - Simplify resume to reuse launch(), remove build_resume_command
   - Skip recovery for Initializing workers (still running on-create hooks)
 - <csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/> add commit-msg hook for conventional commit validation
   Adds a commit-msg git hook that validates commit messages against the
   conventional commits spec using the existing parser and jig.toml config.
   Closes the conventional-commits-validation issue.
 - <csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/> add conventional commit validation and examples
   Add parser, configurable validator, and CLI commands:
   - `jig commit validate` validates HEAD, specific revs, stdin, or files
   - `jig commit examples` shows conventional commit reference
   - Configurable via `[commits]` section in jig.toml
   - 16 unit tests + 12 integration tests
 - <csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/> add issue lifecycle commands (create, status, complete, stats)
   Add CLI subcommands to `jig issues` for managing issue state:
   - `jig issues create` creates issues from templates with title, priority, category, labels
   - `jig issues status <id> --status <new>` updates frontmatter status in file issues
   - `jig issues complete <id>` marks issues as complete, with optional --delete
   - `jig issues stats` shows breakdown by status and priority
   
   For Linear issues, status changes go through the API (not file editing).
   Backwards-compatible: `jig issues` with no subcommand still lists/browses.
   Also adds "in-progress" (hyphenated) to loose status parsing.
 - <csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/> add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limiting
   Add three display features to make ps -w a proper dashboard:
   - Nudge cooldown countdown in NUDGE column (e.g. "2/3 (3m12s)")
   - Nudge messages rendered below worker table when delivered
   - Global sync/poll timer footer alongside keybinding hints
   
   Fix two bugs discovered during implementation:
   - Preserve draft PR status on GitHub API errors so nudges aren't
   silently suppressed for known-draft PRs
   - Throttle GitHub API requests to once per 60s per worker, aligned
   with gh --cache 60s TTL, reducing API pressure ~30x
 - <csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/> add daemon crash recovery and worker resume
   - Install SIGTERM/SIGINT handler for graceful daemon shutdown
   - Record Started/Stopped lifecycle events in daemon.jsonl
   - Detect unclean shutdown on next startup (missing Stopped event)
   - Auto-recover orphaned workers on daemon startup via Worktree::resume()
   - Add `jig resume <name>` CLI command for manual worker recovery
   - Detect dead tmux windows during steady-state ticks and auto-resume
   - Add `auto_recover` global config option (default: true) for opt-out
   - Wire Action::Restart to use recovery::try_resume_worker()
   - Add integration tests for jig resume command
 - <csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/> show nudge state in jig ps output
   Add NUDGE column between STATE and COMMITS in the ps table.
   Displays nudge count as count/max (e.g. 2/3), with color coding:
   grey dash for zero, yellow for in-progress, red for exhausted.
   
   Adds max_nudges field to WorkerDisplayInfo so the UI can render
   the denominator from the resolved health config.
 - <csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/> add per-repo nudge configuration in jig.toml
   Add [health] section to jig.toml supporting per-repo overrides of
   silence_threshold_seconds and max_nudges, plus per-nudge-type
   [health.nudge.<type>] sections with independent max and
   cooldown_seconds settings.
   
   Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]
   > global config > defaults. When cooldown_seconds is not set, falls
   back to silence_threshold_seconds.
   
   - Add RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structs
   - Add ResolvedNudgeConfig with resolver for per-type config
   - Thread resolved config through nudge classify, dispatch, and execute
   - Apply per-type cooldown to both idle/stalled nudges and PR nudges
   - Display effective nudge config in `jig config show`
   - Fixes PR nudge burst bug by enforcing per-type cooldowns
 - <csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/> add jig home command to print base repo root
   Adds `jig home` (alias `jig h`) that prints the base repository root
   path, enabling `cd $(jig home)` navigation from worktrees.
 - <csr-id-df0a3be811b27f8afce047bd088cad410d09e081/> communicate worker initialization state and on-create failures
   Add Initializing event type and worker status to make the worker
   lifecycle visible during setup. When the daemon auto-spawns a worker,
   it now registers the worker as Initializing before running the
   on-create hook, then transitions to Spawned on success or Failed on
   hook failure.
 - <csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/> show auto-spawn config in `jig config` and add `jig issues --auto`
   Display auto-spawn settings (enabled, auto-start, max workers, poll
   interval, spawn labels) in `jig config show` with source attribution.
   Add `--auto` flag to `jig issues` to filter to only daemon-eligible
   auto-spawn candidates using the existing `list_spawnable` method.
 - <csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/> use checkmark instead of asterisk for AUTO column
   Change the AUTO column indicator in `jig issues` from `*` to `✓`
   for better readability.
 - <csr-id-bd1a1faeca5a7634224bef836154791819b4903b/> use checkmark instead of asterisk for AUTO column
   Change the AUTO column indicator in `jig issues` from `*` to `✓`
   for better readability.
 - <csr-id-462f05eaf29929899631125c733738cd8f93e558/> move auto-spawn to background thread to keep ps -w responsive
   The on-create hook (e.g. pnpm install) was running synchronously on
   the tick thread, freezing the ps --watch UI for the entire duration.
   Introduces a spawn_actor following the same pattern as prune_actor,
   issue_actor, etc. The tick now sends spawnable issues to the background
   thread and drains results on the next tick.
   
   Also adds:
   - Spawning worker names shown below the ps table during setup
   - WorkerStatus::Initializing variant for future use
   - spawn_labels config in jig.toml
   - Three new issues (config-show-auto-spawn, worker-initializing-state,
   auto-column-checkmark)
 - <csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/> add labels field for issue tagging and filtering
   Add `labels: Vec<String>` to Issue and IssueFilter types. Linear
   provider now passes all label names through from GraphQL (auto field
   derivation unchanged). File provider parses `**Labels:**` comma-separated
   frontmatter. CLI gains `--label/-l` flag for filtering (all must match).
   Shell completions updated for bash, zsh, and fish.
 - <csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/> add shared UI module with consistent formatting and --plain flag
   Expand ui.rs into a centralized formatting module with:
   - Status symbol constants (✓, →, ✗, !)
   - Formatted output helpers (success, progress, failure, warning, detail, header)
   - Color helpers (highlight, bold, dim) that respect plain mode
   - Table builder helper (new_table) for consistent table creation
   - Global --plain flag for scriptable output (no colors, no decorations)
   - Error display with cause chain formatting
   
   Migrate all 20 command files from inline colored::Colorize calls to
   shared ui:: helpers. Add --plain support to list, repos, and issues
   commands with tab-separated output for piping.
 - <csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/> add AUTO column to `jig issues` table output
   Show a green dot indicator for issues tagged for auto-spawn, making it
   visible at a glance whether file-provider Auto flag or Linear jig-auto
   label is set.
 - <csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/> support -g/--global flag for attaching from anywhere
   Add run_global implementation to the Attach command so users can attach
   to a worktree from outside the owning repo using `jig attach <name> -g`.
   Resolves the owning repo via GlobalCtx::repo_for_worktree.
 - <csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/> jig init --audit launches agent in tmux to populate docs
   --audit now spawns the configured agent in a jig-init:<repo> tmux
   session with the audit prompt instead of just printing instructions.
   --backup enhances the prompt to reference .backup/ files. --audit
   accepts an optional string for extra instructions.
 - <csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/> block auto-spawn on unresolved dependencies
   Add `is_spawnable_with_deps()` to IssueProvider trait that checks all
   depends_on entries resolve to Complete before allowing spawn. Applied in
   both FileProvider and LinearProvider's list_spawnable(). Also adds
   --blocked/--unblocked flags to `jig issues` CLI for filtering.
 - <csr-id-057e8dc3675610e75e826910d051774f32f63cee/> group workers by repo in `jig ps -g` output
   Add `repo` field to `WorkerDisplayInfo` and render grouped tables with
   bold repo headers when running in global mode (`jig ps -g` / `jig ps -gw`).
   Local `jig ps` output is unchanged.
 - <csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/> daemon periodically prunes stale worktrees
   Workers in terminal state (merged/archived/failed) with dead tmux
   sessions now get their git worktrees, event logs, and global state
   entries cleaned up automatically. Prune runs every 120s during watch
   mode. Pruned workers are reported in the tick status and log view.
   
   Also includes snake_case fixes for auto-spawn-filtering ticket.
 - <csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/> default table view for `jig ls` and pretty grouped `jig ls -g`
   - `jig ls` now shows a table with name, branch, and commits ahead
   - `jig ls -g` shows tables grouped by repo with bold headers
   - Add `--plain/-p` flag for bare name output (old behavior)
   - Shell completions use `--plain` and fall back to `-gp` outside a repo
   - Branch column only shown when it differs from worktree name
 - <csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/> show draft vs review state, document PR nudge behavior
   Workers with draft PRs now show "draft" (blue) in the STATE column
   instead of "review" (cyan). This makes it visually clear which workers
   will receive PR nudges (draft) vs which are in human review (non-draft).
   
   Add PR Nudges section to daemon docs explaining the draft/non-draft
   nudge policy and what each health check means.
 - <csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/> unify daemon/ps tick loops and add log toggle to watch mode
   Extract run_with() callback API from daemon so ps --watch shares the
   same setup code path instead of duplicating Daemon/Notifier/TmuxClient
   construction. The callback controls inter-tick delay and can signal
   stop, which enables keypress handling during the sleep window.
   
   Add log view toggle to watch mode: press 'l' to see timestamped daemon
   activity (nudges fired, PR check results, errors), 't' to switch back
   to the table, 'q' to quit cleanly. Uses crossterm raw mode with 100ms
   poll intervals for responsive input.
   
   Also allows spawned workers to transition to stalled (previously
   Spawned status was excluded from silence detection).
 - <csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/> surface PR health in ps --watch display
   Add a HEALTH column to the watch table showing per-worker PR check
   results (ci, conflicts, reviews, commits) so problems are visible at a
   glance without needing RUST_LOG=debug. Upgrade silent debug-level PR
   errors to info-level logging.
 - <csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/> add --base flag to spawn and create for custom branch base
   Allow overriding the default base branch (from jig.toml) per-command
   with --base/-b. Includes shell completions for branch names across
   bash, zsh, and fish. Also fixes spawn status message to show the
   actual base branch used instead of the current branch.
 - <csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/> wire issues into spawn pipeline with --issue flag
   Add `jig spawn --issue <id>` to resolve file-based issues and use their
   body as Claude context. Thread issue_ref through the full pipeline:
   spawn CLI → register() → Spawn event → WorkerState reducer → daemon
   workers.json → ps watch table.
   
   Also adds:
   - `jig issues` CLI command with --ids flag for scripting
   - IssuesConfig in jig.toml for configurable issues directory
   - ISSUE column in ps --watch table (shortened last path segment)
   - Shell completions for --issue in bash, zsh, and fish
   - issue_ref tests in reducer and daemon roundtrip
 - <csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/> add watch mode to ps command for live dashboard
   `jig ps --watch` clears and refreshes the worker table every 2s.
   Shows enriched state from event logs alongside tmux status:
   - TMUX column (●/○/✗) for session liveness
   - STATE column from event-derived WorkerStatus
   - NUDGES count and PR number from event log
   - Configurable interval: `jig ps -w 5` for 5s refresh
 - <csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/> add daemon loop to orchestrate event-driven pipeline
   The missing conductor: `jig daemon` runs a periodic loop that:
   - Discovers workers by scanning event log directories
   - Replays events to derive current WorkerState per worker
   - Compares old vs new state to dispatch actions
   - Executes nudges via tmux and notifications via hooks
   - Persists state to workers.json between ticks
   
   Supports --once for single-pass mode and --interval for tuning.
 - <csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/> add git hook management (install, uninstall, handlers)
   Implements the git-hooks epic (tickets 0-4):
   - Hook wrapper templates that chain jig logic with user hooks
   - Registry tracking installed hooks at jig-hooks.json
   - Idempotent init with backup/restore of existing hooks
   - Post-commit/merge handlers that emit events to worker logs
   - Uninstall with rollback to original user hooks
 - <csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/> expand WorkerStatus with event-driven states
   Add Idle, WaitingInput, Stalled variants. Make all variants unit types
   (remove associated data from WaitingReview/Failed). Add needs_attention(),
   is_active(), is_terminal(), from_legacy() methods. Snake_case serialization.
 - <csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/> add event log format and Claude Code hooks
   Implement event-system tickets 1 and 2:
   - Event schema with typed EventType enum and flat JSONL serialization
   - EventLog append-only reader/writer with per-worker JSONL files
   - Claude Code hook templates (PostToolUse, Notification, Stop)
   - `jig hooks install-claude` CLI command to install hooks to ~/.claude/hooks/
 - <csr-id-82c654ab1137ec963121638f6741617c59ee0c04/> add global state infrastructure for cross-repo aggregation
   Introduces ~/.config/jig/ directory structure with structured TOML config,
   aggregated JSON worker state, and event log directories for the event-driven
   pipeline. Ensures global dirs are created at CLI startup.
 - <csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/> introduce RepoContext and thread repo state through all operations
   Derive repo_root, worktrees_dir, git_common_dir, base_branch, and
   session_name once at startup via RepoContext::from_cwd(), eliminating
   redundant git subprocess calls (e.g. spawn called get_base_repo() 8x).
   OpContext now holds Option<RepoContext>, and all jig-core functions
   accept &RepoContext instead of re-deriving from cwd. Also adds repo
   registry for global mode auto-registration, removes dead spawn::kill(),
   and updates docs/patterns/issue status.
 - <csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/> implement smart jig update command
   Rewrite update command to:
   - Detect installation method (script, cargo, source, unknown)
   - Check latest version from GitHub releases API
   - Auto-update for script installations (~/.local/bin)
   - Prompt dev builds to install release binaries
   - Offer cleanup of old cargo bin after source build updates
   - Add --force flag to skip version check
 - <csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/> prettify jig ps with Op pattern and comfy-table
   Introduce the Op trait to separate command logic from presentation.
   Rewrite `jig ps` as the first adopter: ops return typed data, Display
   impls own all formatting via comfy-table with terminal-width-aware
   column layout and color-coded status indicators.
   
   - Add Op trait in crates/jig-cli/src/op.rs
   - Rewrite ps command with PsOutput, PsError, and Op impl
   - Add comfy-table dependency for dynamic table rendering
   - Update main.rs dispatch to use Op::execute()
   - Add docs/ui/STDOUT-FORMATTING.md documenting the pattern
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
 - <csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/> restructure issue tracking with categories and templates
   - Add directory-based issue organization (epics/, features/, bugs/, chores/)
   - Add issue templates (_templates/): standalone.md, epic-index.md, ticket.md
   - Create plan-and-execute epic for orchestration vision
   - Update issues/README.md with comprehensive documentation
   - Update /issues skill for new directory structure
   - Remove old flat issue files and _template.md
   - Add .backup/ to .gitignore
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
 - <csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/> improve backup, audit prompt, and review skill
   - Backup now copies files to .backup/ directory preserving path structure
   - Audit prompt is detailed and opinionated about what to fill in each doc
   - Review skill now checks for documentation and skills updates
 - <csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/> upgrade jig init scaffolding to language-agnostic skeletons
   - Move issue-tracking.md to issues/README.md, fix "wt" → "jig"
   - Rename skills/jig → skills/spawn for consistency
   - Remove name: field from skill frontmatter
   - Add skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.md
   - Expand docs/index.md as documentation hub
   - Make CLAUDE.md template a skeleton with guidance comments
   - Upgrade settings.json: add $schema, ask tier for destructive ops, better secret patterns
   - Add issues/_template.md ticket template
 - <csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/> add Claude Code skills and simplify permissions
   - Add skills for check, draft, issues, review, and spawn commands
   - Simplify .claude/settings.json using wildcard permissions
   - Add jig.toml with spawn auto-configuration
   - Fix formatting in init.rs
 - <csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/> use actual templates for jig init instead of bare-bones placeholders
   - Embed templates from templates/ directory using include_str!
   - Add all 5 skills: check, draft, issues, review, spawn
   - Expand permissions to cover tools used by skills
   - Set spawn.auto = true by default
   - Use exec() on Unix for --audit flag (full terminal control)
   
   The init command now creates a complete scaffolding that matches
   the documentation, instead of empty placeholder comments.
 - <csr-id-3a78670c102178f25db9dc4020b534370fc36f84/> add --audit flag to init command that launches Claude interactively
   Uses exec() on Unix to replace the current process with Claude Code,
   giving it full terminal control for interactive documentation audit.
 - <csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/> add shell-setup command and fix shell completions
   - Add `jig shell-setup` command to automatically configure shell integration
   - Detects user's shell from $SHELL
   - Finds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)
   - Adds eval line with markers for easy identification
   - Places integration after PATH setup when possible
   - Supports --dry-run flag to preview changes
   
   - Rewrite shell completions with dynamic worktree completion
   - `jig open/attach/review/merge/kill/status <TAB>` shows actual worktrees
   - Context-aware completions for all subcommands
   - Simplified zsh completion using _arguments -C
   
   - Update docs/usage/shell-integration.md
   - Add quick setup section for shell-setup command
   - Add troubleshooting section for common issues
   - Remove stale `sc` alias references (legacy from "scribe" name)
 - <csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/> rewrite health check to validate repo setup and agent scaffolding
   Replace terminal-detection-focused health check with structured validation
   of system deps (git, tmux, claude), repository config (jig.toml, base
   branch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).
   Remove unused jq/gh dependency checks and dead required field. Exit
   non-zero when checks fail.
 - <csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/> add shell completions for bash, zsh, and fish
   Shell completions are now emitted alongside the shell wrapper function
   in `jig shell-init`. Completions cover all subcommands, aliases,
   per-command flags, nested config subcommands, and dynamic worktree
   name completion via `command jig list`.

### Bug Fixes

 - <csr-id-6d4388a490b70bb3401aa84bf936c2cfb9239e3a/> cherry-pick bug fixes from review experiment branch
   - Skip PENDING draft reviews in get_reviews() to prevent false nudges
   while a reviewer is still writing comments
   - Add enforce_styling() to worker tables so comfy-table renders ANSI
   colors regardless of terminal detection
   - Suppress review nudges after dev pushes (dev_pushed_after_reviews
   helper) — if the developer already pushed commits after the latest
   review feedback, the ball is in the reviewer's court
   - Add head_sha field to PrStateInfo for commit-aware operations
   - Make gh_api() pub(crate) to support the new helper
 - <csr-id-47d683b540b09d52770902df1a3d47e501372ba9/> consolidate spawn codepaths to ensure update_status in watch mode
   The watch-mode daemon (spawn_actor::spawn_single) was missing the
   update_status(InProgress) call after spawning, causing issues to be
   re-spawned on every tick. This was because three near-duplicate spawn
   codepaths existed (CLI, daemon blocking, daemon watch) and only two
   had the status update.
   
   Extract spawn_worker_for_issue() and update_issue_status() into the
   shared spawn module so all three codepaths use a single authoritative
   implementation. This eliminates ~120 lines of duplication and makes it
   impossible to miss a step in any codepath.
 - <csr-id-c5b52f59055c0f95498ef657685a18405bf6b515/> consolidate spawn codepaths to ensure update_status in watch mode
   The watch-mode daemon (spawn_actor::spawn_single) was missing the
   update_status(InProgress) call after spawning, causing issues to be
   re-spawned on every tick. This was because three near-duplicate spawn
   codepaths existed (CLI, daemon blocking, daemon watch) and only two
   had the status update.
   
   Extract spawn_worker_for_issue() and update_issue_status() into the
   shared spawn module so all three codepaths use a single authoritative
   implementation. This eliminates ~120 lines of duplication and makes it
   impossible to miss a step in any codepath.
 - <csr-id-0e71bb2f733c8d97cffe7dc2c1826c3aa61da2a9/> support -g/--global flag and show provider-specific details
   `jig config` and `jig config -g` now work outside a git repo by showing
   global config. Issues section displays Linear profile details (profile,
   team, projects, assignee, labels) when provider is "linear", and directory
   when provider is "file".
 - <csr-id-8015b3097d63862324524ac9e88e2bd411982839/> add .jig/ to .gitignore during init
   Previously .jig/ was only added to .git/info/exclude which is
   local to each clone. This meant hooks and state could leak into
   commits in fresh clones or CI. Now `jig init` adds .jig/ alongside
   jig.local.toml in .gitignore so the ignore is shared across clones.
 - <csr-id-847c9ba0e63c6aeb5a46fead6529eca9b8976465/> use `jig -g init` instead of broken `--global` flag on init
   The `--global` flag conflicts with the top-level `-g/--global` CLI flag.
   Implement `run_global()` on Init so `jig -g init` scaffolds the global
   config correctly.
 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter
   - Relation mapping was inverted: "blocks" → "is_blocked_by" so
   dependency checking now correctly identifies blocked issues
   - Remove unused SearchData type
   
   Improve issues command UX:
   - Hide completed issues by default (use --all to include them)
   - Interactive mode (-i) uses alternate screen buffer like less/git diff
   - Add ui::with_alternate_screen() reusable helper
   - Interactive mode: scrolling, auto indicator, title truncation, G/g nav
 - <csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/> accept trailing args in git hook subcommands
   Git passes arguments to hooks (e.g. post-merge receives a squash flag
   "0" or "1"), which the hook wrapper forwards via "$@". The CLI
   subcommands rejected these unexpected args. Add trailing_var_arg to
   PostCommit, PostMerge, and PreCommit to accept and ignore them.
 - <csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/> output cd command from jig home instead of bare path
   Matches the pattern used by `jig open` and `jig exit` — outputs
   `cd '/path'` to stdout for shell eval, not just the path.
 - <csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/> recover from stale git worktree registrations on spawn and prune
   When a worktree directory is removed but git still tracks the entry,
   `git worktree add` fails with "missing but already registered". Now
   create_worktree runs `git worktree prune` first, and prune_actor
   handles the missing-directory case instead of skipping cleanup.
   
   Also extracts prune_actor into its own module and adds urgent issue
   to replace git CLI shelling with git2.
 - <csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/> add issues command to shell completions
   The `issues` command was missing from all three shells' command lists
   and had no flag/argument completions. Adds command entry, issue ID
   positional completions, and flag completions (status, priority,
   category, interactive, ids) for bash, zsh, and fish.
 - <csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/> use if-let instead of unwrap to satisfy clippy
 - <csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/> daemon PR discovery, tmux targeting, and nudge delivery
   - Add proactive PR discovery: daemon queries GitHub for open PRs on
   worker branches when pr_url is unknown, emits PrOpened events to
   make state durable across restarts
   - Create per-repo GitHub clients via registry path lookup instead of
   ambient remote detection (fixes multi-repo daemon)
   - Extract real branch name from spawn events for tmux window lookup
   (spawn creates windows with slashes, e.g. feature/foo, not dashes)
   - Run all four PR checks (CI, conflicts, reviews, commits) on open PRs
   - Nudge on every tick, not just state transitions, so polling daemon
   retries delivery until max_nudges
   - Collapse multiline nudge templates to single line before tmux send
   to prevent premature submission in TUIs
   - Fix tracing init: RUST_LOG now properly overrides default warn level
   - Add stderr tick summary in continuous daemon mode for visibility
   without RUST_LOG
   - Add debug logging for tmux window misses and notification pipeline
 - <csr-id-378031a0afe019f57edc9bae469bf8168e05de29/> register Claude hooks in settings.json, add kill --all and nuke
   Claude Code hooks were installed as scripts but never registered in
   ~/.claude/settings.json, so they never fired. Now jig init registers
   them properly. Also fixes: hook templates read JSON from stdin (not
   env vars), spawned workers no longer nudged as stalled, event logs
   reset on respawn, row ordering stabilized in ps --watch, kill/unregister
   cleans up event logs, and nuke command added for full repo cleanup.
 - <csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/> address review findings and wire up event pipeline end-to-end
   Fix 6 issues from code review: UTF-8 safe truncate, stable status
   serialization via as_str/from_legacy, stuck nudge sends message after
   auto-approve, notification errors logged, branch names URL-encoded,
   tmux commands check exit status.
   
   Wire up missing pipeline links: jig spawn emits Spawn event, jig init
   auto-installs git+Claude hooks (idempotent on re-run), ps --watch runs
   daemon tick on each refresh for integrated orchestration.
   
   Add docs/daemon.md with background service setup for launchd, systemd,
   OpenRC, and generic nohup.
 - <csr-id-a41b92cb77141469539658c133da79f79f714452/> remove unnecessary return statement
 - <csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/> make --audit print command instead of trying to launch claude
   Spawning claude programmatically was causing terminal issues and hangs.
   Now --audit just prints the command for the user to run manually.
 - <csr-id-196774225c8eba52fdb9382f98418ecf82c48567/> prevent shell-setup from corrupting shell config files
   The previous byte-slicing approach in find_path_line_end() calculated
   offsets incorrectly because lines() strips newlines but the code assumed
   +1 byte per line. This could corrupt or truncate config files.

### Other

 - <csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/> fmt
 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-966b95cc52fc563eb2d4cc5762e7cea1edd8119b/> derive issue provider from RepoContext
   Add jig_toml and global_config fields to RepoContext, loaded once during
   construction. Add convenience methods (issue_provider, issue_provider_with_ref,
   file_provider, linear_provider) that eliminate the repeated
   GlobalConfig::load() + JigToml::load() + make_provider() pattern across
   all call sites.
   
   Updated call sites:
   - daemon/issue_actor: uses RepoContext::from_path() per repo
   - core/spawn: update_issue_status uses RepoContext
   - cli/spawn: uses repo.issue_provider()
   - cli/issues: all subcommands use RepoContext methods
 - <csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/> consolidate auto-spawn into single `auto_spawn_labels` field
   Replace three redundant knobs (`spawn.auto`, `spawn.auto_spawn`,
   `issues.spawn_labels`) with a single `issues.auto_spawn_labels` field.
   Semantics: None = disabled, Some([]) = all issues, Some(["x"]) = filtered.
 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/
   - State file moved to .jig/.state/state.json
   - Auto-migration from .worktrees/ layout on first load
   - jig kill/unregister now removes workers from state entirely
   (instead of archiving them)
   - jig ps auto-cleans stale workers whose tmux windows are gone
   - Hidden directories (.state) are skipped when listing worktrees
   - .jig/.state/ added to .gitignore, .jig/ added to git exclude

## v1.11.0 (2026-03-23)

<csr-id-e087a62731a90f8aec9897ef658e113a85c0c9f4/>
<csr-id-29da5be67a92fbb7e2cbd7674cb824ca17afb8d5/>
<csr-id-a28704f7e124ad3c35734469ff285d265fc7ccc9/>
<csr-id-e63634542688c53115dac2f70254224545dcb4c8/>
<csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/>
<csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/>
<csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/>
<csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/>
<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-e087a62731a90f8aec9897ef658e113a85c0c9f4/> bump version to 1.11.0
 - <csr-id-29da5be67a92fbb7e2cbd7674cb824ca17afb8d5/> bump version to 1.10.0
 - <csr-id-a28704f7e124ad3c35734469ff285d265fc7ccc9/> bump version to 1.9.0
 - <csr-id-e63634542688c53115dac2f70254224545dcb4c8/> bump version to 1.8.0
 - <csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/> bump version to 1.7.1
 - <csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/> bump version to 1.7.0
 - <csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/> bump version to 1.6.0
 - <csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/> bump version to 1.5.0
 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

 - <csr-id-11a94ff3771fda2aaec8fd036e9a92966cf9688b/> add `jig issues update` command for editing existing issues
   Add a new `update` subcommand to `jig issues` that supports updating
   an existing issue's title, body, priority, labels, and category for
   both file-based and Linear providers.
   
   - LinearClient: add UPDATE_ISSUE_MUTATION and update_issue() method
   - LinearProvider: add update_issue() delegating to client
   - FileProvider: add update_issue() with replace_title/replace_body helpers
   - CLI: add Update variant to IssuesCommand with --title, --body,
   --priority, --label, --category flags
   - Integration tests: 6 new tests covering title, priority, labels,
   body, multiple fields, and no-fields error case
   - Unit tests: 4 new tests for replace_title, replace_body,
   update_issue_title_and_priority, update_issue_labels
 - <csr-id-f313d4c1e1dba5e00cfcc1127eda9adb0f12d599/> move issues to InProgress on spawn to prevent duplicates
   Add `update_status` to the `IssueProvider` trait and call it in both
   spawn code paths (daemon auto-spawn and CLI `jig spawn`) to transition
   issues to InProgress immediately after a worker is launched. This
   prevents `list_spawnable()` from re-picking the same issue across tick
   cycles, since it filters for Planned status only.
 - <csr-id-b9bca75b2c6e59bcbd84c173a3e92b7717362baf/> improve create UX and add docs/tests
   - Make --category optional (defaults to "features" for file provider,
   omitted for Linear to avoid passing a meaningless project name)
   - Document `jig issues create` in Linear integration docs and issues skill
   - Add integration tests for default category, stdin body, and labels
 - <csr-id-e4445e544c26244b44b9051d72d5f34cd0c87da3/> support Linear provider for `jig issues create`
   Wire up `jig issues create` to dispatch through LinearProvider when
   provider = "linear" is configured. Adds a createIssue GraphQL mutation
   to LinearClient with helper queries to resolve team, label, and project
   IDs. The CLI gains a --body flag (inline text or "-" for stdin) that
   works with both file and Linear providers.
 - <csr-id-99f0311db0fb1fcac156aa2f03e8b04bfdd75199/> add jig.local.toml overlay and fix stderr color detection
   Add deep-merge support for jig.local.toml (gitignored) on top of
   jig.toml, allowing machine-specific overrides without touching the
   committed config. Tables merge recursively; scalars and arrays from the
   local file win. `jig init` auto-adds jig.local.toml to .gitignore.
   
   Revamp `jig config` display with source attribution showing where each
   setting originates (jig.toml, jig.local.toml, global config, or default).
   
   Fix colored crate TTY detection: override based on stderr since all jig
   output goes to stderr, preventing silent color suppression when stdout
   is piped.
 - <csr-id-0f9a72d8d53c686ff305a11c961c37083f26a47d/> add Create event and Created status for bare worktrees
   Distinguishes `jig create` worktrees from `jig spawn` workers via a
   positive Create event rather than relying on empty event logs. The daemon
   discovers Created workers (they appear in `jig list`) but takes no
   actions on them — no auto-resume, no nudges, no recovery.
 - <csr-id-205483042ca48167984c4076b0d01bbd81a00f2f/> derive spawn worktree name from issue, use repo base branch in nudges
   - Make `name` optional in `jig spawn` when `--issue` is provided; derive
   worktree name from the issue's branch_name or ID
   - Extract Linear identifiers from branch-format strings (e.g.
   `feature/aut-5044-refactor-foo` → `AUT-5044`)
   - Move derive_worker_name/sanitize_worker_name to shared issues::naming module
   - Pass repo's actual base branch to conflict and bad-commits nudge templates
   instead of hardcoding origin/main
   - Simplify resume to reuse launch(), remove build_resume_command
   - Skip recovery for Initializing workers (still running on-create hooks)
 - <csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/> add commit-msg hook for conventional commit validation
   Adds a commit-msg git hook that validates commit messages against the
   conventional commits spec using the existing parser and jig.toml config.
   Closes the conventional-commits-validation issue.
 - <csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/> add conventional commit validation and examples
   Add parser, configurable validator, and CLI commands:
   - `jig commit validate` validates HEAD, specific revs, stdin, or files
   - `jig commit examples` shows conventional commit reference
   - Configurable via `[commits]` section in jig.toml
   - 16 unit tests + 12 integration tests
 - <csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/> add issue lifecycle commands (create, status, complete, stats)
   Add CLI subcommands to `jig issues` for managing issue state:
   - `jig issues create` creates issues from templates with title, priority, category, labels
   - `jig issues status <id> --status <new>` updates frontmatter status in file issues
   - `jig issues complete <id>` marks issues as complete, with optional --delete
   - `jig issues stats` shows breakdown by status and priority
   
   For Linear issues, status changes go through the API (not file editing).
   Backwards-compatible: `jig issues` with no subcommand still lists/browses.
   Also adds "in-progress" (hyphenated) to loose status parsing.
 - <csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/> add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limiting
   Add three display features to make ps -w a proper dashboard:
   - Nudge cooldown countdown in NUDGE column (e.g. "2/3 (3m12s)")
   - Nudge messages rendered below worker table when delivered
   - Global sync/poll timer footer alongside keybinding hints
   
   Fix two bugs discovered during implementation:
   - Preserve draft PR status on GitHub API errors so nudges aren't
   silently suppressed for known-draft PRs
   - Throttle GitHub API requests to once per 60s per worker, aligned
   with gh --cache 60s TTL, reducing API pressure ~30x
 - <csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/> add daemon crash recovery and worker resume
   - Install SIGTERM/SIGINT handler for graceful daemon shutdown
   - Record Started/Stopped lifecycle events in daemon.jsonl
   - Detect unclean shutdown on next startup (missing Stopped event)
   - Auto-recover orphaned workers on daemon startup via Worktree::resume()
   - Add `jig resume <name>` CLI command for manual worker recovery
   - Detect dead tmux windows during steady-state ticks and auto-resume
   - Add `auto_recover` global config option (default: true) for opt-out
   - Wire Action::Restart to use recovery::try_resume_worker()
   - Add integration tests for jig resume command
 - <csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/> show nudge state in jig ps output
   Add NUDGE column between STATE and COMMITS in the ps table.
   Displays nudge count as count/max (e.g. 2/3), with color coding:
   grey dash for zero, yellow for in-progress, red for exhausted.
   
   Adds max_nudges field to WorkerDisplayInfo so the UI can render
   the denominator from the resolved health config.
 - <csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/> add per-repo nudge configuration in jig.toml
   Add [health] section to jig.toml supporting per-repo overrides of
   silence_threshold_seconds and max_nudges, plus per-nudge-type
   [health.nudge.<type>] sections with independent max and
   cooldown_seconds settings.
   
   Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]
   > global config > defaults. When cooldown_seconds is not set, falls
   back to silence_threshold_seconds.
   
   - Add RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structs
   - Add ResolvedNudgeConfig with resolver for per-type config
   - Thread resolved config through nudge classify, dispatch, and execute
   - Apply per-type cooldown to both idle/stalled nudges and PR nudges
   - Display effective nudge config in `jig config show`
   - Fixes PR nudge burst bug by enforcing per-type cooldowns
 - <csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/> add jig home command to print base repo root
   Adds `jig home` (alias `jig h`) that prints the base repository root
   path, enabling `cd $(jig home)` navigation from worktrees.
 - <csr-id-df0a3be811b27f8afce047bd088cad410d09e081/> communicate worker initialization state and on-create failures
   Add Initializing event type and worker status to make the worker
   lifecycle visible during setup. When the daemon auto-spawns a worker,
   it now registers the worker as Initializing before running the
   on-create hook, then transitions to Spawned on success or Failed on
   hook failure.
 - <csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/> show auto-spawn config in `jig config` and add `jig issues --auto`
   Display auto-spawn settings (enabled, auto-start, max workers, poll
   interval, spawn labels) in `jig config show` with source attribution.
   Add `--auto` flag to `jig issues` to filter to only daemon-eligible
   auto-spawn candidates using the existing `list_spawnable` method.
 - <csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/> use checkmark instead of asterisk for AUTO column
   Change the AUTO column indicator in `jig issues` from `*` to `✓`
   for better readability.
 - <csr-id-bd1a1faeca5a7634224bef836154791819b4903b/> use checkmark instead of asterisk for AUTO column
   Change the AUTO column indicator in `jig issues` from `*` to `✓`
   for better readability.
 - <csr-id-462f05eaf29929899631125c733738cd8f93e558/> move auto-spawn to background thread to keep ps -w responsive
   The on-create hook (e.g. pnpm install) was running synchronously on
   the tick thread, freezing the ps --watch UI for the entire duration.
   Introduces a spawn_actor following the same pattern as prune_actor,
   issue_actor, etc. The tick now sends spawnable issues to the background
   thread and drains results on the next tick.
   
   Also adds:
   - Spawning worker names shown below the ps table during setup
   - WorkerStatus::Initializing variant for future use
   - spawn_labels config in jig.toml
   - Three new issues (config-show-auto-spawn, worker-initializing-state,
   auto-column-checkmark)
 - <csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/> add labels field for issue tagging and filtering
   Add `labels: Vec<String>` to Issue and IssueFilter types. Linear
   provider now passes all label names through from GraphQL (auto field
   derivation unchanged). File provider parses `**Labels:**` comma-separated
   frontmatter. CLI gains `--label/-l` flag for filtering (all must match).
   Shell completions updated for bash, zsh, and fish.
 - <csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/> add shared UI module with consistent formatting and --plain flag
   Expand ui.rs into a centralized formatting module with:
   - Status symbol constants (✓, →, ✗, !)
   - Formatted output helpers (success, progress, failure, warning, detail, header)
   - Color helpers (highlight, bold, dim) that respect plain mode
   - Table builder helper (new_table) for consistent table creation
   - Global --plain flag for scriptable output (no colors, no decorations)
   - Error display with cause chain formatting
   
   Migrate all 20 command files from inline colored::Colorize calls to
   shared ui:: helpers. Add --plain support to list, repos, and issues
   commands with tab-separated output for piping.
 - <csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/> add AUTO column to `jig issues` table output
   Show a green dot indicator for issues tagged for auto-spawn, making it
   visible at a glance whether file-provider Auto flag or Linear jig-auto
   label is set.
 - <csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/> support -g/--global flag for attaching from anywhere
   Add run_global implementation to the Attach command so users can attach
   to a worktree from outside the owning repo using `jig attach <name> -g`.
   Resolves the owning repo via GlobalCtx::repo_for_worktree.
 - <csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/> jig init --audit launches agent in tmux to populate docs
   --audit now spawns the configured agent in a jig-init:<repo> tmux
   session with the audit prompt instead of just printing instructions.
   --backup enhances the prompt to reference .backup/ files. --audit
   accepts an optional string for extra instructions.
 - <csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/> block auto-spawn on unresolved dependencies
   Add `is_spawnable_with_deps()` to IssueProvider trait that checks all
   depends_on entries resolve to Complete before allowing spawn. Applied in
   both FileProvider and LinearProvider's list_spawnable(). Also adds
   --blocked/--unblocked flags to `jig issues` CLI for filtering.
 - <csr-id-057e8dc3675610e75e826910d051774f32f63cee/> group workers by repo in `jig ps -g` output
   Add `repo` field to `WorkerDisplayInfo` and render grouped tables with
   bold repo headers when running in global mode (`jig ps -g` / `jig ps -gw`).
   Local `jig ps` output is unchanged.
 - <csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/> daemon periodically prunes stale worktrees
   Workers in terminal state (merged/archived/failed) with dead tmux
   sessions now get their git worktrees, event logs, and global state
   entries cleaned up automatically. Prune runs every 120s during watch
   mode. Pruned workers are reported in the tick status and log view.
   
   Also includes snake_case fixes for auto-spawn-filtering ticket.
 - <csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/> default table view for `jig ls` and pretty grouped `jig ls -g`
   - `jig ls` now shows a table with name, branch, and commits ahead
   - `jig ls -g` shows tables grouped by repo with bold headers
   - Add `--plain/-p` flag for bare name output (old behavior)
   - Shell completions use `--plain` and fall back to `-gp` outside a repo
   - Branch column only shown when it differs from worktree name
 - <csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/> show draft vs review state, document PR nudge behavior
   Workers with draft PRs now show "draft" (blue) in the STATE column
   instead of "review" (cyan). This makes it visually clear which workers
   will receive PR nudges (draft) vs which are in human review (non-draft).
   
   Add PR Nudges section to daemon docs explaining the draft/non-draft
   nudge policy and what each health check means.
 - <csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/> unify daemon/ps tick loops and add log toggle to watch mode
   Extract run_with() callback API from daemon so ps --watch shares the
   same setup code path instead of duplicating Daemon/Notifier/TmuxClient
   construction. The callback controls inter-tick delay and can signal
   stop, which enables keypress handling during the sleep window.
   
   Add log view toggle to watch mode: press 'l' to see timestamped daemon
   activity (nudges fired, PR check results, errors), 't' to switch back
   to the table, 'q' to quit cleanly. Uses crossterm raw mode with 100ms
   poll intervals for responsive input.
   
   Also allows spawned workers to transition to stalled (previously
   Spawned status was excluded from silence detection).
 - <csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/> surface PR health in ps --watch display
   Add a HEALTH column to the watch table showing per-worker PR check
   results (ci, conflicts, reviews, commits) so problems are visible at a
   glance without needing RUST_LOG=debug. Upgrade silent debug-level PR
   errors to info-level logging.
 - <csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/> add --base flag to spawn and create for custom branch base
   Allow overriding the default base branch (from jig.toml) per-command
   with --base/-b. Includes shell completions for branch names across
   bash, zsh, and fish. Also fixes spawn status message to show the
   actual base branch used instead of the current branch.
 - <csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/> wire issues into spawn pipeline with --issue flag
   Add `jig spawn --issue <id>` to resolve file-based issues and use their
   body as Claude context. Thread issue_ref through the full pipeline:
   spawn CLI → register() → Spawn event → WorkerState reducer → daemon
   workers.json → ps watch table.
   
   Also adds:
   - `jig issues` CLI command with --ids flag for scripting
   - IssuesConfig in jig.toml for configurable issues directory
   - ISSUE column in ps --watch table (shortened last path segment)
   - Shell completions for --issue in bash, zsh, and fish
   - issue_ref tests in reducer and daemon roundtrip
 - <csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/> add watch mode to ps command for live dashboard
   `jig ps --watch` clears and refreshes the worker table every 2s.
   Shows enriched state from event logs alongside tmux status:
   - TMUX column (●/○/✗) for session liveness
   - STATE column from event-derived WorkerStatus
   - NUDGES count and PR number from event log
   - Configurable interval: `jig ps -w 5` for 5s refresh
 - <csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/> add daemon loop to orchestrate event-driven pipeline
   The missing conductor: `jig daemon` runs a periodic loop that:
   - Discovers workers by scanning event log directories
   - Replays events to derive current WorkerState per worker
   - Compares old vs new state to dispatch actions
   - Executes nudges via tmux and notifications via hooks
   - Persists state to workers.json between ticks
   
   Supports --once for single-pass mode and --interval for tuning.
 - <csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/> add git hook management (install, uninstall, handlers)
   Implements the git-hooks epic (tickets 0-4):
   - Hook wrapper templates that chain jig logic with user hooks
   - Registry tracking installed hooks at jig-hooks.json
   - Idempotent init with backup/restore of existing hooks
   - Post-commit/merge handlers that emit events to worker logs
   - Uninstall with rollback to original user hooks
 - <csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/> expand WorkerStatus with event-driven states
   Add Idle, WaitingInput, Stalled variants. Make all variants unit types
   (remove associated data from WaitingReview/Failed). Add needs_attention(),
   is_active(), is_terminal(), from_legacy() methods. Snake_case serialization.
 - <csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/> add event log format and Claude Code hooks
   Implement event-system tickets 1 and 2:
   - Event schema with typed EventType enum and flat JSONL serialization
   - EventLog append-only reader/writer with per-worker JSONL files
   - Claude Code hook templates (PostToolUse, Notification, Stop)
   - `jig hooks install-claude` CLI command to install hooks to ~/.claude/hooks/
 - <csr-id-82c654ab1137ec963121638f6741617c59ee0c04/> add global state infrastructure for cross-repo aggregation
   Introduces ~/.config/jig/ directory structure with structured TOML config,
   aggregated JSON worker state, and event log directories for the event-driven
   pipeline. Ensures global dirs are created at CLI startup.
 - <csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/> introduce RepoContext and thread repo state through all operations
   Derive repo_root, worktrees_dir, git_common_dir, base_branch, and
   session_name once at startup via RepoContext::from_cwd(), eliminating
   redundant git subprocess calls (e.g. spawn called get_base_repo() 8x).
   OpContext now holds Option<RepoContext>, and all jig-core functions
   accept &RepoContext instead of re-deriving from cwd. Also adds repo
   registry for global mode auto-registration, removes dead spawn::kill(),
   and updates docs/patterns/issue status.
 - <csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/> implement smart jig update command
   Rewrite update command to:
   - Detect installation method (script, cargo, source, unknown)
   - Check latest version from GitHub releases API
   - Auto-update for script installations (~/.local/bin)
   - Prompt dev builds to install release binaries
   - Offer cleanup of old cargo bin after source build updates
   - Add --force flag to skip version check
 - <csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/> prettify jig ps with Op pattern and comfy-table
   Introduce the Op trait to separate command logic from presentation.
   Rewrite `jig ps` as the first adopter: ops return typed data, Display
   impls own all formatting via comfy-table with terminal-width-aware
   column layout and color-coded status indicators.
   
   - Add Op trait in crates/jig-cli/src/op.rs
   - Rewrite ps command with PsOutput, PsError, and Op impl
   - Add comfy-table dependency for dynamic table rendering
   - Update main.rs dispatch to use Op::execute()
   - Add docs/ui/STDOUT-FORMATTING.md documenting the pattern
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
 - <csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/> restructure issue tracking with categories and templates
   - Add directory-based issue organization (epics/, features/, bugs/, chores/)
   - Add issue templates (_templates/): standalone.md, epic-index.md, ticket.md
   - Create plan-and-execute epic for orchestration vision
   - Update issues/README.md with comprehensive documentation
   - Update /issues skill for new directory structure
   - Remove old flat issue files and _template.md
   - Add .backup/ to .gitignore
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
 - <csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/> improve backup, audit prompt, and review skill
   - Backup now copies files to .backup/ directory preserving path structure
   - Audit prompt is detailed and opinionated about what to fill in each doc
   - Review skill now checks for documentation and skills updates
 - <csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/> upgrade jig init scaffolding to language-agnostic skeletons
   - Move issue-tracking.md to issues/README.md, fix "wt" → "jig"
   - Rename skills/jig → skills/spawn for consistency
   - Remove name: field from skill frontmatter
   - Add skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.md
   - Expand docs/index.md as documentation hub
   - Make CLAUDE.md template a skeleton with guidance comments
   - Upgrade settings.json: add $schema, ask tier for destructive ops, better secret patterns
   - Add issues/_template.md ticket template
 - <csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/> add Claude Code skills and simplify permissions
   - Add skills for check, draft, issues, review, and spawn commands
   - Simplify .claude/settings.json using wildcard permissions
   - Add jig.toml with spawn auto-configuration
   - Fix formatting in init.rs
 - <csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/> use actual templates for jig init instead of bare-bones placeholders
   - Embed templates from templates/ directory using include_str!
   - Add all 5 skills: check, draft, issues, review, spawn
   - Expand permissions to cover tools used by skills
   - Set spawn.auto = true by default
   - Use exec() on Unix for --audit flag (full terminal control)
   
   The init command now creates a complete scaffolding that matches
   the documentation, instead of empty placeholder comments.
 - <csr-id-3a78670c102178f25db9dc4020b534370fc36f84/> add --audit flag to init command that launches Claude interactively
   Uses exec() on Unix to replace the current process with Claude Code,
   giving it full terminal control for interactive documentation audit.
 - <csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/> add shell-setup command and fix shell completions
   - Add `jig shell-setup` command to automatically configure shell integration
   - Detects user's shell from $SHELL
   - Finds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)
   - Adds eval line with markers for easy identification
   - Places integration after PATH setup when possible
   - Supports --dry-run flag to preview changes
   
   - Rewrite shell completions with dynamic worktree completion
   - `jig open/attach/review/merge/kill/status <TAB>` shows actual worktrees
   - Context-aware completions for all subcommands
   - Simplified zsh completion using _arguments -C
   
   - Update docs/usage/shell-integration.md
   - Add quick setup section for shell-setup command
   - Add troubleshooting section for common issues
   - Remove stale `sc` alias references (legacy from "scribe" name)
 - <csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/> rewrite health check to validate repo setup and agent scaffolding
   Replace terminal-detection-focused health check with structured validation
   of system deps (git, tmux, claude), repository config (jig.toml, base
   branch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).
   Remove unused jq/gh dependency checks and dead required field. Exit
   non-zero when checks fail.
 - <csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/> add shell completions for bash, zsh, and fish
   Shell completions are now emitted alongside the shell wrapper function
   in `jig shell-init`. Completions cover all subcommands, aliases,
   per-command flags, nested config subcommands, and dynamic worktree
   name completion via `command jig list`.

### Bug Fixes

 - <csr-id-47d683b540b09d52770902df1a3d47e501372ba9/> consolidate spawn codepaths to ensure update_status in watch mode
   The watch-mode daemon (spawn_actor::spawn_single) was missing the
   update_status(InProgress) call after spawning, causing issues to be
   re-spawned on every tick. This was because three near-duplicate spawn
   codepaths existed (CLI, daemon blocking, daemon watch) and only two
   had the status update.
   
   Extract spawn_worker_for_issue() and update_issue_status() into the
   shared spawn module so all three codepaths use a single authoritative
   implementation. This eliminates ~120 lines of duplication and makes it
   impossible to miss a step in any codepath.
 - <csr-id-c5b52f59055c0f95498ef657685a18405bf6b515/> consolidate spawn codepaths to ensure update_status in watch mode
   The watch-mode daemon (spawn_actor::spawn_single) was missing the
   update_status(InProgress) call after spawning, causing issues to be
   re-spawned on every tick. This was because three near-duplicate spawn
   codepaths existed (CLI, daemon blocking, daemon watch) and only two
   had the status update.
   
   Extract spawn_worker_for_issue() and update_issue_status() into the
   shared spawn module so all three codepaths use a single authoritative
   implementation. This eliminates ~120 lines of duplication and makes it
   impossible to miss a step in any codepath.
 - <csr-id-0e71bb2f733c8d97cffe7dc2c1826c3aa61da2a9/> support -g/--global flag and show provider-specific details
   `jig config` and `jig config -g` now work outside a git repo by showing
   global config. Issues section displays Linear profile details (profile,
   team, projects, assignee, labels) when provider is "linear", and directory
   when provider is "file".
 - <csr-id-8015b3097d63862324524ac9e88e2bd411982839/> add .jig/ to .gitignore during init
   Previously .jig/ was only added to .git/info/exclude which is
   local to each clone. This meant hooks and state could leak into
   commits in fresh clones or CI. Now `jig init` adds .jig/ alongside
   jig.local.toml in .gitignore so the ignore is shared across clones.
 - <csr-id-847c9ba0e63c6aeb5a46fead6529eca9b8976465/> use `jig -g init` instead of broken `--global` flag on init
   The `--global` flag conflicts with the top-level `-g/--global` CLI flag.
   Implement `run_global()` on Init so `jig -g init` scaffolds the global
   config correctly.
 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter
   - Relation mapping was inverted: "blocks" → "is_blocked_by" so
   dependency checking now correctly identifies blocked issues
   - Remove unused SearchData type
   
   Improve issues command UX:
   - Hide completed issues by default (use --all to include them)
   - Interactive mode (-i) uses alternate screen buffer like less/git diff
   - Add ui::with_alternate_screen() reusable helper
   - Interactive mode: scrolling, auto indicator, title truncation, G/g nav
 - <csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/> accept trailing args in git hook subcommands
   Git passes arguments to hooks (e.g. post-merge receives a squash flag
   "0" or "1"), which the hook wrapper forwards via "$@". The CLI
   subcommands rejected these unexpected args. Add trailing_var_arg to
   PostCommit, PostMerge, and PreCommit to accept and ignore them.
 - <csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/> output cd command from jig home instead of bare path
   Matches the pattern used by `jig open` and `jig exit` — outputs
   `cd '/path'` to stdout for shell eval, not just the path.
 - <csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/> recover from stale git worktree registrations on spawn and prune
   When a worktree directory is removed but git still tracks the entry,
   `git worktree add` fails with "missing but already registered". Now
   create_worktree runs `git worktree prune` first, and prune_actor
   handles the missing-directory case instead of skipping cleanup.
   
   Also extracts prune_actor into its own module and adds urgent issue
   to replace git CLI shelling with git2.
 - <csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/> add issues command to shell completions
   The `issues` command was missing from all three shells' command lists
   and had no flag/argument completions. Adds command entry, issue ID
   positional completions, and flag completions (status, priority,
   category, interactive, ids) for bash, zsh, and fish.
 - <csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/> use if-let instead of unwrap to satisfy clippy
 - <csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/> daemon PR discovery, tmux targeting, and nudge delivery
   - Add proactive PR discovery: daemon queries GitHub for open PRs on
   worker branches when pr_url is unknown, emits PrOpened events to
   make state durable across restarts
   - Create per-repo GitHub clients via registry path lookup instead of
   ambient remote detection (fixes multi-repo daemon)
   - Extract real branch name from spawn events for tmux window lookup
   (spawn creates windows with slashes, e.g. feature/foo, not dashes)
   - Run all four PR checks (CI, conflicts, reviews, commits) on open PRs
   - Nudge on every tick, not just state transitions, so polling daemon
   retries delivery until max_nudges
   - Collapse multiline nudge templates to single line before tmux send
   to prevent premature submission in TUIs
   - Fix tracing init: RUST_LOG now properly overrides default warn level
   - Add stderr tick summary in continuous daemon mode for visibility
   without RUST_LOG
   - Add debug logging for tmux window misses and notification pipeline
 - <csr-id-378031a0afe019f57edc9bae469bf8168e05de29/> register Claude hooks in settings.json, add kill --all and nuke
   Claude Code hooks were installed as scripts but never registered in
   ~/.claude/settings.json, so they never fired. Now jig init registers
   them properly. Also fixes: hook templates read JSON from stdin (not
   env vars), spawned workers no longer nudged as stalled, event logs
   reset on respawn, row ordering stabilized in ps --watch, kill/unregister
   cleans up event logs, and nuke command added for full repo cleanup.
 - <csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/> address review findings and wire up event pipeline end-to-end
   Fix 6 issues from code review: UTF-8 safe truncate, stable status
   serialization via as_str/from_legacy, stuck nudge sends message after
   auto-approve, notification errors logged, branch names URL-encoded,
   tmux commands check exit status.
   
   Wire up missing pipeline links: jig spawn emits Spawn event, jig init
   auto-installs git+Claude hooks (idempotent on re-run), ps --watch runs
   daemon tick on each refresh for integrated orchestration.
   
   Add docs/daemon.md with background service setup for launchd, systemd,
   OpenRC, and generic nohup.
 - <csr-id-a41b92cb77141469539658c133da79f79f714452/> remove unnecessary return statement
 - <csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/> make --audit print command instead of trying to launch claude
   Spawning claude programmatically was causing terminal issues and hangs.
   Now --audit just prints the command for the user to run manually.
 - <csr-id-196774225c8eba52fdb9382f98418ecf82c48567/> prevent shell-setup from corrupting shell config files
   The previous byte-slicing approach in find_path_line_end() calculated
   offsets incorrectly because lines() strips newlines but the code assumed
   +1 byte per line. This could corrupt or truncate config files.

### Other

 - <csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/> fmt
 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/> consolidate auto-spawn into single `auto_spawn_labels` field
   Replace three redundant knobs (`spawn.auto`, `spawn.auto_spawn`,
   `issues.spawn_labels`) with a single `issues.auto_spawn_labels` field.
   Semantics: None = disabled, Some([]) = all issues, Some(["x"]) = filtered.
 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/
   - State file moved to .jig/.state/state.json
   - Auto-migration from .worktrees/ layout on first load
   - jig kill/unregister now removes workers from state entirely
   (instead of archiving them)
   - jig ps auto-cleans stale workers whose tmux windows are gone
   - Hidden directories (.state) are skipped when listing worktrees
   - .jig/.state/ added to .gitignore, .jig/ added to git exclude

## v1.10.0 (2026-03-20)

<csr-id-29da5be67a92fbb7e2cbd7674cb824ca17afb8d5/>
<csr-id-a28704f7e124ad3c35734469ff285d265fc7ccc9/>
<csr-id-e63634542688c53115dac2f70254224545dcb4c8/>
<csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/>
<csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/>
<csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/>
<csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/>
<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-29da5be67a92fbb7e2cbd7674cb824ca17afb8d5/> bump version to 1.10.0
 - <csr-id-a28704f7e124ad3c35734469ff285d265fc7ccc9/> bump version to 1.9.0
 - <csr-id-e63634542688c53115dac2f70254224545dcb4c8/> bump version to 1.8.0
 - <csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/> bump version to 1.7.1
 - <csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/> bump version to 1.7.0
 - <csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/> bump version to 1.6.0
 - <csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/> bump version to 1.5.0
 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

 - <csr-id-f313d4c1e1dba5e00cfcc1127eda9adb0f12d599/> move issues to InProgress on spawn to prevent duplicates
   Add `update_status` to the `IssueProvider` trait and call it in both
   spawn code paths (daemon auto-spawn and CLI `jig spawn`) to transition
   issues to InProgress immediately after a worker is launched. This
   prevents `list_spawnable()` from re-picking the same issue across tick
   cycles, since it filters for Planned status only.
 - <csr-id-b9bca75b2c6e59bcbd84c173a3e92b7717362baf/> improve create UX and add docs/tests
   - Make --category optional (defaults to "features" for file provider,
   omitted for Linear to avoid passing a meaningless project name)
   - Document `jig issues create` in Linear integration docs and issues skill
   - Add integration tests for default category, stdin body, and labels
 - <csr-id-e4445e544c26244b44b9051d72d5f34cd0c87da3/> support Linear provider for `jig issues create`
   Wire up `jig issues create` to dispatch through LinearProvider when
   provider = "linear" is configured. Adds a createIssue GraphQL mutation
   to LinearClient with helper queries to resolve team, label, and project
   IDs. The CLI gains a --body flag (inline text or "-" for stdin) that
   works with both file and Linear providers.
 - <csr-id-99f0311db0fb1fcac156aa2f03e8b04bfdd75199/> add jig.local.toml overlay and fix stderr color detection
   Add deep-merge support for jig.local.toml (gitignored) on top of
   jig.toml, allowing machine-specific overrides without touching the
   committed config. Tables merge recursively; scalars and arrays from the
   local file win. `jig init` auto-adds jig.local.toml to .gitignore.
   
   Revamp `jig config` display with source attribution showing where each
   setting originates (jig.toml, jig.local.toml, global config, or default).
   
   Fix colored crate TTY detection: override based on stderr since all jig
   output goes to stderr, preventing silent color suppression when stdout
   is piped.
 - <csr-id-0f9a72d8d53c686ff305a11c961c37083f26a47d/> add Create event and Created status for bare worktrees
   Distinguishes `jig create` worktrees from `jig spawn` workers via a
   positive Create event rather than relying on empty event logs. The daemon
   discovers Created workers (they appear in `jig list`) but takes no
   actions on them — no auto-resume, no nudges, no recovery.
 - <csr-id-205483042ca48167984c4076b0d01bbd81a00f2f/> derive spawn worktree name from issue, use repo base branch in nudges
   - Make `name` optional in `jig spawn` when `--issue` is provided; derive
   worktree name from the issue's branch_name or ID
   - Extract Linear identifiers from branch-format strings (e.g.
   `feature/aut-5044-refactor-foo` → `AUT-5044`)
   - Move derive_worker_name/sanitize_worker_name to shared issues::naming module
   - Pass repo's actual base branch to conflict and bad-commits nudge templates
   instead of hardcoding origin/main
   - Simplify resume to reuse launch(), remove build_resume_command
   - Skip recovery for Initializing workers (still running on-create hooks)
 - <csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/> add commit-msg hook for conventional commit validation
   Adds a commit-msg git hook that validates commit messages against the
   conventional commits spec using the existing parser and jig.toml config.
   Closes the conventional-commits-validation issue.
 - <csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/> add conventional commit validation and examples
   Add parser, configurable validator, and CLI commands:
   - `jig commit validate` validates HEAD, specific revs, stdin, or files
   - `jig commit examples` shows conventional commit reference
   - Configurable via `[commits]` section in jig.toml
   - 16 unit tests + 12 integration tests
 - <csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/> add issue lifecycle commands (create, status, complete, stats)
   Add CLI subcommands to `jig issues` for managing issue state:
   - `jig issues create` creates issues from templates with title, priority, category, labels
   - `jig issues status <id> --status <new>` updates frontmatter status in file issues
   - `jig issues complete <id>` marks issues as complete, with optional --delete
   - `jig issues stats` shows breakdown by status and priority
   
   For Linear issues, status changes go through the API (not file editing).
   Backwards-compatible: `jig issues` with no subcommand still lists/browses.
   Also adds "in-progress" (hyphenated) to loose status parsing.
 - <csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/> add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limiting
   Add three display features to make ps -w a proper dashboard:
   - Nudge cooldown countdown in NUDGE column (e.g. "2/3 (3m12s)")
   - Nudge messages rendered below worker table when delivered
   - Global sync/poll timer footer alongside keybinding hints
   
   Fix two bugs discovered during implementation:
   - Preserve draft PR status on GitHub API errors so nudges aren't
   silently suppressed for known-draft PRs
   - Throttle GitHub API requests to once per 60s per worker, aligned
   with gh --cache 60s TTL, reducing API pressure ~30x
 - <csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/> add daemon crash recovery and worker resume
   - Install SIGTERM/SIGINT handler for graceful daemon shutdown
   - Record Started/Stopped lifecycle events in daemon.jsonl
   - Detect unclean shutdown on next startup (missing Stopped event)
   - Auto-recover orphaned workers on daemon startup via Worktree::resume()
   - Add `jig resume <name>` CLI command for manual worker recovery
   - Detect dead tmux windows during steady-state ticks and auto-resume
   - Add `auto_recover` global config option (default: true) for opt-out
   - Wire Action::Restart to use recovery::try_resume_worker()
   - Add integration tests for jig resume command
 - <csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/> show nudge state in jig ps output
   Add NUDGE column between STATE and COMMITS in the ps table.
   Displays nudge count as count/max (e.g. 2/3), with color coding:
   grey dash for zero, yellow for in-progress, red for exhausted.
   
   Adds max_nudges field to WorkerDisplayInfo so the UI can render
   the denominator from the resolved health config.
 - <csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/> add per-repo nudge configuration in jig.toml
   Add [health] section to jig.toml supporting per-repo overrides of
   silence_threshold_seconds and max_nudges, plus per-nudge-type
   [health.nudge.<type>] sections with independent max and
   cooldown_seconds settings.
   
   Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]
   > global config > defaults. When cooldown_seconds is not set, falls
   back to silence_threshold_seconds.
   
   - Add RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structs
   - Add ResolvedNudgeConfig with resolver for per-type config
   - Thread resolved config through nudge classify, dispatch, and execute
   - Apply per-type cooldown to both idle/stalled nudges and PR nudges
   - Display effective nudge config in `jig config show`
   - Fixes PR nudge burst bug by enforcing per-type cooldowns
 - <csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/> add jig home command to print base repo root
   Adds `jig home` (alias `jig h`) that prints the base repository root
   path, enabling `cd $(jig home)` navigation from worktrees.
 - <csr-id-df0a3be811b27f8afce047bd088cad410d09e081/> communicate worker initialization state and on-create failures
   Add Initializing event type and worker status to make the worker
   lifecycle visible during setup. When the daemon auto-spawns a worker,
   it now registers the worker as Initializing before running the
   on-create hook, then transitions to Spawned on success or Failed on
   hook failure.
 - <csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/> show auto-spawn config in `jig config` and add `jig issues --auto`
   Display auto-spawn settings (enabled, auto-start, max workers, poll
   interval, spawn labels) in `jig config show` with source attribution.
   Add `--auto` flag to `jig issues` to filter to only daemon-eligible
   auto-spawn candidates using the existing `list_spawnable` method.
 - <csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/> use checkmark instead of asterisk for AUTO column
   Change the AUTO column indicator in `jig issues` from `*` to `✓`
   for better readability.
 - <csr-id-bd1a1faeca5a7634224bef836154791819b4903b/> use checkmark instead of asterisk for AUTO column
   Change the AUTO column indicator in `jig issues` from `*` to `✓`
   for better readability.
 - <csr-id-462f05eaf29929899631125c733738cd8f93e558/> move auto-spawn to background thread to keep ps -w responsive
   The on-create hook (e.g. pnpm install) was running synchronously on
   the tick thread, freezing the ps --watch UI for the entire duration.
   Introduces a spawn_actor following the same pattern as prune_actor,
   issue_actor, etc. The tick now sends spawnable issues to the background
   thread and drains results on the next tick.
   
   Also adds:
   - Spawning worker names shown below the ps table during setup
   - WorkerStatus::Initializing variant for future use
   - spawn_labels config in jig.toml
   - Three new issues (config-show-auto-spawn, worker-initializing-state,
   auto-column-checkmark)
 - <csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/> add labels field for issue tagging and filtering
   Add `labels: Vec<String>` to Issue and IssueFilter types. Linear
   provider now passes all label names through from GraphQL (auto field
   derivation unchanged). File provider parses `**Labels:**` comma-separated
   frontmatter. CLI gains `--label/-l` flag for filtering (all must match).
   Shell completions updated for bash, zsh, and fish.
 - <csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/> add shared UI module with consistent formatting and --plain flag
   Expand ui.rs into a centralized formatting module with:
   - Status symbol constants (✓, →, ✗, !)
   - Formatted output helpers (success, progress, failure, warning, detail, header)
   - Color helpers (highlight, bold, dim) that respect plain mode
   - Table builder helper (new_table) for consistent table creation
   - Global --plain flag for scriptable output (no colors, no decorations)
   - Error display with cause chain formatting
   
   Migrate all 20 command files from inline colored::Colorize calls to
   shared ui:: helpers. Add --plain support to list, repos, and issues
   commands with tab-separated output for piping.
 - <csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/> add AUTO column to `jig issues` table output
   Show a green dot indicator for issues tagged for auto-spawn, making it
   visible at a glance whether file-provider Auto flag or Linear jig-auto
   label is set.
 - <csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/> support -g/--global flag for attaching from anywhere
   Add run_global implementation to the Attach command so users can attach
   to a worktree from outside the owning repo using `jig attach <name> -g`.
   Resolves the owning repo via GlobalCtx::repo_for_worktree.
 - <csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/> jig init --audit launches agent in tmux to populate docs
   --audit now spawns the configured agent in a jig-init:<repo> tmux
   session with the audit prompt instead of just printing instructions.
   --backup enhances the prompt to reference .backup/ files. --audit
   accepts an optional string for extra instructions.
 - <csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/> block auto-spawn on unresolved dependencies
   Add `is_spawnable_with_deps()` to IssueProvider trait that checks all
   depends_on entries resolve to Complete before allowing spawn. Applied in
   both FileProvider and LinearProvider's list_spawnable(). Also adds
   --blocked/--unblocked flags to `jig issues` CLI for filtering.
 - <csr-id-057e8dc3675610e75e826910d051774f32f63cee/> group workers by repo in `jig ps -g` output
   Add `repo` field to `WorkerDisplayInfo` and render grouped tables with
   bold repo headers when running in global mode (`jig ps -g` / `jig ps -gw`).
   Local `jig ps` output is unchanged.
 - <csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/> daemon periodically prunes stale worktrees
   Workers in terminal state (merged/archived/failed) with dead tmux
   sessions now get their git worktrees, event logs, and global state
   entries cleaned up automatically. Prune runs every 120s during watch
   mode. Pruned workers are reported in the tick status and log view.
   
   Also includes snake_case fixes for auto-spawn-filtering ticket.
 - <csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/> default table view for `jig ls` and pretty grouped `jig ls -g`
   - `jig ls` now shows a table with name, branch, and commits ahead
   - `jig ls -g` shows tables grouped by repo with bold headers
   - Add `--plain/-p` flag for bare name output (old behavior)
   - Shell completions use `--plain` and fall back to `-gp` outside a repo
   - Branch column only shown when it differs from worktree name
 - <csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/> show draft vs review state, document PR nudge behavior
   Workers with draft PRs now show "draft" (blue) in the STATE column
   instead of "review" (cyan). This makes it visually clear which workers
   will receive PR nudges (draft) vs which are in human review (non-draft).
   
   Add PR Nudges section to daemon docs explaining the draft/non-draft
   nudge policy and what each health check means.
 - <csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/> unify daemon/ps tick loops and add log toggle to watch mode
   Extract run_with() callback API from daemon so ps --watch shares the
   same setup code path instead of duplicating Daemon/Notifier/TmuxClient
   construction. The callback controls inter-tick delay and can signal
   stop, which enables keypress handling during the sleep window.
   
   Add log view toggle to watch mode: press 'l' to see timestamped daemon
   activity (nudges fired, PR check results, errors), 't' to switch back
   to the table, 'q' to quit cleanly. Uses crossterm raw mode with 100ms
   poll intervals for responsive input.
   
   Also allows spawned workers to transition to stalled (previously
   Spawned status was excluded from silence detection).
 - <csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/> surface PR health in ps --watch display
   Add a HEALTH column to the watch table showing per-worker PR check
   results (ci, conflicts, reviews, commits) so problems are visible at a
   glance without needing RUST_LOG=debug. Upgrade silent debug-level PR
   errors to info-level logging.
 - <csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/> add --base flag to spawn and create for custom branch base
   Allow overriding the default base branch (from jig.toml) per-command
   with --base/-b. Includes shell completions for branch names across
   bash, zsh, and fish. Also fixes spawn status message to show the
   actual base branch used instead of the current branch.
 - <csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/> wire issues into spawn pipeline with --issue flag
   Add `jig spawn --issue <id>` to resolve file-based issues and use their
   body as Claude context. Thread issue_ref through the full pipeline:
   spawn CLI → register() → Spawn event → WorkerState reducer → daemon
   workers.json → ps watch table.
   
   Also adds:
   - `jig issues` CLI command with --ids flag for scripting
   - IssuesConfig in jig.toml for configurable issues directory
   - ISSUE column in ps --watch table (shortened last path segment)
   - Shell completions for --issue in bash, zsh, and fish
   - issue_ref tests in reducer and daemon roundtrip
 - <csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/> add watch mode to ps command for live dashboard
   `jig ps --watch` clears and refreshes the worker table every 2s.
   Shows enriched state from event logs alongside tmux status:
   - TMUX column (●/○/✗) for session liveness
   - STATE column from event-derived WorkerStatus
   - NUDGES count and PR number from event log
   - Configurable interval: `jig ps -w 5` for 5s refresh
 - <csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/> add daemon loop to orchestrate event-driven pipeline
   The missing conductor: `jig daemon` runs a periodic loop that:
   - Discovers workers by scanning event log directories
   - Replays events to derive current WorkerState per worker
   - Compares old vs new state to dispatch actions
   - Executes nudges via tmux and notifications via hooks
   - Persists state to workers.json between ticks
   
   Supports --once for single-pass mode and --interval for tuning.
 - <csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/> add git hook management (install, uninstall, handlers)
   Implements the git-hooks epic (tickets 0-4):
   - Hook wrapper templates that chain jig logic with user hooks
   - Registry tracking installed hooks at jig-hooks.json
   - Idempotent init with backup/restore of existing hooks
   - Post-commit/merge handlers that emit events to worker logs
   - Uninstall with rollback to original user hooks
 - <csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/> expand WorkerStatus with event-driven states
   Add Idle, WaitingInput, Stalled variants. Make all variants unit types
   (remove associated data from WaitingReview/Failed). Add needs_attention(),
   is_active(), is_terminal(), from_legacy() methods. Snake_case serialization.
 - <csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/> add event log format and Claude Code hooks
   Implement event-system tickets 1 and 2:
   - Event schema with typed EventType enum and flat JSONL serialization
   - EventLog append-only reader/writer with per-worker JSONL files
   - Claude Code hook templates (PostToolUse, Notification, Stop)
   - `jig hooks install-claude` CLI command to install hooks to ~/.claude/hooks/
 - <csr-id-82c654ab1137ec963121638f6741617c59ee0c04/> add global state infrastructure for cross-repo aggregation
   Introduces ~/.config/jig/ directory structure with structured TOML config,
   aggregated JSON worker state, and event log directories for the event-driven
   pipeline. Ensures global dirs are created at CLI startup.
 - <csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/> introduce RepoContext and thread repo state through all operations
   Derive repo_root, worktrees_dir, git_common_dir, base_branch, and
   session_name once at startup via RepoContext::from_cwd(), eliminating
   redundant git subprocess calls (e.g. spawn called get_base_repo() 8x).
   OpContext now holds Option<RepoContext>, and all jig-core functions
   accept &RepoContext instead of re-deriving from cwd. Also adds repo
   registry for global mode auto-registration, removes dead spawn::kill(),
   and updates docs/patterns/issue status.
 - <csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/> implement smart jig update command
   Rewrite update command to:
   - Detect installation method (script, cargo, source, unknown)
   - Check latest version from GitHub releases API
   - Auto-update for script installations (~/.local/bin)
   - Prompt dev builds to install release binaries
   - Offer cleanup of old cargo bin after source build updates
   - Add --force flag to skip version check
 - <csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/> prettify jig ps with Op pattern and comfy-table
   Introduce the Op trait to separate command logic from presentation.
   Rewrite `jig ps` as the first adopter: ops return typed data, Display
   impls own all formatting via comfy-table with terminal-width-aware
   column layout and color-coded status indicators.
   
   - Add Op trait in crates/jig-cli/src/op.rs
   - Rewrite ps command with PsOutput, PsError, and Op impl
   - Add comfy-table dependency for dynamic table rendering
   - Update main.rs dispatch to use Op::execute()
   - Add docs/ui/STDOUT-FORMATTING.md documenting the pattern
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
 - <csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/> restructure issue tracking with categories and templates
   - Add directory-based issue organization (epics/, features/, bugs/, chores/)
   - Add issue templates (_templates/): standalone.md, epic-index.md, ticket.md
   - Create plan-and-execute epic for orchestration vision
   - Update issues/README.md with comprehensive documentation
   - Update /issues skill for new directory structure
   - Remove old flat issue files and _template.md
   - Add .backup/ to .gitignore
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
 - <csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/> improve backup, audit prompt, and review skill
   - Backup now copies files to .backup/ directory preserving path structure
   - Audit prompt is detailed and opinionated about what to fill in each doc
   - Review skill now checks for documentation and skills updates
 - <csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/> upgrade jig init scaffolding to language-agnostic skeletons
   - Move issue-tracking.md to issues/README.md, fix "wt" → "jig"
   - Rename skills/jig → skills/spawn for consistency
   - Remove name: field from skill frontmatter
   - Add skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.md
   - Expand docs/index.md as documentation hub
   - Make CLAUDE.md template a skeleton with guidance comments
   - Upgrade settings.json: add $schema, ask tier for destructive ops, better secret patterns
   - Add issues/_template.md ticket template
 - <csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/> add Claude Code skills and simplify permissions
   - Add skills for check, draft, issues, review, and spawn commands
   - Simplify .claude/settings.json using wildcard permissions
   - Add jig.toml with spawn auto-configuration
   - Fix formatting in init.rs
 - <csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/> use actual templates for jig init instead of bare-bones placeholders
   - Embed templates from templates/ directory using include_str!
   - Add all 5 skills: check, draft, issues, review, spawn
   - Expand permissions to cover tools used by skills
   - Set spawn.auto = true by default
   - Use exec() on Unix for --audit flag (full terminal control)
   
   The init command now creates a complete scaffolding that matches
   the documentation, instead of empty placeholder comments.
 - <csr-id-3a78670c102178f25db9dc4020b534370fc36f84/> add --audit flag to init command that launches Claude interactively
   Uses exec() on Unix to replace the current process with Claude Code,
   giving it full terminal control for interactive documentation audit.
 - <csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/> add shell-setup command and fix shell completions
   - Add `jig shell-setup` command to automatically configure shell integration
   - Detects user's shell from $SHELL
   - Finds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)
   - Adds eval line with markers for easy identification
   - Places integration after PATH setup when possible
   - Supports --dry-run flag to preview changes
   
   - Rewrite shell completions with dynamic worktree completion
   - `jig open/attach/review/merge/kill/status <TAB>` shows actual worktrees
   - Context-aware completions for all subcommands
   - Simplified zsh completion using _arguments -C
   
   - Update docs/usage/shell-integration.md
   - Add quick setup section for shell-setup command
   - Add troubleshooting section for common issues
   - Remove stale `sc` alias references (legacy from "scribe" name)
 - <csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/> rewrite health check to validate repo setup and agent scaffolding
   Replace terminal-detection-focused health check with structured validation
   of system deps (git, tmux, claude), repository config (jig.toml, base
   branch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).
   Remove unused jq/gh dependency checks and dead required field. Exit
   non-zero when checks fail.
 - <csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/> add shell completions for bash, zsh, and fish
   Shell completions are now emitted alongside the shell wrapper function
   in `jig shell-init`. Completions cover all subcommands, aliases,
   per-command flags, nested config subcommands, and dynamic worktree
   name completion via `command jig list`.

### Bug Fixes

 - <csr-id-c5b52f59055c0f95498ef657685a18405bf6b515/> consolidate spawn codepaths to ensure update_status in watch mode
   The watch-mode daemon (spawn_actor::spawn_single) was missing the
   update_status(InProgress) call after spawning, causing issues to be
   re-spawned on every tick. This was because three near-duplicate spawn
   codepaths existed (CLI, daemon blocking, daemon watch) and only two
   had the status update.
   
   Extract spawn_worker_for_issue() and update_issue_status() into the
   shared spawn module so all three codepaths use a single authoritative
   implementation. This eliminates ~120 lines of duplication and makes it
   impossible to miss a step in any codepath.
 - <csr-id-0e71bb2f733c8d97cffe7dc2c1826c3aa61da2a9/> support -g/--global flag and show provider-specific details
   `jig config` and `jig config -g` now work outside a git repo by showing
   global config. Issues section displays Linear profile details (profile,
   team, projects, assignee, labels) when provider is "linear", and directory
   when provider is "file".
 - <csr-id-8015b3097d63862324524ac9e88e2bd411982839/> add .jig/ to .gitignore during init
   Previously .jig/ was only added to .git/info/exclude which is
   local to each clone. This meant hooks and state could leak into
   commits in fresh clones or CI. Now `jig init` adds .jig/ alongside
   jig.local.toml in .gitignore so the ignore is shared across clones.
 - <csr-id-847c9ba0e63c6aeb5a46fead6529eca9b8976465/> use `jig -g init` instead of broken `--global` flag on init
   The `--global` flag conflicts with the top-level `-g/--global` CLI flag.
   Implement `run_global()` on Init so `jig -g init` scaffolds the global
   config correctly.
 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter
   - Relation mapping was inverted: "blocks" → "is_blocked_by" so
   dependency checking now correctly identifies blocked issues
   - Remove unused SearchData type
   
   Improve issues command UX:
   - Hide completed issues by default (use --all to include them)
   - Interactive mode (-i) uses alternate screen buffer like less/git diff
   - Add ui::with_alternate_screen() reusable helper
   - Interactive mode: scrolling, auto indicator, title truncation, G/g nav
 - <csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/> accept trailing args in git hook subcommands
   Git passes arguments to hooks (e.g. post-merge receives a squash flag
   "0" or "1"), which the hook wrapper forwards via "$@". The CLI
   subcommands rejected these unexpected args. Add trailing_var_arg to
   PostCommit, PostMerge, and PreCommit to accept and ignore them.
 - <csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/> output cd command from jig home instead of bare path
   Matches the pattern used by `jig open` and `jig exit` — outputs
   `cd '/path'` to stdout for shell eval, not just the path.
 - <csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/> recover from stale git worktree registrations on spawn and prune
   When a worktree directory is removed but git still tracks the entry,
   `git worktree add` fails with "missing but already registered". Now
   create_worktree runs `git worktree prune` first, and prune_actor
   handles the missing-directory case instead of skipping cleanup.
   
   Also extracts prune_actor into its own module and adds urgent issue
   to replace git CLI shelling with git2.
 - <csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/> add issues command to shell completions
   The `issues` command was missing from all three shells' command lists
   and had no flag/argument completions. Adds command entry, issue ID
   positional completions, and flag completions (status, priority,
   category, interactive, ids) for bash, zsh, and fish.
 - <csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/> use if-let instead of unwrap to satisfy clippy
 - <csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/> daemon PR discovery, tmux targeting, and nudge delivery
   - Add proactive PR discovery: daemon queries GitHub for open PRs on
   worker branches when pr_url is unknown, emits PrOpened events to
   make state durable across restarts
   - Create per-repo GitHub clients via registry path lookup instead of
   ambient remote detection (fixes multi-repo daemon)
   - Extract real branch name from spawn events for tmux window lookup
   (spawn creates windows with slashes, e.g. feature/foo, not dashes)
   - Run all four PR checks (CI, conflicts, reviews, commits) on open PRs
   - Nudge on every tick, not just state transitions, so polling daemon
   retries delivery until max_nudges
   - Collapse multiline nudge templates to single line before tmux send
   to prevent premature submission in TUIs
   - Fix tracing init: RUST_LOG now properly overrides default warn level
   - Add stderr tick summary in continuous daemon mode for visibility
   without RUST_LOG
   - Add debug logging for tmux window misses and notification pipeline
 - <csr-id-378031a0afe019f57edc9bae469bf8168e05de29/> register Claude hooks in settings.json, add kill --all and nuke
   Claude Code hooks were installed as scripts but never registered in
   ~/.claude/settings.json, so they never fired. Now jig init registers
   them properly. Also fixes: hook templates read JSON from stdin (not
   env vars), spawned workers no longer nudged as stalled, event logs
   reset on respawn, row ordering stabilized in ps --watch, kill/unregister
   cleans up event logs, and nuke command added for full repo cleanup.
 - <csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/> address review findings and wire up event pipeline end-to-end
   Fix 6 issues from code review: UTF-8 safe truncate, stable status
   serialization via as_str/from_legacy, stuck nudge sends message after
   auto-approve, notification errors logged, branch names URL-encoded,
   tmux commands check exit status.
   
   Wire up missing pipeline links: jig spawn emits Spawn event, jig init
   auto-installs git+Claude hooks (idempotent on re-run), ps --watch runs
   daemon tick on each refresh for integrated orchestration.
   
   Add docs/daemon.md with background service setup for launchd, systemd,
   OpenRC, and generic nohup.
 - <csr-id-a41b92cb77141469539658c133da79f79f714452/> remove unnecessary return statement
 - <csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/> make --audit print command instead of trying to launch claude
   Spawning claude programmatically was causing terminal issues and hangs.
   Now --audit just prints the command for the user to run manually.
 - <csr-id-196774225c8eba52fdb9382f98418ecf82c48567/> prevent shell-setup from corrupting shell config files
   The previous byte-slicing approach in find_path_line_end() calculated
   offsets incorrectly because lines() strips newlines but the code assumed
   +1 byte per line. This could corrupt or truncate config files.

### Other

 - <csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/> fmt
 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/> consolidate auto-spawn into single `auto_spawn_labels` field
   Replace three redundant knobs (`spawn.auto`, `spawn.auto_spawn`,
   `issues.spawn_labels`) with a single `issues.auto_spawn_labels` field.
   Semantics: None = disabled, Some([]) = all issues, Some(["x"]) = filtered.
 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/
   - State file moved to .jig/.state/state.json
   - Auto-migration from .worktrees/ layout on first load
   - jig kill/unregister now removes workers from state entirely
   (instead of archiving them)
   - jig ps auto-cleans stale workers whose tmux windows are gone
   - Hidden directories (.state) are skipped when listing worktrees
   - .jig/.state/ added to .gitignore, .jig/ added to git exclude

## v1.9.0 (2026-03-19)

<csr-id-a28704f7e124ad3c35734469ff285d265fc7ccc9/>
<csr-id-e63634542688c53115dac2f70254224545dcb4c8/>
<csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/>
<csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/>
<csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/>
<csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/>
<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-a28704f7e124ad3c35734469ff285d265fc7ccc9/> bump version to 1.9.0
 - <csr-id-e63634542688c53115dac2f70254224545dcb4c8/> bump version to 1.8.0
 - <csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/> bump version to 1.7.1
 - <csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/> bump version to 1.7.0
 - <csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/> bump version to 1.6.0
 - <csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/> bump version to 1.5.0
 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

<csr-id-e4445e544c26244b44b9051d72d5f34cd0c87da3/>
<csr-id-99f0311db0fb1fcac156aa2f03e8b04bfdd75199/>
<csr-id-0f9a72d8d53c686ff305a11c961c37083f26a47d/>
<csr-id-205483042ca48167984c4076b0d01bbd81a00f2f/>
<csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/>
<csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/>
<csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/>
<csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/>
<csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/>
<csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/>
<csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/>
<csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/>
<csr-id-df0a3be811b27f8afce047bd088cad410d09e081/>
<csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/>
<csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/>
<csr-id-bd1a1faeca5a7634224bef836154791819b4903b/>
<csr-id-462f05eaf29929899631125c733738cd8f93e558/>
<csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/>
<csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/>
<csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/>
<csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/>
<csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/>
<csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/>
<csr-id-057e8dc3675610e75e826910d051774f32f63cee/>
<csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/>
<csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/>
<csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/>
<csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/>
<csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/>
<csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/>
<csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/>
<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-f313d4c1e1dba5e00cfcc1127eda9adb0f12d599/> move issues to InProgress on spawn to prevent duplicates
   Add `update_status` to the `IssueProvider` trait and call it in both
   spawn code paths (daemon auto-spawn and CLI `jig spawn`) to transition
   issues to InProgress immediately after a worker is launched. This
   prevents `list_spawnable()` from re-picking the same issue across tick
   cycles, since it filters for Planned status only.
 - <csr-id-b9bca75b2c6e59bcbd84c173a3e92b7717362baf/> improve create UX and add docs/tests
   - Make --category optional (defaults to "features" for file provider,
   omitted for Linear to avoid passing a meaningless project name)
- Document `jig issues create` in Linear integration docs and issues skill
- Add integration tests for default category, stdin body, and labels
- Make `name` optional in `jig spawn` when `--issue` is provided; derive
     worktree name from the issue's branch_name or ID
- Extract Linear identifiers from branch-format strings (e.g.
     `feature/aut-5044-refactor-foo` → `AUT-5044`)
- Move derive_worker_name/sanitize_worker_name to shared issues::naming module
- Pass repo's actual base branch to conflict and bad-commits nudge templates
     instead of hardcoding origin/main
- Simplify resume to reuse launch(), remove build_resume_command
- Skip recovery for Initializing workers (still running on-create hooks)
- `jig commit validate` validates HEAD, specific revs, stdin, or files
- `jig commit examples` shows conventional commit reference
- Configurable via `[commits]` section in jig.toml
- 16 unit tests + 12 integration tests
- `jig issues create` creates issues from templates with title, priority, category, labels
- `jig issues status <id> --status <new>` updates frontmatter status in file issues
- `jig issues complete <id>` marks issues as complete, with optional --delete
- `jig issues stats` shows breakdown by status and priority
- Nudge cooldown countdown in NUDGE column (e.g. "2/3 (3m12s)")
- Nudge messages rendered below worker table when delivered
- Global sync/poll timer footer alongside keybinding hints
- Preserve draft PR status on GitHub API errors so nudges aren't
     silently suppressed for known-draft PRs
- Throttle GitHub API requests to once per 60s per worker, aligned
     with gh --cache 60s TTL, reducing API pressure ~30x
- Install SIGTERM/SIGINT handler for graceful daemon shutdown
- Record Started/Stopped lifecycle events in daemon.jsonl
- Detect unclean shutdown on next startup (missing Stopped event)
- Auto-recover orphaned workers on daemon startup via Worktree::resume()
- Add `jig resume <name>` CLI command for manual worker recovery
- Detect dead tmux windows during steady-state ticks and auto-resume
- Add `auto_recover` global config option (default: true) for opt-out
- Wire Action::Restart to use recovery::try_resume_worker()
- Add integration tests for jig resume command
- Add RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structs
- Add ResolvedNudgeConfig with resolver for per-type config
- Thread resolved config through nudge classify, dispatch, and execute
- Apply per-type cooldown to both idle/stalled nudges and PR nudges
- Display effective nudge config in `jig config show`
- Fixes PR nudge burst bug by enforcing per-type cooldowns
- Spawning worker names shown below the ps table during setup
- WorkerStatus::Initializing variant for future use
- spawn_labels config in jig.toml
- Three new issues (config-show-auto-spawn, worker-initializing-state,
     auto-column-checkmark)
- Status symbol constants (✓, →, ✗, !)
- Formatted output helpers (success, progress, failure, warning, detail, header)
- Color helpers (highlight, bold, dim) that respect plain mode
- Table builder helper (new_table) for consistent table creation
- Global --plain flag for scriptable output (no colors, no decorations)
- Error display with cause chain formatting
- `jig ls` now shows a table with name, branch, and commits ahead
- `jig ls -g` shows tables grouped by repo with bold headers
- Add `--plain/-p` flag for bare name output (old behavior)
- Shell completions use `--plain` and fall back to `-gp` outside a repo
- Branch column only shown when it differs from worktree name
- `jig issues` CLI command with --ids flag for scripting
- IssuesConfig in jig.toml for configurable issues directory
- ISSUE column in ps --watch table (shortened last path segment)
- Shell completions for --issue in bash, zsh, and fish
- issue_ref tests in reducer and daemon roundtrip
- TMUX column (●/○/✗) for session liveness
- STATE column from event-derived WorkerStatus
- NUDGES count and PR number from event log
- Configurable interval: `jig ps -w 5` for 5s refresh
- Discovers workers by scanning event log directories
- Replays events to derive current WorkerState per worker
- Compares old vs new state to dispatch actions
- Executes nudges via tmux and notifications via hooks
- Persists state to workers.json between ticks
- Hook wrapper templates that chain jig logic with user hooks
- Registry tracking installed hooks at jig-hooks.json
- Idempotent init with backup/restore of existing hooks
- Post-commit/merge handlers that emit events to worker logs
- Uninstall with rollback to original user hooks
- Event schema with typed EventType enum and flat JSONL serialization
- EventLog append-only reader/writer with per-worker JSONL files
- Claude Code hook templates (PostToolUse, Notification, Stop)
- `jig hooks install-claude` CLI command to install hooks to ~/.claude/hooks/
- Detect installation method (script, cargo, source, unknown)
- Check latest version from GitHub releases API
- Auto-update for script installations (~/.local/bin)
- Prompt dev builds to install release binaries
- Offer cleanup of old cargo bin after source build updates
- Add --force flag to skip version check
- Add Op trait in crates/jig-cli/src/op.rs
- Rewrite ps command with PsOutput, PsError, and Op impl
- Add comfy-table dependency for dynamic table rendering
- Update main.rs dispatch to use Op::execute()
- Add docs/ui/STDOUT-FORMATTING.md documenting the pattern
- `worktree.base` — base branch for new worktrees (overrides global)
- `worktree.on_create` — command to run after worktree creation
- Add directory-based issue organization (epics/, features/, bugs/, chores/)
- Add issue templates (_templates/): standalone.md, epic-index.md, ticket.md
- Create plan-and-execute epic for orchestration vision
- Update issues/README.md with comprehensive documentation
- Update /issues skill for new directory structure
- Remove old flat issue files and _template.md
- Add .backup/ to .gitignore
- Add AgentType enum for compile-time safe matching
- Rename template to PROJECT.md (agent-agnostic name)
- Dynamic audit prompt uses adapter.project_file and adapter.skills_dir
- Validate agent is installed before init (warns if not in PATH)
- Fix settings.json schema URL
- Fix settings.json to use correct schemastore.org URL
- Add WebFetch, WebSearch, mcp__*, jig:* to default permissions
- Update review skill to check jig-specific docs and skills
- Update issues skill to reference issues/README.md
- Add adapter module with AgentAdapter struct for pluggable agent support
- jig init now requires agent argument: `jig init claude`
- jig.toml stores agent type in [agent] section
- spawn command uses adapter to build agent-specific commands
- Move settings.json to templates/adapters/claude-code/
- Backup now copies files to .backup/ directory preserving path structure
- Audit prompt is detailed and opinionated about what to fill in each doc
- Review skill now checks for documentation and skills updates
- Move issue-tracking.md to issues/README.md, fix "wt" → "jig"
- Rename skills/jig → skills/spawn for consistency
- Remove name: field from skill frontmatter
- Add skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.md
- Expand docs/index.md as documentation hub
- Make CLAUDE.md template a skeleton with guidance comments
- Upgrade settings.json: add $schema, ask tier for destructive ops, better secret patterns
- Add issues/_template.md ticket template
- Add skills for check, draft, issues, review, and spawn commands
- Simplify .claude/settings.json using wildcard permissions
- Add jig.toml with spawn auto-configuration
- Fix formatting in init.rs
- Embed templates from templates/ directory using include_str!
- Add all 5 skills: check, draft, issues, review, spawn
- Expand permissions to cover tools used by skills
- Set spawn.auto = true by default
- Use exec() on Unix for --audit flag (full terminal control)
- Add `jig shell-setup` command to automatically configure shell integration
     - Detects user's shell from $SHELL
     - Finds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)
     - Adds eval line with markers for easy identification
     - Places integration after PATH setup when possible
     - Supports --dry-run flag to preview changes
- Finds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)
- Adds eval line with markers for easy identification
- Places integration after PATH setup when possible
- Supports --dry-run flag to preview changes
- `jig open/attach/review/merge/kill/status <TAB>` shows actual worktrees
- Context-aware completions for all subcommands
- Simplified zsh completion using _arguments -C
- Add quick setup section for shell-setup command
- Add troubleshooting section for common issues
- Remove stale `sc` alias references (legacy from "scribe" name)

### Bug Fixes

<csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/>
<csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/>
<csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/>
<csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/>
<csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/>
<csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/>
<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-0e71bb2f733c8d97cffe7dc2c1826c3aa61da2a9/> support -g/--global flag and show provider-specific details
   `jig config` and `jig config -g` now work outside a git repo by showing
   global config. Issues section displays Linear profile details (profile,
   team, projects, assignee, labels) when provider is "linear", and directory
   when provider is "file".
 - <csr-id-8015b3097d63862324524ac9e88e2bd411982839/> add .jig/ to .gitignore during init
   Previously .jig/ was only added to .git/info/exclude which is
   local to each clone. This meant hooks and state could leak into
   commits in fresh clones or CI. Now `jig init` adds .jig/ alongside
   jig.local.toml in .gitignore so the ignore is shared across clones.
 - <csr-id-847c9ba0e63c6aeb5a46fead6529eca9b8976465/> use `jig -g init` instead of broken `--global` flag on init
   The `--global` flag conflicts with the top-level `-g/--global` CLI flag.
   Implement `run_global()` on Init so `jig -g init` scaffolds the global
   config correctly.
 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter
- Relation mapping was inverted: "blocks" → "is_blocked_by" so
     dependency checking now correctly identifies blocked issues
- Remove unused SearchData type
- Hide completed issues by default (use --all to include them)
- Interactive mode (-i) uses alternate screen buffer like less/git diff
- Add ui::with_alternate_screen() reusable helper
- Interactive mode: scrolling, auto indicator, title truncation, G/g nav
- Add proactive PR discovery: daemon queries GitHub for open PRs on
     worker branches when pr_url is unknown, emits PrOpened events to
     make state durable across restarts
- Create per-repo GitHub clients via registry path lookup instead of
     ambient remote detection (fixes multi-repo daemon)
- Extract real branch name from spawn events for tmux window lookup
     (spawn creates windows with slashes, e.g. feature/foo, not dashes)
- Run all four PR checks (CI, conflicts, reviews, commits) on open PRs
- Nudge on every tick, not just state transitions, so polling daemon
     retries delivery until max_nudges
- Collapse multiline nudge templates to single line before tmux send
     to prevent premature submission in TUIs
- Fix tracing init: RUST_LOG now properly overrides default warn level
- Add stderr tick summary in continuous daemon mode for visibility
     without RUST_LOG
- Add debug logging for tmux window misses and notification pipeline

### Other

 - <csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/> fmt
 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/> consolidate auto-spawn into single `auto_spawn_labels` field
   Replace three redundant knobs (`spawn.auto`, `spawn.auto_spawn`,
   `issues.spawn_labels`) with a single `issues.auto_spawn_labels` field.
   Semantics: None = disabled, Some([]) = all issues, Some(["x"]) = filtered.
 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/
- State file moved to .jig/.state/state.json
- Auto-migration from .worktrees/ layout on first load
- jig kill/unregister now removes workers from state entirely
     (instead of archiving them)
- jig ps auto-cleans stale workers whose tmux windows are gone
- Hidden directories (.state) are skipped when listing worktrees
- .jig/.state/ added to .gitignore, .jig/ added to git exclude

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
 support Linear provider for jig issues createWire up jig issues create to dispatch through LinearProvider whenprovider = “linear” is configured. Adds a createIssue GraphQL mutationto LinearClient with helper queries to resolve team, label, and projectIDs. The CLI gains a –body flag (inline text or “-” for stdin) thatworks with both file and Linear providers. add jig.local.toml overlay and fix stderr color detectionAdd deep-merge support for jig.local.toml (gitignored) on top ofjig.toml, allowing machine-specific overrides without touching thecommitted config. Tables merge recursively; scalars and arrays from thelocal file win. jig init auto-adds jig.local.toml to .gitignore.Revamp jig config display with source attribution showing where eachsetting originates (jig.toml, jig.local.toml, global config, or default).Fix colored crate TTY detection: override based on stderr since all jigoutput goes to stderr, preventing silent color suppression when stdoutis piped. add Create event and Created status for bare worktreesDistinguishes jig create worktrees from jig spawn workers via apositive Create event rather than relying on empty event logs. The daemondiscovers Created workers (they appear in jig list) but takes noactions on them — no auto-resume, no nudges, no recovery. derive spawn worktree name from issue, use repo base branch in nudges add commit-msg hook for conventional commit validationAdds a commit-msg git hook that validates commit messages against theconventional commits spec using the existing parser and jig.toml config.Closes the conventional-commits-validation issue. add conventional commit validation and examplesAdd parser, configurable validator, and CLI commands: add issue lifecycle commands (create, status, complete, stats)Add CLI subcommands to jig issues for managing issue state:For Linear issues, status changes go through the API (not file editing).Backwards-compatible: jig issues with no subcommand still lists/browses.Also adds “in-progress” (hyphenated) to loose status parsing. add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limitingAdd three display features to make ps -w a proper dashboard:Fix two bugs discovered during implementation: add daemon crash recovery and worker resume show nudge state in jig ps outputAdd NUDGE column between STATE and COMMITS in the ps table.Displays nudge count as count/max (e.g. 2/3), with color coding:grey dash for zero, yellow for in-progress, red for exhausted.Adds max_nudges field to WorkerDisplayInfo so the UI can renderthe denominator from the resolved health config. add per-repo nudge configuration in jig.tomlAdd [health] section to jig.toml supporting per-repo overrides ofsilence_threshold_seconds and max_nudges, plus per-nudge-type[health.nudge.<type>] sections with independent max andcooldown_seconds settings.Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]global config > defaults. When cooldown_seconds is not set, fallsback to silence_threshold_seconds. add jig home command to print base repo rootAdds jig home (alias jig h) that prints the base repository rootpath, enabling cd $(jig home) navigation from worktrees. communicate worker initialization state and on-create failuresAdd Initializing event type and worker status to make the workerlifecycle visible during setup. When the daemon auto-spawns a worker,it now registers the worker as Initializing before running theon-create hook, then transitions to Spawned on success or Failed onhook failure. show auto-spawn config in jig config and add jig issues --autoDisplay auto-spawn settings (enabled, auto-start, max workers, pollinterval, spawn labels) in jig config show with source attribution.Add --auto flag to jig issues to filter to only daemon-eligibleauto-spawn candidates using the existing list_spawnable method. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. move auto-spawn to background thread to keep ps -w responsiveThe on-create hook (e.g. pnpm install) was running synchronously onthe tick thread, freezing the ps –watch UI for the entire duration.Introduces a spawn_actor following the same pattern as prune_actor,issue_actor, etc. The tick now sends spawnable issues to the backgroundthread and drains results on the next tick.Also adds: add labels field for issue tagging and filteringAdd labels: Vec<String> to Issue and IssueFilter types. Linearprovider now passes all label names through from GraphQL (auto fieldderivation unchanged). File provider parses **Labels:** comma-separatedfrontmatter. CLI gains --label/-l flag for filtering (all must match).Shell completions updated for bash, zsh, and fish. add shared UI module with consistent formatting and –plain flagExpand ui.rs into a centralized formatting module with:Migrate all 20 command files from inline colored::Colorize calls toshared ui:: helpers. Add –plain support to list, repos, and issuescommands with tab-separated output for piping. add AUTO column to jig issues table outputShow a green dot indicator for issues tagged for auto-spawn, making itvisible at a glance whether file-provider Auto flag or Linear jig-autolabel is set. support -g/–global flag for attaching from anywhereAdd run_global implementation to the Attach command so users can attachto a worktree from outside the owning repo using jig attach <name> -g.Resolves the owning repo via GlobalCtx::repo_for_worktree. jig init –audit launches agent in tmux to populate docs–audit now spawns the configured agent in a jig-init:<repo> tmuxsession with the audit prompt instead of just printing instructions.–backup enhances the prompt to reference .backup/ files. –auditaccepts an optional string for extra instructions. block auto-spawn on unresolved dependenciesAdd is_spawnable_with_deps() to IssueProvider trait that checks alldepends_on entries resolve to Complete before allowing spawn. Applied inboth FileProvider and LinearProvider’s list_spawnable(). Also adds–blocked/–unblocked flags to jig issues CLI for filtering. group workers by repo in jig ps -g outputAdd repo field to WorkerDisplayInfo and render grouped tables withbold repo headers when running in global mode (jig ps -g / jig ps -gw).Local jig ps output is unchanged. daemon periodically prunes stale worktreesWorkers in terminal state (merged/archived/failed) with dead tmuxsessions now get their git worktrees, event logs, and global stateentries cleaned up automatically. Prune runs every 120s during watchmode. Pruned workers are reported in the tick status and log view.Also includes snake_case fixes for auto-spawn-filtering ticket. default table view for jig ls and pretty grouped jig ls -g show draft vs review state, document PR nudge behaviorWorkers with draft PRs now show “draft” (blue) in the STATE columninstead of “review” (cyan). This makes it visually clear which workerswill receive PR nudges (draft) vs which are in human review (non-draft).Add PR Nudges section to daemon docs explaining the draft/non-draftnudge policy and what each health check means. unify daemon/ps tick loops and add log toggle to watch modeExtract run_with() callback API from daemon so ps –watch shares thesame setup code path instead of duplicating Daemon/Notifier/TmuxClientconstruction. The callback controls inter-tick delay and can signalstop, which enables keypress handling during the sleep window.Add log view toggle to watch mode: press ‘l’ to see timestamped daemonactivity (nudges fired, PR check results, errors), ‘t’ to switch backto the table, ‘q’ to quit cleanly. Uses crossterm raw mode with 100mspoll intervals for responsive input.Also allows spawned workers to transition to stalled (previouslySpawned status was excluded from silence detection). surface PR health in ps –watch displayAdd a HEALTH column to the watch table showing per-worker PR checkresults (ci, conflicts, reviews, commits) so problems are visible at aglance without needing RUST_LOG=debug. Upgrade silent debug-level PRerrors to info-level logging. add –base flag to spawn and create for custom branch baseAllow overriding the default base branch (from jig.toml) per-commandwith –base/-b. Includes shell completions for branch names acrossbash, zsh, and fish. Also fixes spawn status message to show theactual base branch used instead of the current branch. wire issues into spawn pipeline with –issue flagAdd jig spawn --issue <id> to resolve file-based issues and use theirbody as Claude context. Thread issue_ref through the full pipeline:spawn CLI → register() → Spawn event → WorkerState reducer → daemonworkers.json → ps watch table.Also adds: add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.Improve issues command UX: accept trailing args in git hook subcommandsGit passes arguments to hooks (e.g. post-merge receives a squash flag“0” or “1”), which the hook wrapper forwards via “$@”. The CLIsubcommands rejected these unexpected args. Add trailing_var_arg toPostCommit, PostMerge, and PreCommit to accept and ignore them. output cd command from jig home instead of bare pathMatches the pattern used by jig open and jig exit — outputscd '/path' to stdout for shell eval, not just the path. recover from stale git worktree registrations on spawn and pruneWhen a worktree directory is removed but git still tracks the entry,git worktree add fails with “missing but already registered”. Nowcreate_worktree runs git worktree prune first, and prune_actorhandles the missing-directory case instead of skipping cleanup.Also extracts prune_actor into its own module and adds urgent issueto replace git CLI shelling with git2. add issues command to shell completionsThe issues command was missing from all three shells’ command listsand had no flag/argument completions. Adds command entry, issue IDpositional completions, and flag completions (status, priority,category, interactive, ids) for bash, zsh, and fish. use if-let instead of unwrap to satisfy clippy daemon PR discovery, tmux targeting, and nudge delivery register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.8.0 (2026-03-19)

<csr-id-e63634542688c53115dac2f70254224545dcb4c8/>
<csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/>
<csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/>
<csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/>
<csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/>
<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-e63634542688c53115dac2f70254224545dcb4c8/> bump version to 1.8.0
 - <csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/> bump version to 1.7.1
 - <csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/> bump version to 1.7.0
 - <csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/> bump version to 1.6.0
 - <csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/> bump version to 1.5.0
 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

<csr-id-e4445e544c26244b44b9051d72d5f34cd0c87da3/>
<csr-id-99f0311db0fb1fcac156aa2f03e8b04bfdd75199/>
<csr-id-0f9a72d8d53c686ff305a11c961c37083f26a47d/>
<csr-id-205483042ca48167984c4076b0d01bbd81a00f2f/>
<csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/>
<csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/>
<csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/>
<csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/>
<csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/>
<csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/>
<csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/>
<csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/>
<csr-id-df0a3be811b27f8afce047bd088cad410d09e081/>
<csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/>
<csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/>
<csr-id-bd1a1faeca5a7634224bef836154791819b4903b/>
<csr-id-462f05eaf29929899631125c733738cd8f93e558/>
<csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/>
<csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/>
<csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/>
<csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/>
<csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/>
<csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/>
<csr-id-057e8dc3675610e75e826910d051774f32f63cee/>
<csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/>
<csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/>
<csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/>
<csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/>
<csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/>
<csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/>
<csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/>
<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-b9bca75b2c6e59bcbd84c173a3e92b7717362baf/> improve create UX and add docs/tests
   - Make --category optional (defaults to "features" for file provider,
   omitted for Linear to avoid passing a meaningless project name)
   - Detects user's shell from $SHELL
   - Finds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)
   - Adds eval line with markers for easy identification
   - Places integration after PATH setup when possible
   - Supports --dry-run flag to preview changes

### Bug Fixes

<csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/>
<csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/>
<csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/>
<csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/>
<csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/>
<csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/>
<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-0e71bb2f733c8d97cffe7dc2c1826c3aa61da2a9/> support -g/--global flag and show provider-specific details
   `jig config` and `jig config -g` now work outside a git repo by showing
   global config. Issues section displays Linear profile details (profile,
   team, projects, assignee, labels) when provider is "linear", and directory
   when provider is "file".
 - <csr-id-8015b3097d63862324524ac9e88e2bd411982839/> add .jig/ to .gitignore during init
   Previously .jig/ was only added to .git/info/exclude which is
   local to each clone. This meant hooks and state could leak into
   commits in fresh clones or CI. Now `jig init` adds .jig/ alongside
   jig.local.toml in .gitignore so the ignore is shared across clones.
 - <csr-id-847c9ba0e63c6aeb5a46fead6529eca9b8976465/> use `jig -g init` instead of broken `--global` flag on init
   The `--global` flag conflicts with the top-level `-g/--global` CLI flag.
   Implement `run_global()` on Init so `jig -g init` scaffolds the global
   config correctly.
 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter

### Other

 - <csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/> fmt
 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/> consolidate auto-spawn into single `auto_spawn_labels` field
   Replace three redundant knobs (`spawn.auto`, `spawn.auto_spawn`,
   `issues.spawn_labels`) with a single `issues.auto_spawn_labels` field.
   Semantics: None = disabled, Some([]) = all issues, Some(["x"]) = filtered.
 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Document jig issues create in Linear integration docs and issues skillAdd integration tests for default category, stdin body, and labelsMake name optional in jig spawn when --issue is provided; deriveworktree name from the issue’s branch_name or IDExtract Linear identifiers from branch-format strings (e.g.feature/aut-5044-refactor-foo → AUT-5044)Move derive_worker_name/sanitize_worker_name to shared issues::naming modulePass repo’s actual base branch to conflict and bad-commits nudge templatesinstead of hardcoding origin/mainSimplify resume to reuse launch(), remove build_resume_commandSkip recovery for Initializing workers (still running on-create hooks)jig commit validate validates HEAD, specific revs, stdin, or filesjig commit examples shows conventional commit referenceConfigurable via [commits] section in jig.toml16 unit tests + 12 integration testsjig issues create creates issues from templates with title, priority, category, labelsjig issues status <id> --status <new> updates frontmatter status in file issuesjig issues complete <id> marks issues as complete, with optional –deletejig issues stats shows breakdown by status and priorityNudge cooldown countdown in NUDGE column (e.g. “2/3 (3m12s)”)Nudge messages rendered below worker table when deliveredGlobal sync/poll timer footer alongside keybinding hintsPreserve draft PR status on GitHub API errors so nudges aren’tsilently suppressed for known-draft PRsThrottle GitHub API requests to once per 60s per worker, alignedwith gh –cache 60s TTL, reducing API pressure ~30xInstall SIGTERM/SIGINT handler for graceful daemon shutdownRecord Started/Stopped lifecycle events in daemon.jsonlDetect unclean shutdown on next startup (missing Stopped event)Auto-recover orphaned workers on daemon startup via Worktree::resume()Add jig resume <name> CLI command for manual worker recoveryDetect dead tmux windows during steady-state ticks and auto-resumeAdd auto_recover global config option (default: true) for opt-outWire Action::Restart to use recovery::try_resume_worker()Add integration tests for jig resume commandAdd RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structsAdd ResolvedNudgeConfig with resolver for per-type configThread resolved config through nudge classify, dispatch, and executeApply per-type cooldown to both idle/stalled nudges and PR nudgesDisplay effective nudge config in jig config showFixes PR nudge burst bug by enforcing per-type cooldownsSpawning worker names shown below the ps table during setupWorkerStatus::Initializing variant for future usespawn_labels config in jig.tomlThree new issues (config-show-auto-spawn, worker-initializing-state,auto-column-checkmark)Status symbol constants (✓, →, ✗, !)Formatted output helpers (success, progress, failure, warning, detail, header)Color helpers (highlight, bold, dim) that respect plain modeTable builder helper (new_table) for consistent table creationGlobal –plain flag for scriptable output (no colors, no decorations)Error display with cause chain formattingjig ls now shows a table with name, branch, and commits aheadjig ls -g shows tables grouped by repo with bold headersAdd --plain/-p flag for bare name output (old behavior)Shell completions use --plain and fall back to -gp outside a repoBranch column only shown when it differs from worktree namejig issues CLI command with –ids flag for scriptingIssuesConfig in jig.toml for configurable issues directoryISSUE column in ps –watch table (shortened last path segment)Shell completions for –issue in bash, zsh, and fishissue_ref tests in reducer and daemon roundtripTMUX column (●/○/✗) for session livenessSTATE column from event-derived WorkerStatusNUDGES count and PR number from event logConfigurable interval: jig ps -w 5 for 5s refreshDiscovers workers by scanning event log directoriesReplays events to derive current WorkerState per workerCompares old vs new state to dispatch actionsExecutes nudges via tmux and notifications via hooksPersists state to workers.json between ticksHook wrapper templates that chain jig logic with user hooksRegistry tracking installed hooks at jig-hooks.jsonIdempotent init with backup/restore of existing hooksPost-commit/merge handlers that emit events to worker logsUninstall with rollback to original user hooksEvent schema with typed EventType enum and flat JSONL serializationEventLog append-only reader/writer with per-worker JSONL filesClaude Code hook templates (PostToolUse, Notification, Stop)jig hooks install-claude CLI command to install hooks to ~/.claude/hooks/Detect installation method (script, cargo, source, unknown)Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)Relation mapping was inverted: “blocks” → “is_blocked_by” sodependency checking now correctly identifies blocked issuesRemove unused SearchData typeHide completed issues by default (use –all to include them)Interactive mode (-i) uses alternate screen buffer like less/git diffAdd ui::with_alternate_screen() reusable helperInteractive mode: scrolling, auto indicator, title truncation, G/g navAdd proactive PR discovery: daemon queries GitHub for open PRs onworker branches when pr_url is unknown, emits PrOpened events tomake state durable across restartsCreate per-repo GitHub clients via registry path lookup instead ofambient remote detection (fixes multi-repo daemon)Extract real branch name from spawn events for tmux window lookup(spawn creates windows with slashes, e.g. feature/foo, not dashes)Run all four PR checks (CI, conflicts, reviews, commits) on open PRsNudge on every tick, not just state transitions, so polling daemonretries delivery until max_nudgesCollapse multiline nudge templates to single line before tmux sendto prevent premature submission in TUIsFix tracing init: RUST_LOG now properly overrides default warn levelAdd stderr tick summary in continuous daemon mode for visibilitywithout RUST_LOGAdd debug logging for tmux window misses and notification pipelineState file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 support Linear provider for jig issues createWire up jig issues create to dispatch through LinearProvider whenprovider = “linear” is configured. Adds a createIssue GraphQL mutationto LinearClient with helper queries to resolve team, label, and projectIDs. The CLI gains a –body flag (inline text or “-” for stdin) thatworks with both file and Linear providers. add jig.local.toml overlay and fix stderr color detectionAdd deep-merge support for jig.local.toml (gitignored) on top ofjig.toml, allowing machine-specific overrides without touching thecommitted config. Tables merge recursively; scalars and arrays from thelocal file win. jig init auto-adds jig.local.toml to .gitignore.Revamp jig config display with source attribution showing where eachsetting originates (jig.toml, jig.local.toml, global config, or default).Fix colored crate TTY detection: override based on stderr since all jigoutput goes to stderr, preventing silent color suppression when stdoutis piped. add Create event and Created status for bare worktreesDistinguishes jig create worktrees from jig spawn workers via apositive Create event rather than relying on empty event logs. The daemondiscovers Created workers (they appear in jig list) but takes noactions on them — no auto-resume, no nudges, no recovery. derive spawn worktree name from issue, use repo base branch in nudges add commit-msg hook for conventional commit validationAdds a commit-msg git hook that validates commit messages against theconventional commits spec using the existing parser and jig.toml config.Closes the conventional-commits-validation issue. add conventional commit validation and examplesAdd parser, configurable validator, and CLI commands: add issue lifecycle commands (create, status, complete, stats)Add CLI subcommands to jig issues for managing issue state:For Linear issues, status changes go through the API (not file editing).Backwards-compatible: jig issues with no subcommand still lists/browses.Also adds “in-progress” (hyphenated) to loose status parsing. add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limitingAdd three display features to make ps -w a proper dashboard:Fix two bugs discovered during implementation: add daemon crash recovery and worker resume show nudge state in jig ps outputAdd NUDGE column between STATE and COMMITS in the ps table.Displays nudge count as count/max (e.g. 2/3), with color coding:grey dash for zero, yellow for in-progress, red for exhausted.Adds max_nudges field to WorkerDisplayInfo so the UI can renderthe denominator from the resolved health config. add per-repo nudge configuration in jig.tomlAdd [health] section to jig.toml supporting per-repo overrides ofsilence_threshold_seconds and max_nudges, plus per-nudge-type[health.nudge.<type>] sections with independent max andcooldown_seconds settings.Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]global config > defaults. When cooldown_seconds is not set, fallsback to silence_threshold_seconds. add jig home command to print base repo rootAdds jig home (alias jig h) that prints the base repository rootpath, enabling cd $(jig home) navigation from worktrees. communicate worker initialization state and on-create failuresAdd Initializing event type and worker status to make the workerlifecycle visible during setup. When the daemon auto-spawns a worker,it now registers the worker as Initializing before running theon-create hook, then transitions to Spawned on success or Failed onhook failure. show auto-spawn config in jig config and add jig issues --autoDisplay auto-spawn settings (enabled, auto-start, max workers, pollinterval, spawn labels) in jig config show with source attribution.Add --auto flag to jig issues to filter to only daemon-eligibleauto-spawn candidates using the existing list_spawnable method. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. move auto-spawn to background thread to keep ps -w responsiveThe on-create hook (e.g. pnpm install) was running synchronously onthe tick thread, freezing the ps –watch UI for the entire duration.Introduces a spawn_actor following the same pattern as prune_actor,issue_actor, etc. The tick now sends spawnable issues to the backgroundthread and drains results on the next tick.Also adds: add labels field for issue tagging and filteringAdd labels: Vec<String> to Issue and IssueFilter types. Linearprovider now passes all label names through from GraphQL (auto fieldderivation unchanged). File provider parses **Labels:** comma-separatedfrontmatter. CLI gains --label/-l flag for filtering (all must match).Shell completions updated for bash, zsh, and fish. add shared UI module with consistent formatting and –plain flagExpand ui.rs into a centralized formatting module with:Migrate all 20 command files from inline colored::Colorize calls toshared ui:: helpers. Add –plain support to list, repos, and issuescommands with tab-separated output for piping. add AUTO column to jig issues table outputShow a green dot indicator for issues tagged for auto-spawn, making itvisible at a glance whether file-provider Auto flag or Linear jig-autolabel is set. support -g/–global flag for attaching from anywhereAdd run_global implementation to the Attach command so users can attachto a worktree from outside the owning repo using jig attach <name> -g.Resolves the owning repo via GlobalCtx::repo_for_worktree. jig init –audit launches agent in tmux to populate docs–audit now spawns the configured agent in a jig-init:<repo> tmuxsession with the audit prompt instead of just printing instructions.–backup enhances the prompt to reference .backup/ files. –auditaccepts an optional string for extra instructions. block auto-spawn on unresolved dependenciesAdd is_spawnable_with_deps() to IssueProvider trait that checks alldepends_on entries resolve to Complete before allowing spawn. Applied inboth FileProvider and LinearProvider’s list_spawnable(). Also adds–blocked/–unblocked flags to jig issues CLI for filtering. group workers by repo in jig ps -g outputAdd repo field to WorkerDisplayInfo and render grouped tables withbold repo headers when running in global mode (jig ps -g / jig ps -gw).Local jig ps output is unchanged. daemon periodically prunes stale worktreesWorkers in terminal state (merged/archived/failed) with dead tmuxsessions now get their git worktrees, event logs, and global stateentries cleaned up automatically. Prune runs every 120s during watchmode. Pruned workers are reported in the tick status and log view.Also includes snake_case fixes for auto-spawn-filtering ticket. default table view for jig ls and pretty grouped jig ls -g show draft vs review state, document PR nudge behaviorWorkers with draft PRs now show “draft” (blue) in the STATE columninstead of “review” (cyan). This makes it visually clear which workerswill receive PR nudges (draft) vs which are in human review (non-draft).Add PR Nudges section to daemon docs explaining the draft/non-draftnudge policy and what each health check means. unify daemon/ps tick loops and add log toggle to watch modeExtract run_with() callback API from daemon so ps –watch shares thesame setup code path instead of duplicating Daemon/Notifier/TmuxClientconstruction. The callback controls inter-tick delay and can signalstop, which enables keypress handling during the sleep window.Add log view toggle to watch mode: press ‘l’ to see timestamped daemonactivity (nudges fired, PR check results, errors), ‘t’ to switch backto the table, ‘q’ to quit cleanly. Uses crossterm raw mode with 100mspoll intervals for responsive input.Also allows spawned workers to transition to stalled (previouslySpawned status was excluded from silence detection). surface PR health in ps –watch displayAdd a HEALTH column to the watch table showing per-worker PR checkresults (ci, conflicts, reviews, commits) so problems are visible at aglance without needing RUST_LOG=debug. Upgrade silent debug-level PRerrors to info-level logging. add –base flag to spawn and create for custom branch baseAllow overriding the default base branch (from jig.toml) per-commandwith –base/-b. Includes shell completions for branch names acrossbash, zsh, and fish. Also fixes spawn status message to show theactual base branch used instead of the current branch. wire issues into spawn pipeline with –issue flagAdd jig spawn --issue <id> to resolve file-based issues and use theirbody as Claude context. Thread issue_ref through the full pipeline:spawn CLI → register() → Spawn event → WorkerState reducer → daemonworkers.json → ps watch table.Also adds: add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.Improve issues command UX: accept trailing args in git hook subcommandsGit passes arguments to hooks (e.g. post-merge receives a squash flag“0” or “1”), which the hook wrapper forwards via “$@”. The CLIsubcommands rejected these unexpected args. Add trailing_var_arg toPostCommit, PostMerge, and PreCommit to accept and ignore them. output cd command from jig home instead of bare pathMatches the pattern used by jig open and jig exit — outputscd '/path' to stdout for shell eval, not just the path. recover from stale git worktree registrations on spawn and pruneWhen a worktree directory is removed but git still tracks the entry,git worktree add fails with “missing but already registered”. Nowcreate_worktree runs git worktree prune first, and prune_actorhandles the missing-directory case instead of skipping cleanup.Also extracts prune_actor into its own module and adds urgent issueto replace git CLI shelling with git2. add issues command to shell completionsThe issues command was missing from all three shells’ command listsand had no flag/argument completions. Adds command entry, issue IDpositional completions, and flag completions (status, priority,category, interactive, ids) for bash, zsh, and fish. use if-let instead of unwrap to satisfy clippy daemon PR discovery, tmux targeting, and nudge delivery register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.7.1 (2026-03-19)

<csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/>
<csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/>
<csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/>
<csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/>
<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-c07af99fea4549ff15d293e641e2a8f6e4504ceb/> bump version to 1.7.1
 - <csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/> bump version to 1.7.0
 - <csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/> bump version to 1.6.0
 - <csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/> bump version to 1.5.0
 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

<csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/>
<csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/>
<csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/>
<csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/>
<csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/>
<csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/>
<csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/>
<csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/>
<csr-id-df0a3be811b27f8afce047bd088cad410d09e081/>
<csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/>
<csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/>
<csr-id-bd1a1faeca5a7634224bef836154791819b4903b/>
<csr-id-462f05eaf29929899631125c733738cd8f93e558/>
<csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/>
<csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/>
<csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/>
<csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/>
<csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/>
<csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/>
<csr-id-057e8dc3675610e75e826910d051774f32f63cee/>
<csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/>
<csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/>
<csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/>
<csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/>
<csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/>
<csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/>
<csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/>
<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-99f0311db0fb1fcac156aa2f03e8b04bfdd75199/> add jig.local.toml overlay and fix stderr color detection
   Add deep-merge support for jig.local.toml (gitignored) on top of
   jig.toml, allowing machine-specific overrides without touching the
   committed config. Tables merge recursively; scalars and arrays from the
   local file win. `jig init` auto-adds jig.local.toml to .gitignore.
   
   Revamp `jig config` display with source attribution showing where each
   setting originates (jig.toml, jig.local.toml, global config, or default).
   
   Fix colored crate TTY detection: override based on stderr since all jig
   output goes to stderr, preventing silent color suppression when stdout
   is piped.
 - <csr-id-0f9a72d8d53c686ff305a11c961c37083f26a47d/> add Create event and Created status for bare worktrees
   Distinguishes `jig create` worktrees from `jig spawn` workers via a
   positive Create event rather than relying on empty event logs. The daemon
   discovers Created workers (they appear in `jig list`) but takes no
   actions on them — no auto-resume, no nudges, no recovery.
 - <csr-id-205483042ca48167984c4076b0d01bbd81a00f2f/> derive spawn worktree name from issue, use repo base branch in nudges
   - Make `name` optional in `jig spawn` when `--issue` is provided; derive
   worktree name from the issue's branch_name or ID
- Detects user's shell from $SHELL
- Finds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)
- Adds eval line with markers for easy identification
- Places integration after PATH setup when possible
- Supports --dry-run flag to preview changes

### Bug Fixes

<csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/>
<csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/>
<csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/>
<csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/>
<csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/>
<csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/>
<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-0e71bb2f733c8d97cffe7dc2c1826c3aa61da2a9/> support -g/--global flag and show provider-specific details
   `jig config` and `jig config -g` now work outside a git repo by showing
   global config. Issues section displays Linear profile details (profile,
   team, projects, assignee, labels) when provider is "linear", and directory
   when provider is "file".
 - <csr-id-8015b3097d63862324524ac9e88e2bd411982839/> add .jig/ to .gitignore during init
   Previously .jig/ was only added to .git/info/exclude which is
   local to each clone. This meant hooks and state could leak into
   commits in fresh clones or CI. Now `jig init` adds .jig/ alongside
   jig.local.toml in .gitignore so the ignore is shared across clones.
 - <csr-id-847c9ba0e63c6aeb5a46fead6529eca9b8976465/> use `jig -g init` instead of broken `--global` flag on init
   The `--global` flag conflicts with the top-level `-g/--global` CLI flag.
   Implement `run_global()` on Init so `jig -g init` scaffolds the global
   config correctly.
 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter

### Other

 - <csr-id-d32cb32b168798c7e3acd930f25f963e80a58ef0/> fmt
 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/> consolidate auto-spawn into single `auto_spawn_labels` field
   Replace three redundant knobs (`spawn.auto`, `spawn.auto_spawn`,
   `issues.spawn_labels`) with a single `issues.auto_spawn_labels` field.
   Semantics: None = disabled, Some([]) = all issues, Some(["x"]) = filtered.
 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Extract Linear identifiers from branch-format strings (e.g.feature/aut-5044-refactor-foo → AUT-5044)Move derive_worker_name/sanitize_worker_name to shared issues::naming modulePass repo’s actual base branch to conflict and bad-commits nudge templatesinstead of hardcoding origin/mainSimplify resume to reuse launch(), remove build_resume_commandSkip recovery for Initializing workers (still running on-create hooks)jig commit validate validates HEAD, specific revs, stdin, or filesjig commit examples shows conventional commit referenceConfigurable via [commits] section in jig.toml16 unit tests + 12 integration testsjig issues create creates issues from templates with title, priority, category, labelsjig issues status <id> --status <new> updates frontmatter status in file issuesjig issues complete <id> marks issues as complete, with optional –deletejig issues stats shows breakdown by status and priorityNudge cooldown countdown in NUDGE column (e.g. “2/3 (3m12s)”)Nudge messages rendered below worker table when deliveredGlobal sync/poll timer footer alongside keybinding hintsPreserve draft PR status on GitHub API errors so nudges aren’tsilently suppressed for known-draft PRsThrottle GitHub API requests to once per 60s per worker, alignedwith gh –cache 60s TTL, reducing API pressure ~30xInstall SIGTERM/SIGINT handler for graceful daemon shutdownRecord Started/Stopped lifecycle events in daemon.jsonlDetect unclean shutdown on next startup (missing Stopped event)Auto-recover orphaned workers on daemon startup via Worktree::resume()Add jig resume <name> CLI command for manual worker recoveryDetect dead tmux windows during steady-state ticks and auto-resumeAdd auto_recover global config option (default: true) for opt-outWire Action::Restart to use recovery::try_resume_worker()Add integration tests for jig resume commandAdd RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structsAdd ResolvedNudgeConfig with resolver for per-type configThread resolved config through nudge classify, dispatch, and executeApply per-type cooldown to both idle/stalled nudges and PR nudgesDisplay effective nudge config in jig config showFixes PR nudge burst bug by enforcing per-type cooldownsSpawning worker names shown below the ps table during setupWorkerStatus::Initializing variant for future usespawn_labels config in jig.tomlThree new issues (config-show-auto-spawn, worker-initializing-state,auto-column-checkmark)Status symbol constants (✓, →, ✗, !)Formatted output helpers (success, progress, failure, warning, detail, header)Color helpers (highlight, bold, dim) that respect plain modeTable builder helper (new_table) for consistent table creationGlobal –plain flag for scriptable output (no colors, no decorations)Error display with cause chain formattingjig ls now shows a table with name, branch, and commits aheadjig ls -g shows tables grouped by repo with bold headersAdd --plain/-p flag for bare name output (old behavior)Shell completions use --plain and fall back to -gp outside a repoBranch column only shown when it differs from worktree namejig issues CLI command with –ids flag for scriptingIssuesConfig in jig.toml for configurable issues directoryISSUE column in ps –watch table (shortened last path segment)Shell completions for –issue in bash, zsh, and fishissue_ref tests in reducer and daemon roundtripTMUX column (●/○/✗) for session livenessSTATE column from event-derived WorkerStatusNUDGES count and PR number from event logConfigurable interval: jig ps -w 5 for 5s refreshDiscovers workers by scanning event log directoriesReplays events to derive current WorkerState per workerCompares old vs new state to dispatch actionsExecutes nudges via tmux and notifications via hooksPersists state to workers.json between ticksHook wrapper templates that chain jig logic with user hooksRegistry tracking installed hooks at jig-hooks.jsonIdempotent init with backup/restore of existing hooksPost-commit/merge handlers that emit events to worker logsUninstall with rollback to original user hooksEvent schema with typed EventType enum and flat JSONL serializationEventLog append-only reader/writer with per-worker JSONL filesClaude Code hook templates (PostToolUse, Notification, Stop)jig hooks install-claude CLI command to install hooks to ~/.claude/hooks/Detect installation method (script, cargo, source, unknown)Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)Relation mapping was inverted: “blocks” → “is_blocked_by” sodependency checking now correctly identifies blocked issuesRemove unused SearchData typeHide completed issues by default (use –all to include them)Interactive mode (-i) uses alternate screen buffer like less/git diffAdd ui::with_alternate_screen() reusable helperInteractive mode: scrolling, auto indicator, title truncation, G/g navAdd proactive PR discovery: daemon queries GitHub for open PRs onworker branches when pr_url is unknown, emits PrOpened events tomake state durable across restartsCreate per-repo GitHub clients via registry path lookup instead ofambient remote detection (fixes multi-repo daemon)Extract real branch name from spawn events for tmux window lookup(spawn creates windows with slashes, e.g. feature/foo, not dashes)Run all four PR checks (CI, conflicts, reviews, commits) on open PRsNudge on every tick, not just state transitions, so polling daemonretries delivery until max_nudgesCollapse multiline nudge templates to single line before tmux sendto prevent premature submission in TUIsFix tracing init: RUST_LOG now properly overrides default warn levelAdd stderr tick summary in continuous daemon mode for visibilitywithout RUST_LOGAdd debug logging for tmux window misses and notification pipelineState file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 add commit-msg hook for conventional commit validationAdds a commit-msg git hook that validates commit messages against theconventional commits spec using the existing parser and jig.toml config.Closes the conventional-commits-validation issue. add conventional commit validation and examplesAdd parser, configurable validator, and CLI commands: add issue lifecycle commands (create, status, complete, stats)Add CLI subcommands to jig issues for managing issue state:For Linear issues, status changes go through the API (not file editing).Backwards-compatible: jig issues with no subcommand still lists/browses.Also adds “in-progress” (hyphenated) to loose status parsing. add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limitingAdd three display features to make ps -w a proper dashboard:Fix two bugs discovered during implementation: add daemon crash recovery and worker resume show nudge state in jig ps outputAdd NUDGE column between STATE and COMMITS in the ps table.Displays nudge count as count/max (e.g. 2/3), with color coding:grey dash for zero, yellow for in-progress, red for exhausted.Adds max_nudges field to WorkerDisplayInfo so the UI can renderthe denominator from the resolved health config. add per-repo nudge configuration in jig.tomlAdd [health] section to jig.toml supporting per-repo overrides ofsilence_threshold_seconds and max_nudges, plus per-nudge-type[health.nudge.<type>] sections with independent max andcooldown_seconds settings.Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]global config > defaults. When cooldown_seconds is not set, fallsback to silence_threshold_seconds. add jig home command to print base repo rootAdds jig home (alias jig h) that prints the base repository rootpath, enabling cd $(jig home) navigation from worktrees. communicate worker initialization state and on-create failuresAdd Initializing event type and worker status to make the workerlifecycle visible during setup. When the daemon auto-spawns a worker,it now registers the worker as Initializing before running theon-create hook, then transitions to Spawned on success or Failed onhook failure. show auto-spawn config in jig config and add jig issues --autoDisplay auto-spawn settings (enabled, auto-start, max workers, pollinterval, spawn labels) in jig config show with source attribution.Add --auto flag to jig issues to filter to only daemon-eligibleauto-spawn candidates using the existing list_spawnable method. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. move auto-spawn to background thread to keep ps -w responsiveThe on-create hook (e.g. pnpm install) was running synchronously onthe tick thread, freezing the ps –watch UI for the entire duration.Introduces a spawn_actor following the same pattern as prune_actor,issue_actor, etc. The tick now sends spawnable issues to the backgroundthread and drains results on the next tick.Also adds: add labels field for issue tagging and filteringAdd labels: Vec<String> to Issue and IssueFilter types. Linearprovider now passes all label names through from GraphQL (auto fieldderivation unchanged). File provider parses **Labels:** comma-separatedfrontmatter. CLI gains --label/-l flag for filtering (all must match).Shell completions updated for bash, zsh, and fish. add shared UI module with consistent formatting and –plain flagExpand ui.rs into a centralized formatting module with:Migrate all 20 command files from inline colored::Colorize calls toshared ui:: helpers. Add –plain support to list, repos, and issuescommands with tab-separated output for piping. add AUTO column to jig issues table outputShow a green dot indicator for issues tagged for auto-spawn, making itvisible at a glance whether file-provider Auto flag or Linear jig-autolabel is set. support -g/–global flag for attaching from anywhereAdd run_global implementation to the Attach command so users can attachto a worktree from outside the owning repo using jig attach <name> -g.Resolves the owning repo via GlobalCtx::repo_for_worktree. jig init –audit launches agent in tmux to populate docs–audit now spawns the configured agent in a jig-init:<repo> tmuxsession with the audit prompt instead of just printing instructions.–backup enhances the prompt to reference .backup/ files. –auditaccepts an optional string for extra instructions. block auto-spawn on unresolved dependenciesAdd is_spawnable_with_deps() to IssueProvider trait that checks alldepends_on entries resolve to Complete before allowing spawn. Applied inboth FileProvider and LinearProvider’s list_spawnable(). Also adds–blocked/–unblocked flags to jig issues CLI for filtering. group workers by repo in jig ps -g outputAdd repo field to WorkerDisplayInfo and render grouped tables withbold repo headers when running in global mode (jig ps -g / jig ps -gw).Local jig ps output is unchanged. daemon periodically prunes stale worktreesWorkers in terminal state (merged/archived/failed) with dead tmuxsessions now get their git worktrees, event logs, and global stateentries cleaned up automatically. Prune runs every 120s during watchmode. Pruned workers are reported in the tick status and log view.Also includes snake_case fixes for auto-spawn-filtering ticket. default table view for jig ls and pretty grouped jig ls -g show draft vs review state, document PR nudge behaviorWorkers with draft PRs now show “draft” (blue) in the STATE columninstead of “review” (cyan). This makes it visually clear which workerswill receive PR nudges (draft) vs which are in human review (non-draft).Add PR Nudges section to daemon docs explaining the draft/non-draftnudge policy and what each health check means. unify daemon/ps tick loops and add log toggle to watch modeExtract run_with() callback API from daemon so ps –watch shares thesame setup code path instead of duplicating Daemon/Notifier/TmuxClientconstruction. The callback controls inter-tick delay and can signalstop, which enables keypress handling during the sleep window.Add log view toggle to watch mode: press ‘l’ to see timestamped daemonactivity (nudges fired, PR check results, errors), ‘t’ to switch backto the table, ‘q’ to quit cleanly. Uses crossterm raw mode with 100mspoll intervals for responsive input.Also allows spawned workers to transition to stalled (previouslySpawned status was excluded from silence detection). surface PR health in ps –watch displayAdd a HEALTH column to the watch table showing per-worker PR checkresults (ci, conflicts, reviews, commits) so problems are visible at aglance without needing RUST_LOG=debug. Upgrade silent debug-level PRerrors to info-level logging. add –base flag to spawn and create for custom branch baseAllow overriding the default base branch (from jig.toml) per-commandwith –base/-b. Includes shell completions for branch names acrossbash, zsh, and fish. Also fixes spawn status message to show theactual base branch used instead of the current branch. wire issues into spawn pipeline with –issue flagAdd jig spawn --issue <id> to resolve file-based issues and use theirbody as Claude context. Thread issue_ref through the full pipeline:spawn CLI → register() → Spawn event → WorkerState reducer → daemonworkers.json → ps watch table.Also adds: add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.Improve issues command UX: accept trailing args in git hook subcommandsGit passes arguments to hooks (e.g. post-merge receives a squash flag“0” or “1”), which the hook wrapper forwards via “$@”. The CLIsubcommands rejected these unexpected args. Add trailing_var_arg toPostCommit, PostMerge, and PreCommit to accept and ignore them. output cd command from jig home instead of bare pathMatches the pattern used by jig open and jig exit — outputscd '/path' to stdout for shell eval, not just the path. recover from stale git worktree registrations on spawn and pruneWhen a worktree directory is removed but git still tracks the entry,git worktree add fails with “missing but already registered”. Nowcreate_worktree runs git worktree prune first, and prune_actorhandles the missing-directory case instead of skipping cleanup.Also extracts prune_actor into its own module and adds urgent issueto replace git CLI shelling with git2. add issues command to shell completionsThe issues command was missing from all three shells’ command listsand had no flag/argument completions. Adds command entry, issue IDpositional completions, and flag completions (status, priority,category, interactive, ids) for bash, zsh, and fish. use if-let instead of unwrap to satisfy clippy daemon PR discovery, tmux targeting, and nudge delivery register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.7.0 (2026-03-19)

<csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/>
<csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/>
<csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/>
<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-ae4ed1246e5824f85f8bc64e9806e138354375a6/> bump version to 1.7.0
 - <csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/> bump version to 1.6.0
 - <csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/> bump version to 1.5.0
 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

<csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/>
<csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/>
<csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/>
<csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/>
<csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/>
<csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/>
<csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/>
<csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/>
<csr-id-df0a3be811b27f8afce047bd088cad410d09e081/>
<csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/>
<csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/>
<csr-id-bd1a1faeca5a7634224bef836154791819b4903b/>
<csr-id-462f05eaf29929899631125c733738cd8f93e558/>
<csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/>
<csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/>
<csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/>
<csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/>
<csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/>
<csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/>
<csr-id-057e8dc3675610e75e826910d051774f32f63cee/>
<csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/>
<csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/>
<csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/>
<csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/>
<csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/>
<csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/>
<csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/>
<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-99f0311db0fb1fcac156aa2f03e8b04bfdd75199/> add jig.local.toml overlay and fix stderr color detection
   Add deep-merge support for jig.local.toml (gitignored) on top of
   jig.toml, allowing machine-specific overrides without touching the
   committed config. Tables merge recursively; scalars and arrays from the
   local file win. `jig init` auto-adds jig.local.toml to .gitignore.
   
   Revamp `jig config` display with source attribution showing where each
   setting originates (jig.toml, jig.local.toml, global config, or default).
   
   Fix colored crate TTY detection: override based on stderr since all jig
   output goes to stderr, preventing silent color suppression when stdout
   is piped.
 - <csr-id-0f9a72d8d53c686ff305a11c961c37083f26a47d/> add Create event and Created status for bare worktrees
   Distinguishes `jig create` worktrees from `jig spawn` workers via a
   positive Create event rather than relying on empty event logs. The daemon
   discovers Created workers (they appear in `jig list`) but takes no
   actions on them — no auto-resume, no nudges, no recovery.
 - <csr-id-205483042ca48167984c4076b0d01bbd81a00f2f/> derive spawn worktree name from issue, use repo base branch in nudges
   - Make `name` optional in `jig spawn` when `--issue` is provided; derive
   worktree name from the issue's branch_name or ID

### Bug Fixes

<csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/>
<csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/>
<csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/>
<csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/>
<csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/>
<csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/>
<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-8015b3097d63862324524ac9e88e2bd411982839/> add .jig/ to .gitignore during init
   Previously .jig/ was only added to .git/info/exclude which is
   local to each clone. This meant hooks and state could leak into
   commits in fresh clones or CI. Now `jig init` adds .jig/ alongside
   jig.local.toml in .gitignore so the ignore is shared across clones.
 - <csr-id-847c9ba0e63c6aeb5a46fead6529eca9b8976465/> use `jig -g init` instead of broken `--global` flag on init
   The `--global` flag conflicts with the top-level `-g/--global` CLI flag.
   Implement `run_global()` on Init so `jig -g init` scaffolds the global
   config correctly.
 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter

### Other

 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/> consolidate auto-spawn into single `auto_spawn_labels` field
   Replace three redundant knobs (`spawn.auto`, `spawn.auto_spawn`,
   `issues.spawn_labels`) with a single `issues.auto_spawn_labels` field.
   Semantics: None = disabled, Some([]) = all issues, Some(["x"]) = filtered.
 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELLFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changes<csr-unknown>
<csr-unknown>
Extract Linear identifiers from branch-format strings (e.g.feature/aut-5044-refactor-foo → AUT-5044)Move derive_worker_name/sanitize_worker_name to shared issues::naming modulePass repo’s actual base branch to conflict and bad-commits nudge templatesinstead of hardcoding origin/mainSimplify resume to reuse launch(), remove build_resume_commandSkip recovery for Initializing workers (still running on-create hooks)jig commit validate validates HEAD, specific revs, stdin, or filesjig commit examples shows conventional commit referenceConfigurable via [commits] section in jig.toml16 unit tests + 12 integration testsjig issues create creates issues from templates with title, priority, category, labelsjig issues status <id> --status <new> updates frontmatter status in file issuesjig issues complete <id> marks issues as complete, with optional –deletejig issues stats shows breakdown by status and priorityNudge cooldown countdown in NUDGE column (e.g. “2/3 (3m12s)”)Nudge messages rendered below worker table when deliveredGlobal sync/poll timer footer alongside keybinding hintsPreserve draft PR status on GitHub API errors so nudges aren’tsilently suppressed for known-draft PRsThrottle GitHub API requests to once per 60s per worker, alignedwith gh –cache 60s TTL, reducing API pressure ~30xInstall SIGTERM/SIGINT handler for graceful daemon shutdownRecord Started/Stopped lifecycle events in daemon.jsonlDetect unclean shutdown on next startup (missing Stopped event)Auto-recover orphaned workers on daemon startup via Worktree::resume()Add jig resume <name> CLI command for manual worker recoveryDetect dead tmux windows during steady-state ticks and auto-resumeAdd auto_recover global config option (default: true) for opt-outWire Action::Restart to use recovery::try_resume_worker()Add integration tests for jig resume commandAdd RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structsAdd ResolvedNudgeConfig with resolver for per-type configThread resolved config through nudge classify, dispatch, and executeApply per-type cooldown to both idle/stalled nudges and PR nudgesDisplay effective nudge config in jig config showFixes PR nudge burst bug by enforcing per-type cooldownsSpawning worker names shown below the ps table during setupWorkerStatus::Initializing variant for future usespawn_labels config in jig.tomlThree new issues (config-show-auto-spawn, worker-initializing-state,auto-column-checkmark)Status symbol constants (✓, →, ✗, !)Formatted output helpers (success, progress, failure, warning, detail, header)Color helpers (highlight, bold, dim) that respect plain modeTable builder helper (new_table) for consistent table creationGlobal –plain flag for scriptable output (no colors, no decorations)Error display with cause chain formattingjig ls now shows a table with name, branch, and commits aheadjig ls -g shows tables grouped by repo with bold headersAdd --plain/-p flag for bare name output (old behavior)Shell completions use --plain and fall back to -gp outside a repoBranch column only shown when it differs from worktree namejig issues CLI command with –ids flag for scriptingIssuesConfig in jig.toml for configurable issues directoryISSUE column in ps –watch table (shortened last path segment)Shell completions for –issue in bash, zsh, and fishissue_ref tests in reducer and daemon roundtripTMUX column (●/○/✗) for session livenessSTATE column from event-derived WorkerStatusNUDGES count and PR number from event logConfigurable interval: jig ps -w 5 for 5s refreshDiscovers workers by scanning event log directoriesReplays events to derive current WorkerState per workerCompares old vs new state to dispatch actionsExecutes nudges via tmux and notifications via hooksPersists state to workers.json between ticksHook wrapper templates that chain jig logic with user hooksRegistry tracking installed hooks at jig-hooks.jsonIdempotent init with backup/restore of existing hooksPost-commit/merge handlers that emit events to worker logsUninstall with rollback to original user hooksEvent schema with typed EventType enum and flat JSONL serializationEventLog append-only reader/writer with per-worker JSONL filesClaude Code hook templates (PostToolUse, Notification, Stop)jig hooks install-claude CLI command to install hooks to ~/.claude/hooks/Detect installation method (script, cargo, source, unknown)Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)Relation mapping was inverted: “blocks” → “is_blocked_by” sodependency checking now correctly identifies blocked issuesRemove unused SearchData typeHide completed issues by default (use –all to include them)Interactive mode (-i) uses alternate screen buffer like less/git diffAdd ui::with_alternate_screen() reusable helperInteractive mode: scrolling, auto indicator, title truncation, G/g navAdd proactive PR discovery: daemon queries GitHub for open PRs onworker branches when pr_url is unknown, emits PrOpened events tomake state durable across restartsCreate per-repo GitHub clients via registry path lookup instead ofambient remote detection (fixes multi-repo daemon)Extract real branch name from spawn events for tmux window lookup(spawn creates windows with slashes, e.g. feature/foo, not dashes)Run all four PR checks (CI, conflicts, reviews, commits) on open PRsNudge on every tick, not just state transitions, so polling daemonretries delivery until max_nudgesCollapse multiline nudge templates to single line before tmux sendto prevent premature submission in TUIsFix tracing init: RUST_LOG now properly overrides default warn levelAdd stderr tick summary in continuous daemon mode for visibilitywithout RUST_LOGAdd debug logging for tmux window misses and notification pipelineState file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 add commit-msg hook for conventional commit validationAdds a commit-msg git hook that validates commit messages against theconventional commits spec using the existing parser and jig.toml config.Closes the conventional-commits-validation issue. add conventional commit validation and examplesAdd parser, configurable validator, and CLI commands: add issue lifecycle commands (create, status, complete, stats)Add CLI subcommands to jig issues for managing issue state:For Linear issues, status changes go through the API (not file editing).Backwards-compatible: jig issues with no subcommand still lists/browses.Also adds “in-progress” (hyphenated) to loose status parsing. add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limitingAdd three display features to make ps -w a proper dashboard:Fix two bugs discovered during implementation: add daemon crash recovery and worker resume show nudge state in jig ps outputAdd NUDGE column between STATE and COMMITS in the ps table.Displays nudge count as count/max (e.g. 2/3), with color coding:grey dash for zero, yellow for in-progress, red for exhausted.Adds max_nudges field to WorkerDisplayInfo so the UI can renderthe denominator from the resolved health config. add per-repo nudge configuration in jig.tomlAdd [health] section to jig.toml supporting per-repo overrides ofsilence_threshold_seconds and max_nudges, plus per-nudge-type[health.nudge.<type>] sections with independent max andcooldown_seconds settings.Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]global config > defaults. When cooldown_seconds is not set, fallsback to silence_threshold_seconds. add jig home command to print base repo rootAdds jig home (alias jig h) that prints the base repository rootpath, enabling cd $(jig home) navigation from worktrees. communicate worker initialization state and on-create failuresAdd Initializing event type and worker status to make the workerlifecycle visible during setup. When the daemon auto-spawns a worker,it now registers the worker as Initializing before running theon-create hook, then transitions to Spawned on success or Failed onhook failure. show auto-spawn config in jig config and add jig issues --autoDisplay auto-spawn settings (enabled, auto-start, max workers, pollinterval, spawn labels) in jig config show with source attribution.Add --auto flag to jig issues to filter to only daemon-eligibleauto-spawn candidates using the existing list_spawnable method. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. move auto-spawn to background thread to keep ps -w responsiveThe on-create hook (e.g. pnpm install) was running synchronously onthe tick thread, freezing the ps –watch UI for the entire duration.Introduces a spawn_actor following the same pattern as prune_actor,issue_actor, etc. The tick now sends spawnable issues to the backgroundthread and drains results on the next tick.Also adds: add labels field for issue tagging and filteringAdd labels: Vec<String> to Issue and IssueFilter types. Linearprovider now passes all label names through from GraphQL (auto fieldderivation unchanged). File provider parses **Labels:** comma-separatedfrontmatter. CLI gains --label/-l flag for filtering (all must match).Shell completions updated for bash, zsh, and fish. add shared UI module with consistent formatting and –plain flagExpand ui.rs into a centralized formatting module with:Migrate all 20 command files from inline colored::Colorize calls toshared ui:: helpers. Add –plain support to list, repos, and issuescommands with tab-separated output for piping. add AUTO column to jig issues table outputShow a green dot indicator for issues tagged for auto-spawn, making itvisible at a glance whether file-provider Auto flag or Linear jig-autolabel is set. support -g/–global flag for attaching from anywhereAdd run_global implementation to the Attach command so users can attachto a worktree from outside the owning repo using jig attach <name> -g.Resolves the owning repo via GlobalCtx::repo_for_worktree. jig init –audit launches agent in tmux to populate docs–audit now spawns the configured agent in a jig-init:<repo> tmuxsession with the audit prompt instead of just printing instructions.–backup enhances the prompt to reference .backup/ files. –auditaccepts an optional string for extra instructions. block auto-spawn on unresolved dependenciesAdd is_spawnable_with_deps() to IssueProvider trait that checks alldepends_on entries resolve to Complete before allowing spawn. Applied inboth FileProvider and LinearProvider’s list_spawnable(). Also adds–blocked/–unblocked flags to jig issues CLI for filtering. group workers by repo in jig ps -g outputAdd repo field to WorkerDisplayInfo and render grouped tables withbold repo headers when running in global mode (jig ps -g / jig ps -gw).Local jig ps output is unchanged. daemon periodically prunes stale worktreesWorkers in terminal state (merged/archived/failed) with dead tmuxsessions now get their git worktrees, event logs, and global stateentries cleaned up automatically. Prune runs every 120s during watchmode. Pruned workers are reported in the tick status and log view.Also includes snake_case fixes for auto-spawn-filtering ticket. default table view for jig ls and pretty grouped jig ls -g show draft vs review state, document PR nudge behaviorWorkers with draft PRs now show “draft” (blue) in the STATE columninstead of “review” (cyan). This makes it visually clear which workerswill receive PR nudges (draft) vs which are in human review (non-draft).Add PR Nudges section to daemon docs explaining the draft/non-draftnudge policy and what each health check means. unify daemon/ps tick loops and add log toggle to watch modeExtract run_with() callback API from daemon so ps –watch shares thesame setup code path instead of duplicating Daemon/Notifier/TmuxClientconstruction. The callback controls inter-tick delay and can signalstop, which enables keypress handling during the sleep window.Add log view toggle to watch mode: press ‘l’ to see timestamped daemonactivity (nudges fired, PR check results, errors), ‘t’ to switch backto the table, ‘q’ to quit cleanly. Uses crossterm raw mode with 100mspoll intervals for responsive input.Also allows spawned workers to transition to stalled (previouslySpawned status was excluded from silence detection). surface PR health in ps –watch displayAdd a HEALTH column to the watch table showing per-worker PR checkresults (ci, conflicts, reviews, commits) so problems are visible at aglance without needing RUST_LOG=debug. Upgrade silent debug-level PRerrors to info-level logging. add –base flag to spawn and create for custom branch baseAllow overriding the default base branch (from jig.toml) per-commandwith –base/-b. Includes shell completions for branch names acrossbash, zsh, and fish. Also fixes spawn status message to show theactual base branch used instead of the current branch. wire issues into spawn pipeline with –issue flagAdd jig spawn --issue <id> to resolve file-based issues and use theirbody as Claude context. Thread issue_ref through the full pipeline:spawn CLI → register() → Spawn event → WorkerState reducer → daemonworkers.json → ps watch table.Also adds: add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.Improve issues command UX: accept trailing args in git hook subcommandsGit passes arguments to hooks (e.g. post-merge receives a squash flag“0” or “1”), which the hook wrapper forwards via “$@”. The CLIsubcommands rejected these unexpected args. Add trailing_var_arg toPostCommit, PostMerge, and PreCommit to accept and ignore them. output cd command from jig home instead of bare pathMatches the pattern used by jig open and jig exit — outputscd '/path' to stdout for shell eval, not just the path. recover from stale git worktree registrations on spawn and pruneWhen a worktree directory is removed but git still tracks the entry,git worktree add fails with “missing but already registered”. Nowcreate_worktree runs git worktree prune first, and prune_actorhandles the missing-directory case instead of skipping cleanup.Also extracts prune_actor into its own module and adds urgent issueto replace git CLI shelling with git2. add issues command to shell completionsThe issues command was missing from all three shells’ command listsand had no flag/argument completions. Adds command entry, issue IDpositional completions, and flag completions (status, priority,category, interactive, ids) for bash, zsh, and fish. use if-let instead of unwrap to satisfy clippy daemon PR discovery, tmux targeting, and nudge delivery register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.6.0 (2026-03-18)

<csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/>
<csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/>
<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-6bd97564a9bf415d58bb81711344a6a68ca27e03/> bump version to 1.6.0
 - <csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/> bump version to 1.5.0
 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

<csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/>
<csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/>
<csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/>
<csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/>
<csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/>
<csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/>
<csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/>
<csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/>
<csr-id-df0a3be811b27f8afce047bd088cad410d09e081/>
<csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/>
<csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/>
<csr-id-bd1a1faeca5a7634224bef836154791819b4903b/>
<csr-id-462f05eaf29929899631125c733738cd8f93e558/>
<csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/>
<csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/>
<csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/>
<csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/>
<csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/>
<csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/>
<csr-id-057e8dc3675610e75e826910d051774f32f63cee/>
<csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/>
<csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/>
<csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/>
<csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/>
<csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/>
<csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/>
<csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/>
<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-99f0311db0fb1fcac156aa2f03e8b04bfdd75199/> add jig.local.toml overlay and fix stderr color detection
   Add deep-merge support for jig.local.toml (gitignored) on top of
   jig.toml, allowing machine-specific overrides without touching the
   committed config. Tables merge recursively; scalars and arrays from the
   local file win. `jig init` auto-adds jig.local.toml to .gitignore.
   
   Revamp `jig config` display with source attribution showing where each
   setting originates (jig.toml, jig.local.toml, global config, or default).
   
   Fix colored crate TTY detection: override based on stderr since all jig
   output goes to stderr, preventing silent color suppression when stdout
   is piped.
 - <csr-id-0f9a72d8d53c686ff305a11c961c37083f26a47d/> add Create event and Created status for bare worktrees
   Distinguishes `jig create` worktrees from `jig spawn` workers via a
   positive Create event rather than relying on empty event logs. The daemon
   discovers Created workers (they appear in `jig list`) but takes no
   actions on them — no auto-resume, no nudges, no recovery.
 - <csr-id-205483042ca48167984c4076b0d01bbd81a00f2f/> derive spawn worktree name from issue, use repo base branch in nudges
   - Make `name` optional in `jig spawn` when `--issue` is provided; derive
   worktree name from the issue's branch_name or ID

### Bug Fixes

<csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/>
<csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/>
<csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/>
<csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/>
<csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/>
<csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/>
<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-847c9ba0e63c6aeb5a46fead6529eca9b8976465/> use `jig -g init` instead of broken `--global` flag on init
   The `--global` flag conflicts with the top-level `-g/--global` CLI flag.
   Implement `run_global()` on Init so `jig -g init` scaffolds the global
   config correctly.
 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter

### Other

 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/> consolidate auto-spawn into single `auto_spawn_labels` field
   Replace three redundant knobs (`spawn.auto`, `spawn.auto_spawn`,
   `issues.spawn_labels`) with a single `issues.auto_spawn_labels` field.
   Semantics: None = disabled, Some([]) = all issues, Some(["x"]) = filtered.
 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELLFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changes<csr-unknown>
<csr-unknown>
Extract Linear identifiers from branch-format strings (e.g.feature/aut-5044-refactor-foo → AUT-5044)Move derive_worker_name/sanitize_worker_name to shared issues::naming modulePass repo’s actual base branch to conflict and bad-commits nudge templatesinstead of hardcoding origin/mainSimplify resume to reuse launch(), remove build_resume_commandSkip recovery for Initializing workers (still running on-create hooks)jig commit validate validates HEAD, specific revs, stdin, or filesjig commit examples shows conventional commit referenceConfigurable via [commits] section in jig.toml16 unit tests + 12 integration testsjig issues create creates issues from templates with title, priority, category, labelsjig issues status <id> --status <new> updates frontmatter status in file issuesjig issues complete <id> marks issues as complete, with optional –deletejig issues stats shows breakdown by status and priorityNudge cooldown countdown in NUDGE column (e.g. “2/3 (3m12s)”)Nudge messages rendered below worker table when deliveredGlobal sync/poll timer footer alongside keybinding hintsPreserve draft PR status on GitHub API errors so nudges aren’tsilently suppressed for known-draft PRsThrottle GitHub API requests to once per 60s per worker, alignedwith gh –cache 60s TTL, reducing API pressure ~30xInstall SIGTERM/SIGINT handler for graceful daemon shutdownRecord Started/Stopped lifecycle events in daemon.jsonlDetect unclean shutdown on next startup (missing Stopped event)Auto-recover orphaned workers on daemon startup via Worktree::resume()Add jig resume <name> CLI command for manual worker recoveryDetect dead tmux windows during steady-state ticks and auto-resumeAdd auto_recover global config option (default: true) for opt-outWire Action::Restart to use recovery::try_resume_worker()Add integration tests for jig resume commandAdd RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structsAdd ResolvedNudgeConfig with resolver for per-type configThread resolved config through nudge classify, dispatch, and executeApply per-type cooldown to both idle/stalled nudges and PR nudgesDisplay effective nudge config in jig config showFixes PR nudge burst bug by enforcing per-type cooldownsSpawning worker names shown below the ps table during setupWorkerStatus::Initializing variant for future usespawn_labels config in jig.tomlThree new issues (config-show-auto-spawn, worker-initializing-state,auto-column-checkmark)Status symbol constants (✓, →, ✗, !)Formatted output helpers (success, progress, failure, warning, detail, header)Color helpers (highlight, bold, dim) that respect plain modeTable builder helper (new_table) for consistent table creationGlobal –plain flag for scriptable output (no colors, no decorations)Error display with cause chain formattingjig ls now shows a table with name, branch, and commits aheadjig ls -g shows tables grouped by repo with bold headersAdd --plain/-p flag for bare name output (old behavior)Shell completions use --plain and fall back to -gp outside a repoBranch column only shown when it differs from worktree namejig issues CLI command with –ids flag for scriptingIssuesConfig in jig.toml for configurable issues directoryISSUE column in ps –watch table (shortened last path segment)Shell completions for –issue in bash, zsh, and fishissue_ref tests in reducer and daemon roundtripTMUX column (●/○/✗) for session livenessSTATE column from event-derived WorkerStatusNUDGES count and PR number from event logConfigurable interval: jig ps -w 5 for 5s refreshDiscovers workers by scanning event log directoriesReplays events to derive current WorkerState per workerCompares old vs new state to dispatch actionsExecutes nudges via tmux and notifications via hooksPersists state to workers.json between ticksHook wrapper templates that chain jig logic with user hooksRegistry tracking installed hooks at jig-hooks.jsonIdempotent init with backup/restore of existing hooksPost-commit/merge handlers that emit events to worker logsUninstall with rollback to original user hooksEvent schema with typed EventType enum and flat JSONL serializationEventLog append-only reader/writer with per-worker JSONL filesClaude Code hook templates (PostToolUse, Notification, Stop)jig hooks install-claude CLI command to install hooks to ~/.claude/hooks/Detect installation method (script, cargo, source, unknown)Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)Relation mapping was inverted: “blocks” → “is_blocked_by” sodependency checking now correctly identifies blocked issuesRemove unused SearchData typeHide completed issues by default (use –all to include them)Interactive mode (-i) uses alternate screen buffer like less/git diffAdd ui::with_alternate_screen() reusable helperInteractive mode: scrolling, auto indicator, title truncation, G/g navAdd proactive PR discovery: daemon queries GitHub for open PRs onworker branches when pr_url is unknown, emits PrOpened events tomake state durable across restartsCreate per-repo GitHub clients via registry path lookup instead ofambient remote detection (fixes multi-repo daemon)Extract real branch name from spawn events for tmux window lookup(spawn creates windows with slashes, e.g. feature/foo, not dashes)Run all four PR checks (CI, conflicts, reviews, commits) on open PRsNudge on every tick, not just state transitions, so polling daemonretries delivery until max_nudgesCollapse multiline nudge templates to single line before tmux sendto prevent premature submission in TUIsFix tracing init: RUST_LOG now properly overrides default warn levelAdd stderr tick summary in continuous daemon mode for visibilitywithout RUST_LOGAdd debug logging for tmux window misses and notification pipelineState file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 add commit-msg hook for conventional commit validationAdds a commit-msg git hook that validates commit messages against theconventional commits spec using the existing parser and jig.toml config.Closes the conventional-commits-validation issue. add conventional commit validation and examplesAdd parser, configurable validator, and CLI commands: add issue lifecycle commands (create, status, complete, stats)Add CLI subcommands to jig issues for managing issue state:For Linear issues, status changes go through the API (not file editing).Backwards-compatible: jig issues with no subcommand still lists/browses.Also adds “in-progress” (hyphenated) to loose status parsing. add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limitingAdd three display features to make ps -w a proper dashboard:Fix two bugs discovered during implementation: add daemon crash recovery and worker resume show nudge state in jig ps outputAdd NUDGE column between STATE and COMMITS in the ps table.Displays nudge count as count/max (e.g. 2/3), with color coding:grey dash for zero, yellow for in-progress, red for exhausted.Adds max_nudges field to WorkerDisplayInfo so the UI can renderthe denominator from the resolved health config. add per-repo nudge configuration in jig.tomlAdd [health] section to jig.toml supporting per-repo overrides ofsilence_threshold_seconds and max_nudges, plus per-nudge-type[health.nudge.<type>] sections with independent max andcooldown_seconds settings.Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]global config > defaults. When cooldown_seconds is not set, fallsback to silence_threshold_seconds. add jig home command to print base repo rootAdds jig home (alias jig h) that prints the base repository rootpath, enabling cd $(jig home) navigation from worktrees. communicate worker initialization state and on-create failuresAdd Initializing event type and worker status to make the workerlifecycle visible during setup. When the daemon auto-spawns a worker,it now registers the worker as Initializing before running theon-create hook, then transitions to Spawned on success or Failed onhook failure. show auto-spawn config in jig config and add jig issues --autoDisplay auto-spawn settings (enabled, auto-start, max workers, pollinterval, spawn labels) in jig config show with source attribution.Add --auto flag to jig issues to filter to only daemon-eligibleauto-spawn candidates using the existing list_spawnable method. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. move auto-spawn to background thread to keep ps -w responsiveThe on-create hook (e.g. pnpm install) was running synchronously onthe tick thread, freezing the ps –watch UI for the entire duration.Introduces a spawn_actor following the same pattern as prune_actor,issue_actor, etc. The tick now sends spawnable issues to the backgroundthread and drains results on the next tick.Also adds: add labels field for issue tagging and filteringAdd labels: Vec<String> to Issue and IssueFilter types. Linearprovider now passes all label names through from GraphQL (auto fieldderivation unchanged). File provider parses **Labels:** comma-separatedfrontmatter. CLI gains --label/-l flag for filtering (all must match).Shell completions updated for bash, zsh, and fish. add shared UI module with consistent formatting and –plain flagExpand ui.rs into a centralized formatting module with:Migrate all 20 command files from inline colored::Colorize calls toshared ui:: helpers. Add –plain support to list, repos, and issuescommands with tab-separated output for piping. add AUTO column to jig issues table outputShow a green dot indicator for issues tagged for auto-spawn, making itvisible at a glance whether file-provider Auto flag or Linear jig-autolabel is set. support -g/–global flag for attaching from anywhereAdd run_global implementation to the Attach command so users can attachto a worktree from outside the owning repo using jig attach <name> -g.Resolves the owning repo via GlobalCtx::repo_for_worktree. jig init –audit launches agent in tmux to populate docs–audit now spawns the configured agent in a jig-init:<repo> tmuxsession with the audit prompt instead of just printing instructions.–backup enhances the prompt to reference .backup/ files. –auditaccepts an optional string for extra instructions. block auto-spawn on unresolved dependenciesAdd is_spawnable_with_deps() to IssueProvider trait that checks alldepends_on entries resolve to Complete before allowing spawn. Applied inboth FileProvider and LinearProvider’s list_spawnable(). Also adds–blocked/–unblocked flags to jig issues CLI for filtering. group workers by repo in jig ps -g outputAdd repo field to WorkerDisplayInfo and render grouped tables withbold repo headers when running in global mode (jig ps -g / jig ps -gw).Local jig ps output is unchanged. daemon periodically prunes stale worktreesWorkers in terminal state (merged/archived/failed) with dead tmuxsessions now get their git worktrees, event logs, and global stateentries cleaned up automatically. Prune runs every 120s during watchmode. Pruned workers are reported in the tick status and log view.Also includes snake_case fixes for auto-spawn-filtering ticket. default table view for jig ls and pretty grouped jig ls -g show draft vs review state, document PR nudge behaviorWorkers with draft PRs now show “draft” (blue) in the STATE columninstead of “review” (cyan). This makes it visually clear which workerswill receive PR nudges (draft) vs which are in human review (non-draft).Add PR Nudges section to daemon docs explaining the draft/non-draftnudge policy and what each health check means. unify daemon/ps tick loops and add log toggle to watch modeExtract run_with() callback API from daemon so ps –watch shares thesame setup code path instead of duplicating Daemon/Notifier/TmuxClientconstruction. The callback controls inter-tick delay and can signalstop, which enables keypress handling during the sleep window.Add log view toggle to watch mode: press ‘l’ to see timestamped daemonactivity (nudges fired, PR check results, errors), ‘t’ to switch backto the table, ‘q’ to quit cleanly. Uses crossterm raw mode with 100mspoll intervals for responsive input.Also allows spawned workers to transition to stalled (previouslySpawned status was excluded from silence detection). surface PR health in ps –watch displayAdd a HEALTH column to the watch table showing per-worker PR checkresults (ci, conflicts, reviews, commits) so problems are visible at aglance without needing RUST_LOG=debug. Upgrade silent debug-level PRerrors to info-level logging. add –base flag to spawn and create for custom branch baseAllow overriding the default base branch (from jig.toml) per-commandwith –base/-b. Includes shell completions for branch names acrossbash, zsh, and fish. Also fixes spawn status message to show theactual base branch used instead of the current branch. wire issues into spawn pipeline with –issue flagAdd jig spawn --issue <id> to resolve file-based issues and use theirbody as Claude context. Thread issue_ref through the full pipeline:spawn CLI → register() → Spawn event → WorkerState reducer → daemonworkers.json → ps watch table.Also adds: add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.Improve issues command UX: accept trailing args in git hook subcommandsGit passes arguments to hooks (e.g. post-merge receives a squash flag“0” or “1”), which the hook wrapper forwards via “$@”. The CLIsubcommands rejected these unexpected args. Add trailing_var_arg toPostCommit, PostMerge, and PreCommit to accept and ignore them. output cd command from jig home instead of bare pathMatches the pattern used by jig open and jig exit — outputscd '/path' to stdout for shell eval, not just the path. recover from stale git worktree registrations on spawn and pruneWhen a worktree directory is removed but git still tracks the entry,git worktree add fails with “missing but already registered”. Nowcreate_worktree runs git worktree prune first, and prune_actorhandles the missing-directory case instead of skipping cleanup.Also extracts prune_actor into its own module and adds urgent issueto replace git CLI shelling with git2. add issues command to shell completionsThe issues command was missing from all three shells’ command listsand had no flag/argument completions. Adds command entry, issue IDpositional completions, and flag completions (status, priority,category, interactive, ids) for bash, zsh, and fish. use if-let instead of unwrap to satisfy clippy daemon PR discovery, tmux targeting, and nudge delivery register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.5.0 (2026-03-18)

<csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/>
<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-ae3558092ac509c1e5b495dca6c48fb11c6a18c6/> bump version to 1.5.0
 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

<csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/>
<csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/>
<csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/>
<csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/>
<csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/>
<csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/>
<csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/>
<csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/>
<csr-id-df0a3be811b27f8afce047bd088cad410d09e081/>
<csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/>
<csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/>
<csr-id-bd1a1faeca5a7634224bef836154791819b4903b/>
<csr-id-462f05eaf29929899631125c733738cd8f93e558/>
<csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/>
<csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/>
<csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/>
<csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/>
<csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/>
<csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/>
<csr-id-057e8dc3675610e75e826910d051774f32f63cee/>
<csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/>
<csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/>
<csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/>
<csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/>
<csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/>
<csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/>
<csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/>
<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-99f0311db0fb1fcac156aa2f03e8b04bfdd75199/> add jig.local.toml overlay and fix stderr color detection
   Add deep-merge support for jig.local.toml (gitignored) on top of
   jig.toml, allowing machine-specific overrides without touching the
   committed config. Tables merge recursively; scalars and arrays from the
   local file win. `jig init` auto-adds jig.local.toml to .gitignore.
   
   Revamp `jig config` display with source attribution showing where each
   setting originates (jig.toml, jig.local.toml, global config, or default).
   
   Fix colored crate TTY detection: override based on stderr since all jig
   output goes to stderr, preventing silent color suppression when stdout
   is piped.
 - <csr-id-0f9a72d8d53c686ff305a11c961c37083f26a47d/> add Create event and Created status for bare worktrees
   Distinguishes `jig create` worktrees from `jig spawn` workers via a
   positive Create event rather than relying on empty event logs. The daemon
   discovers Created workers (they appear in `jig list`) but takes no
   actions on them — no auto-resume, no nudges, no recovery.
 - <csr-id-205483042ca48167984c4076b0d01bbd81a00f2f/> derive spawn worktree name from issue, use repo base branch in nudges
   - Make `name` optional in `jig spawn` when `--issue` is provided; derive
   worktree name from the issue's branch_name or ID

### Bug Fixes

<csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/>
<csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/>
<csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/>
<csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/>
<csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/>
<csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/>
<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-847c9ba0e63c6aeb5a46fead6529eca9b8976465/> use `jig -g init` instead of broken `--global` flag on init
   The `--global` flag conflicts with the top-level `-g/--global` CLI flag.
   Implement `run_global()` on Init so `jig -g init` scaffolds the global
   config correctly.
 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter

### Other

 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-cc0f4f9dd73d03781dade885456d00d0dab790b7/> consolidate auto-spawn into single `auto_spawn_labels` field
   Replace three redundant knobs (`spawn.auto`, `spawn.auto_spawn`,
   `issues.spawn_labels`) with a single `issues.auto_spawn_labels` field.
   Semantics: None = disabled, Some([]) = all issues, Some(["x"]) = filtered.
 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELLFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changes<csr-unknown>
<csr-unknown>
Extract Linear identifiers from branch-format strings (e.g.feature/aut-5044-refactor-foo → AUT-5044)Move derive_worker_name/sanitize_worker_name to shared issues::naming modulePass repo’s actual base branch to conflict and bad-commits nudge templatesinstead of hardcoding origin/mainSimplify resume to reuse launch(), remove build_resume_commandSkip recovery for Initializing workers (still running on-create hooks)jig commit validate validates HEAD, specific revs, stdin, or filesjig commit examples shows conventional commit referenceConfigurable via [commits] section in jig.toml16 unit tests + 12 integration testsjig issues create creates issues from templates with title, priority, category, labelsjig issues status <id> --status <new> updates frontmatter status in file issuesjig issues complete <id> marks issues as complete, with optional –deletejig issues stats shows breakdown by status and priorityNudge cooldown countdown in NUDGE column (e.g. “2/3 (3m12s)”)Nudge messages rendered below worker table when deliveredGlobal sync/poll timer footer alongside keybinding hintsPreserve draft PR status on GitHub API errors so nudges aren’tsilently suppressed for known-draft PRsThrottle GitHub API requests to once per 60s per worker, alignedwith gh –cache 60s TTL, reducing API pressure ~30xInstall SIGTERM/SIGINT handler for graceful daemon shutdownRecord Started/Stopped lifecycle events in daemon.jsonlDetect unclean shutdown on next startup (missing Stopped event)Auto-recover orphaned workers on daemon startup via Worktree::resume()Add jig resume <name> CLI command for manual worker recoveryDetect dead tmux windows during steady-state ticks and auto-resumeAdd auto_recover global config option (default: true) for opt-outWire Action::Restart to use recovery::try_resume_worker()Add integration tests for jig resume commandAdd RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structsAdd ResolvedNudgeConfig with resolver for per-type configThread resolved config through nudge classify, dispatch, and executeApply per-type cooldown to both idle/stalled nudges and PR nudgesDisplay effective nudge config in jig config showFixes PR nudge burst bug by enforcing per-type cooldownsSpawning worker names shown below the ps table during setupWorkerStatus::Initializing variant for future usespawn_labels config in jig.tomlThree new issues (config-show-auto-spawn, worker-initializing-state,auto-column-checkmark)Status symbol constants (✓, →, ✗, !)Formatted output helpers (success, progress, failure, warning, detail, header)Color helpers (highlight, bold, dim) that respect plain modeTable builder helper (new_table) for consistent table creationGlobal –plain flag for scriptable output (no colors, no decorations)Error display with cause chain formattingjig ls now shows a table with name, branch, and commits aheadjig ls -g shows tables grouped by repo with bold headersAdd --plain/-p flag for bare name output (old behavior)Shell completions use --plain and fall back to -gp outside a repoBranch column only shown when it differs from worktree namejig issues CLI command with –ids flag for scriptingIssuesConfig in jig.toml for configurable issues directoryISSUE column in ps –watch table (shortened last path segment)Shell completions for –issue in bash, zsh, and fishissue_ref tests in reducer and daemon roundtripTMUX column (●/○/✗) for session livenessSTATE column from event-derived WorkerStatusNUDGES count and PR number from event logConfigurable interval: jig ps -w 5 for 5s refreshDiscovers workers by scanning event log directoriesReplays events to derive current WorkerState per workerCompares old vs new state to dispatch actionsExecutes nudges via tmux and notifications via hooksPersists state to workers.json between ticksHook wrapper templates that chain jig logic with user hooksRegistry tracking installed hooks at jig-hooks.jsonIdempotent init with backup/restore of existing hooksPost-commit/merge handlers that emit events to worker logsUninstall with rollback to original user hooksEvent schema with typed EventType enum and flat JSONL serializationEventLog append-only reader/writer with per-worker JSONL filesClaude Code hook templates (PostToolUse, Notification, Stop)jig hooks install-claude CLI command to install hooks to ~/.claude/hooks/Detect installation method (script, cargo, source, unknown)Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)Relation mapping was inverted: “blocks” → “is_blocked_by” sodependency checking now correctly identifies blocked issuesRemove unused SearchData typeHide completed issues by default (use –all to include them)Interactive mode (-i) uses alternate screen buffer like less/git diffAdd ui::with_alternate_screen() reusable helperInteractive mode: scrolling, auto indicator, title truncation, G/g navAdd proactive PR discovery: daemon queries GitHub for open PRs onworker branches when pr_url is unknown, emits PrOpened events tomake state durable across restartsCreate per-repo GitHub clients via registry path lookup instead ofambient remote detection (fixes multi-repo daemon)Extract real branch name from spawn events for tmux window lookup(spawn creates windows with slashes, e.g. feature/foo, not dashes)Run all four PR checks (CI, conflicts, reviews, commits) on open PRsNudge on every tick, not just state transitions, so polling daemonretries delivery until max_nudgesCollapse multiline nudge templates to single line before tmux sendto prevent premature submission in TUIsFix tracing init: RUST_LOG now properly overrides default warn levelAdd stderr tick summary in continuous daemon mode for visibilitywithout RUST_LOGAdd debug logging for tmux window misses and notification pipelineState file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 add commit-msg hook for conventional commit validationAdds a commit-msg git hook that validates commit messages against theconventional commits spec using the existing parser and jig.toml config.Closes the conventional-commits-validation issue. add conventional commit validation and examplesAdd parser, configurable validator, and CLI commands: add issue lifecycle commands (create, status, complete, stats)Add CLI subcommands to jig issues for managing issue state:For Linear issues, status changes go through the API (not file editing).Backwards-compatible: jig issues with no subcommand still lists/browses.Also adds “in-progress” (hyphenated) to loose status parsing. add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limitingAdd three display features to make ps -w a proper dashboard:Fix two bugs discovered during implementation: add daemon crash recovery and worker resume show nudge state in jig ps outputAdd NUDGE column between STATE and COMMITS in the ps table.Displays nudge count as count/max (e.g. 2/3), with color coding:grey dash for zero, yellow for in-progress, red for exhausted.Adds max_nudges field to WorkerDisplayInfo so the UI can renderthe denominator from the resolved health config. add per-repo nudge configuration in jig.tomlAdd [health] section to jig.toml supporting per-repo overrides ofsilence_threshold_seconds and max_nudges, plus per-nudge-type[health.nudge.<type>] sections with independent max andcooldown_seconds settings.Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]global config > defaults. When cooldown_seconds is not set, fallsback to silence_threshold_seconds. add jig home command to print base repo rootAdds jig home (alias jig h) that prints the base repository rootpath, enabling cd $(jig home) navigation from worktrees. communicate worker initialization state and on-create failuresAdd Initializing event type and worker status to make the workerlifecycle visible during setup. When the daemon auto-spawns a worker,it now registers the worker as Initializing before running theon-create hook, then transitions to Spawned on success or Failed onhook failure. show auto-spawn config in jig config and add jig issues --autoDisplay auto-spawn settings (enabled, auto-start, max workers, pollinterval, spawn labels) in jig config show with source attribution.Add --auto flag to jig issues to filter to only daemon-eligibleauto-spawn candidates using the existing list_spawnable method. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. move auto-spawn to background thread to keep ps -w responsiveThe on-create hook (e.g. pnpm install) was running synchronously onthe tick thread, freezing the ps –watch UI for the entire duration.Introduces a spawn_actor following the same pattern as prune_actor,issue_actor, etc. The tick now sends spawnable issues to the backgroundthread and drains results on the next tick.Also adds: add labels field for issue tagging and filteringAdd labels: Vec<String> to Issue and IssueFilter types. Linearprovider now passes all label names through from GraphQL (auto fieldderivation unchanged). File provider parses **Labels:** comma-separatedfrontmatter. CLI gains --label/-l flag for filtering (all must match).Shell completions updated for bash, zsh, and fish. add shared UI module with consistent formatting and –plain flagExpand ui.rs into a centralized formatting module with:Migrate all 20 command files from inline colored::Colorize calls toshared ui:: helpers. Add –plain support to list, repos, and issuescommands with tab-separated output for piping. add AUTO column to jig issues table outputShow a green dot indicator for issues tagged for auto-spawn, making itvisible at a glance whether file-provider Auto flag or Linear jig-autolabel is set. support -g/–global flag for attaching from anywhereAdd run_global implementation to the Attach command so users can attachto a worktree from outside the owning repo using jig attach <name> -g.Resolves the owning repo via GlobalCtx::repo_for_worktree. jig init –audit launches agent in tmux to populate docs–audit now spawns the configured agent in a jig-init:<repo> tmuxsession with the audit prompt instead of just printing instructions.–backup enhances the prompt to reference .backup/ files. –auditaccepts an optional string for extra instructions. block auto-spawn on unresolved dependenciesAdd is_spawnable_with_deps() to IssueProvider trait that checks alldepends_on entries resolve to Complete before allowing spawn. Applied inboth FileProvider and LinearProvider’s list_spawnable(). Also adds–blocked/–unblocked flags to jig issues CLI for filtering. group workers by repo in jig ps -g outputAdd repo field to WorkerDisplayInfo and render grouped tables withbold repo headers when running in global mode (jig ps -g / jig ps -gw).Local jig ps output is unchanged. daemon periodically prunes stale worktreesWorkers in terminal state (merged/archived/failed) with dead tmuxsessions now get their git worktrees, event logs, and global stateentries cleaned up automatically. Prune runs every 120s during watchmode. Pruned workers are reported in the tick status and log view.Also includes snake_case fixes for auto-spawn-filtering ticket. default table view for jig ls and pretty grouped jig ls -g show draft vs review state, document PR nudge behaviorWorkers with draft PRs now show “draft” (blue) in the STATE columninstead of “review” (cyan). This makes it visually clear which workerswill receive PR nudges (draft) vs which are in human review (non-draft).Add PR Nudges section to daemon docs explaining the draft/non-draftnudge policy and what each health check means. unify daemon/ps tick loops and add log toggle to watch modeExtract run_with() callback API from daemon so ps –watch shares thesame setup code path instead of duplicating Daemon/Notifier/TmuxClientconstruction. The callback controls inter-tick delay and can signalstop, which enables keypress handling during the sleep window.Add log view toggle to watch mode: press ‘l’ to see timestamped daemonactivity (nudges fired, PR check results, errors), ‘t’ to switch backto the table, ‘q’ to quit cleanly. Uses crossterm raw mode with 100mspoll intervals for responsive input.Also allows spawned workers to transition to stalled (previouslySpawned status was excluded from silence detection). surface PR health in ps –watch displayAdd a HEALTH column to the watch table showing per-worker PR checkresults (ci, conflicts, reviews, commits) so problems are visible at aglance without needing RUST_LOG=debug. Upgrade silent debug-level PRerrors to info-level logging. add –base flag to spawn and create for custom branch baseAllow overriding the default base branch (from jig.toml) per-commandwith –base/-b. Includes shell completions for branch names acrossbash, zsh, and fish. Also fixes spawn status message to show theactual base branch used instead of the current branch. wire issues into spawn pipeline with –issue flagAdd jig spawn --issue <id> to resolve file-based issues and use theirbody as Claude context. Thread issue_ref through the full pipeline:spawn CLI → register() → Spawn event → WorkerState reducer → daemonworkers.json → ps watch table.Also adds: add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.Improve issues command UX: accept trailing args in git hook subcommandsGit passes arguments to hooks (e.g. post-merge receives a squash flag“0” or “1”), which the hook wrapper forwards via “$@”. The CLIsubcommands rejected these unexpected args. Add trailing_var_arg toPostCommit, PostMerge, and PreCommit to accept and ignore them. output cd command from jig home instead of bare pathMatches the pattern used by jig open and jig exit — outputscd '/path' to stdout for shell eval, not just the path. recover from stale git worktree registrations on spawn and pruneWhen a worktree directory is removed but git still tracks the entry,git worktree add fails with “missing but already registered”. Nowcreate_worktree runs git worktree prune first, and prune_actorhandles the missing-directory case instead of skipping cleanup.Also extracts prune_actor into its own module and adds urgent issueto replace git CLI shelling with git2. add issues command to shell completionsThe issues command was missing from all three shells’ command listsand had no flag/argument completions. Adds command entry, issue IDpositional completions, and flag completions (status, priority,category, interactive, ids) for bash, zsh, and fish. use if-let instead of unwrap to satisfy clippy daemon PR discovery, tmux targeting, and nudge delivery register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.4.0 (2026-03-16)

<csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/>
<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-50cd69bab62ce85984779d5fe50378ddebecb350/> bump version to 1.4.0
 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

<csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/>
<csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/>
<csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/>
<csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/>
<csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/>
<csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/>
<csr-id-df0a3be811b27f8afce047bd088cad410d09e081/>
<csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/>
<csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/>
<csr-id-bd1a1faeca5a7634224bef836154791819b4903b/>
<csr-id-462f05eaf29929899631125c733738cd8f93e558/>
<csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/>
<csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/>
<csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/>
<csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/>
<csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/>
<csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/>
<csr-id-057e8dc3675610e75e826910d051774f32f63cee/>
<csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/>
<csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/>
<csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/>
<csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/>
<csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/>
<csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/>
<csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/>
<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/> add commit-msg hook for conventional commit validation
   Adds a commit-msg git hook that validates commit messages against the
   conventional commits spec using the existing parser and jig.toml config.
   Closes the conventional-commits-validation issue.
 - <csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/> add conventional commit validation and examples
   Add parser, configurable validator, and CLI commands:
   - `jig commit validate` validates HEAD, specific revs, stdin, or files

### Bug Fixes

<csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/>
<csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/>
<csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/>
<csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/>
<csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/>
<csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/>
<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter

### Other

 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELLFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changes<csr-unknown>
<csr-unknown>
jig commit examples shows conventional commit referenceConfigurable via [commits] section in jig.toml16 unit tests + 12 integration testsjig issues create creates issues from templates with title, priority, category, labelsjig issues status <id> --status <new> updates frontmatter status in file issuesjig issues complete <id> marks issues as complete, with optional –deletejig issues stats shows breakdown by status and priorityNudge cooldown countdown in NUDGE column (e.g. “2/3 (3m12s)”)Nudge messages rendered below worker table when deliveredGlobal sync/poll timer footer alongside keybinding hintsPreserve draft PR status on GitHub API errors so nudges aren’tsilently suppressed for known-draft PRsThrottle GitHub API requests to once per 60s per worker, alignedwith gh –cache 60s TTL, reducing API pressure ~30xInstall SIGTERM/SIGINT handler for graceful daemon shutdownRecord Started/Stopped lifecycle events in daemon.jsonlDetect unclean shutdown on next startup (missing Stopped event)Auto-recover orphaned workers on daemon startup via Worktree::resume()Add jig resume <name> CLI command for manual worker recoveryDetect dead tmux windows during steady-state ticks and auto-resumeAdd auto_recover global config option (default: true) for opt-outWire Action::Restart to use recovery::try_resume_worker()Add integration tests for jig resume commandAdd RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structsAdd ResolvedNudgeConfig with resolver for per-type configThread resolved config through nudge classify, dispatch, and executeApply per-type cooldown to both idle/stalled nudges and PR nudgesDisplay effective nudge config in jig config showFixes PR nudge burst bug by enforcing per-type cooldownsSpawning worker names shown below the ps table during setupWorkerStatus::Initializing variant for future usespawn_labels config in jig.tomlThree new issues (config-show-auto-spawn, worker-initializing-state,auto-column-checkmark)Status symbol constants (✓, →, ✗, !)Formatted output helpers (success, progress, failure, warning, detail, header)Color helpers (highlight, bold, dim) that respect plain modeTable builder helper (new_table) for consistent table creationGlobal –plain flag for scriptable output (no colors, no decorations)Error display with cause chain formattingjig ls now shows a table with name, branch, and commits aheadjig ls -g shows tables grouped by repo with bold headersAdd --plain/-p flag for bare name output (old behavior)Shell completions use --plain and fall back to -gp outside a repoBranch column only shown when it differs from worktree namejig issues CLI command with –ids flag for scriptingIssuesConfig in jig.toml for configurable issues directoryISSUE column in ps –watch table (shortened last path segment)Shell completions for –issue in bash, zsh, and fishissue_ref tests in reducer and daemon roundtripTMUX column (●/○/✗) for session livenessSTATE column from event-derived WorkerStatusNUDGES count and PR number from event logConfigurable interval: jig ps -w 5 for 5s refreshDiscovers workers by scanning event log directoriesReplays events to derive current WorkerState per workerCompares old vs new state to dispatch actionsExecutes nudges via tmux and notifications via hooksPersists state to workers.json between ticksHook wrapper templates that chain jig logic with user hooksRegistry tracking installed hooks at jig-hooks.jsonIdempotent init with backup/restore of existing hooksPost-commit/merge handlers that emit events to worker logsUninstall with rollback to original user hooksEvent schema with typed EventType enum and flat JSONL serializationEventLog append-only reader/writer with per-worker JSONL filesClaude Code hook templates (PostToolUse, Notification, Stop)jig hooks install-claude CLI command to install hooks to ~/.claude/hooks/Detect installation method (script, cargo, source, unknown)Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)Relation mapping was inverted: “blocks” → “is_blocked_by” sodependency checking now correctly identifies blocked issuesRemove unused SearchData typeHide completed issues by default (use –all to include them)Interactive mode (-i) uses alternate screen buffer like less/git diffAdd ui::with_alternate_screen() reusable helperInteractive mode: scrolling, auto indicator, title truncation, G/g navAdd proactive PR discovery: daemon queries GitHub for open PRs onworker branches when pr_url is unknown, emits PrOpened events tomake state durable across restartsCreate per-repo GitHub clients via registry path lookup instead ofambient remote detection (fixes multi-repo daemon)Extract real branch name from spawn events for tmux window lookup(spawn creates windows with slashes, e.g. feature/foo, not dashes)Run all four PR checks (CI, conflicts, reviews, commits) on open PRsNudge on every tick, not just state transitions, so polling daemonretries delivery until max_nudgesCollapse multiline nudge templates to single line before tmux sendto prevent premature submission in TUIsFix tracing init: RUST_LOG now properly overrides default warn levelAdd stderr tick summary in continuous daemon mode for visibilitywithout RUST_LOGAdd debug logging for tmux window misses and notification pipelineState file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 add issue lifecycle commands (create, status, complete, stats)Add CLI subcommands to jig issues for managing issue state:For Linear issues, status changes go through the API (not file editing).Backwards-compatible: jig issues with no subcommand still lists/browses.Also adds “in-progress” (hyphenated) to loose status parsing. add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limitingAdd three display features to make ps -w a proper dashboard:Fix two bugs discovered during implementation: add daemon crash recovery and worker resume show nudge state in jig ps outputAdd NUDGE column between STATE and COMMITS in the ps table.Displays nudge count as count/max (e.g. 2/3), with color coding:grey dash for zero, yellow for in-progress, red for exhausted.Adds max_nudges field to WorkerDisplayInfo so the UI can renderthe denominator from the resolved health config. add per-repo nudge configuration in jig.tomlAdd [health] section to jig.toml supporting per-repo overrides ofsilence_threshold_seconds and max_nudges, plus per-nudge-type[health.nudge.<type>] sections with independent max andcooldown_seconds settings.Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]global config > defaults. When cooldown_seconds is not set, fallsback to silence_threshold_seconds. add jig home command to print base repo rootAdds jig home (alias jig h) that prints the base repository rootpath, enabling cd $(jig home) navigation from worktrees. communicate worker initialization state and on-create failuresAdd Initializing event type and worker status to make the workerlifecycle visible during setup. When the daemon auto-spawns a worker,it now registers the worker as Initializing before running theon-create hook, then transitions to Spawned on success or Failed onhook failure. show auto-spawn config in jig config and add jig issues --autoDisplay auto-spawn settings (enabled, auto-start, max workers, pollinterval, spawn labels) in jig config show with source attribution.Add --auto flag to jig issues to filter to only daemon-eligibleauto-spawn candidates using the existing list_spawnable method. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. move auto-spawn to background thread to keep ps -w responsiveThe on-create hook (e.g. pnpm install) was running synchronously onthe tick thread, freezing the ps –watch UI for the entire duration.Introduces a spawn_actor following the same pattern as prune_actor,issue_actor, etc. The tick now sends spawnable issues to the backgroundthread and drains results on the next tick.Also adds: add labels field for issue tagging and filteringAdd labels: Vec<String> to Issue and IssueFilter types. Linearprovider now passes all label names through from GraphQL (auto fieldderivation unchanged). File provider parses **Labels:** comma-separatedfrontmatter. CLI gains --label/-l flag for filtering (all must match).Shell completions updated for bash, zsh, and fish. add shared UI module with consistent formatting and –plain flagExpand ui.rs into a centralized formatting module with:Migrate all 20 command files from inline colored::Colorize calls toshared ui:: helpers. Add –plain support to list, repos, and issuescommands with tab-separated output for piping. add AUTO column to jig issues table outputShow a green dot indicator for issues tagged for auto-spawn, making itvisible at a glance whether file-provider Auto flag or Linear jig-autolabel is set. support -g/–global flag for attaching from anywhereAdd run_global implementation to the Attach command so users can attachto a worktree from outside the owning repo using jig attach <name> -g.Resolves the owning repo via GlobalCtx::repo_for_worktree. jig init –audit launches agent in tmux to populate docs–audit now spawns the configured agent in a jig-init:<repo> tmuxsession with the audit prompt instead of just printing instructions.–backup enhances the prompt to reference .backup/ files. –auditaccepts an optional string for extra instructions. block auto-spawn on unresolved dependenciesAdd is_spawnable_with_deps() to IssueProvider trait that checks alldepends_on entries resolve to Complete before allowing spawn. Applied inboth FileProvider and LinearProvider’s list_spawnable(). Also adds–blocked/–unblocked flags to jig issues CLI for filtering. group workers by repo in jig ps -g outputAdd repo field to WorkerDisplayInfo and render grouped tables withbold repo headers when running in global mode (jig ps -g / jig ps -gw).Local jig ps output is unchanged. daemon periodically prunes stale worktreesWorkers in terminal state (merged/archived/failed) with dead tmuxsessions now get their git worktrees, event logs, and global stateentries cleaned up automatically. Prune runs every 120s during watchmode. Pruned workers are reported in the tick status and log view.Also includes snake_case fixes for auto-spawn-filtering ticket. default table view for jig ls and pretty grouped jig ls -g show draft vs review state, document PR nudge behaviorWorkers with draft PRs now show “draft” (blue) in the STATE columninstead of “review” (cyan). This makes it visually clear which workerswill receive PR nudges (draft) vs which are in human review (non-draft).Add PR Nudges section to daemon docs explaining the draft/non-draftnudge policy and what each health check means. unify daemon/ps tick loops and add log toggle to watch modeExtract run_with() callback API from daemon so ps –watch shares thesame setup code path instead of duplicating Daemon/Notifier/TmuxClientconstruction. The callback controls inter-tick delay and can signalstop, which enables keypress handling during the sleep window.Add log view toggle to watch mode: press ‘l’ to see timestamped daemonactivity (nudges fired, PR check results, errors), ‘t’ to switch backto the table, ‘q’ to quit cleanly. Uses crossterm raw mode with 100mspoll intervals for responsive input.Also allows spawned workers to transition to stalled (previouslySpawned status was excluded from silence detection). surface PR health in ps –watch displayAdd a HEALTH column to the watch table showing per-worker PR checkresults (ci, conflicts, reviews, commits) so problems are visible at aglance without needing RUST_LOG=debug. Upgrade silent debug-level PRerrors to info-level logging. add –base flag to spawn and create for custom branch baseAllow overriding the default base branch (from jig.toml) per-commandwith –base/-b. Includes shell completions for branch names acrossbash, zsh, and fish. Also fixes spawn status message to show theactual base branch used instead of the current branch. wire issues into spawn pipeline with –issue flagAdd jig spawn --issue <id> to resolve file-based issues and use theirbody as Claude context. Thread issue_ref through the full pipeline:spawn CLI → register() → Spawn event → WorkerState reducer → daemonworkers.json → ps watch table.Also adds: add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.Improve issues command UX: accept trailing args in git hook subcommandsGit passes arguments to hooks (e.g. post-merge receives a squash flag“0” or “1”), which the hook wrapper forwards via “$@”. The CLIsubcommands rejected these unexpected args. Add trailing_var_arg toPostCommit, PostMerge, and PreCommit to accept and ignore them. output cd command from jig home instead of bare pathMatches the pattern used by jig open and jig exit — outputscd '/path' to stdout for shell eval, not just the path. recover from stale git worktree registrations on spawn and pruneWhen a worktree directory is removed but git still tracks the entry,git worktree add fails with “missing but already registered”. Nowcreate_worktree runs git worktree prune first, and prune_actorhandles the missing-directory case instead of skipping cleanup.Also extracts prune_actor into its own module and adds urgent issueto replace git CLI shelling with git2. add issues command to shell completionsThe issues command was missing from all three shells’ command listsand had no flag/argument completions. Adds command entry, issue IDpositional completions, and flag completions (status, priority,category, interactive, ids) for bash, zsh, and fish. use if-let instead of unwrap to satisfy clippy daemon PR discovery, tmux targeting, and nudge delivery register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.3.0 (2026-03-16)

<csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/>
<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/>
<csr-id-df444a5f287b413186005cd71b4071d6244fa31a/>
<csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/>
<csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/>
<csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/>
<csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-70d18a393801feb6fd45eb146928e9a96c37f072/> bump version to 1.3.0
 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

<csr-id-fca094fb2b823ad1288e803b5f814af76a69ec1c/>
<csr-id-8b54af9a711cd4286e483257622f4fc4ab056cce/>
<csr-id-10a900d4990351479eedf289bea2a44669f4e8e1/>
<csr-id-ffcc788e769bab29484f284e1dd659b9ba4f04b1/>
<csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/>
<csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/>
<csr-id-df0a3be811b27f8afce047bd088cad410d09e081/>
<csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/>
<csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/>
<csr-id-bd1a1faeca5a7634224bef836154791819b4903b/>
<csr-id-462f05eaf29929899631125c733738cd8f93e558/>
<csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/>
<csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/>
<csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/>
<csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/>
<csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/>
<csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/>
<csr-id-057e8dc3675610e75e826910d051774f32f63cee/>
<csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/>
<csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/>
<csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/>
<csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/>
<csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/>
<csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/>
<csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/>
<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-7ab467efe4eaf7b9652fb04777889f4822c0d033/> add commit-msg hook for conventional commit validation
   Adds a commit-msg git hook that validates commit messages against the
   conventional commits spec using the existing parser and jig.toml config.
   Closes the conventional-commits-validation issue.
 - <csr-id-21d793777b6eb0c764378725c4e9c59f6a6d8174/> add conventional commit validation and examples
   Add parser, configurable validator, and CLI commands:
   - `jig commit validate` validates HEAD, specific revs, stdin, or files

### Bug Fixes

<csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/>
<csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/>
<csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/>
<csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/>
<csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/>
<csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/>
<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-8dfb46ceb8945a03a40993187395a2bdd22dc4d6/> suppress deprecated cargo_bin warnings in test files
   CI uses -D warnings which promotes the assert_cmd::Command::cargo_bin
   deprecation warning to an error. Added #![allow(deprecated)] to match
   the existing pattern in attach_tests.rs.
 - <csr-id-54b775600e10ae3570e3934bd8c780715dafb85e/> suppress deprecated cargo_bin warning in integration tests
   CI uses -D warnings, causing the deprecated assert_cmd::Command::cargo_bin
   call to fail clippy. Add #[allow(deprecated)] to the test helper.
 - <csr-id-3855393d5fafedc5b77b37032362f80348140913/> suppress deprecated cargo_bin warning in attach tests
 - <csr-id-bc72d377a49040d30c4c9696959d0fa1799129c5/> auto-detect global mode in attach when outside git repo
   When running `jig attach <name>` outside a git repo, automatically
   fall back to global discovery across all tracked repos instead of
   requiring the `-g` flag. Mirrors the fix already applied to `open`.
 - <csr-id-e72a39866d32263188b01f858e0a3f941c6ba40d/> auto-detect global mode for completions and open outside git repo
   Shell completion scripts now use git rev-parse to detect whether cwd is
   inside a git repo. Inside a repo, completions use local worktrees;
   outside, they fall back to global worktrees automatically. The open
   command also auto-detects global mode when outside a repo, making -g
   redundant for jig open.
 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter

### Other

 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-d3bb7cc01727fcbd285fca0516db5820bdcf005b/> remove install-claude, auto-detect agent in hooks init
   `jig hooks init` now reads `[agent]` from jig.toml and installs
   agent-specific hooks automatically, removing the need for a separate
   `install-claude` subcommand.
 - <csr-id-df444a5f287b413186005cd71b4071d6244fa31a/> remove run_global from attach, add integration tests
   Remove the run_global method since auto-detection in run() makes -g
   redundant for attach. Add integration tests verifying behavior outside
   a git repo: name required, nonexistent worktree error, and -g rejection.
 - <csr-id-07a2e33ab6de0f1cc1adfe3022ed51d2de545d70/> remove run_global from open, inline global discovery
   Open no longer supports -g/--global mode. Instead, run() auto-detects
   when outside a git repo and performs global registry lookup inline,
   eliminating the need for the separate run_global path.
 - <csr-id-8a6b7487602794a2a3aa3f9ec750d928e4e5a64f/> wire up GlobalDaemonConfig, remove dead code, clean up APIs
   - Remove unused Deref impl from DaemonLifecycleLog
   - Wire GlobalDaemonConfig.interval_seconds and session_prefix into
     daemon command (CLI args override config, config overrides defaults)
   - Accept &HealthConfig in RecoveryScanner::new() instead of redundantly
     loading GlobalConfig
 - <csr-id-22cab4642dba2f198ab47b3a6c47590c5d3ea330/> address PR review — struct-based APIs, remove hardcoded deps
   - lifecycle.rs: Replace free functions with DaemonLifecycleLog struct
     (Deref to Path, methods for record_started/record_stopped/last_event)
   - recovery.rs: Replace free functions with RecoveryScanner struct
     (holds registry + health config, methods for find/recover/resume)
   - resume.rs: Remove hardcoded tmux/claude dependency checks — Worktree
     handles this internally via adapter pattern in launch()
   - config.rs: Expand GlobalDaemonConfig with interval_seconds and
     session_prefix fields alongside auto_recover
 - <csr-id-3123b7b74d9d0a1f6a2927ba97f7fc1eda85b8bb/> address review — remove unused flag, deduplicate helpers
   - Remove unused --auto flag from jig resume command
   - Deduplicate daemon_log_path: lifecycle.rs now uses global::paths
   - Deduplicate read_spawn_context: make recovery.rs version public,
     CLI resume command calls into it
   - Remove redundant is_terminal() guard in dead tmux detection
 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELLFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changes<csr-unknown>
<csr-unknown>
jig commit examples shows conventional commit referenceConfigurable via [commits] section in jig.toml16 unit tests + 12 integration testsjig issues create creates issues from templates with title, priority, category, labelsjig issues status <id> --status <new> updates frontmatter status in file issuesjig issues complete <id> marks issues as complete, with optional –deletejig issues stats shows breakdown by status and priorityNudge cooldown countdown in NUDGE column (e.g. “2/3 (3m12s)”)Nudge messages rendered below worker table when deliveredGlobal sync/poll timer footer alongside keybinding hintsPreserve draft PR status on GitHub API errors so nudges aren’tsilently suppressed for known-draft PRsThrottle GitHub API requests to once per 60s per worker, alignedwith gh –cache 60s TTL, reducing API pressure ~30xInstall SIGTERM/SIGINT handler for graceful daemon shutdownRecord Started/Stopped lifecycle events in daemon.jsonlDetect unclean shutdown on next startup (missing Stopped event)Auto-recover orphaned workers on daemon startup via Worktree::resume()Add jig resume <name> CLI command for manual worker recoveryDetect dead tmux windows during steady-state ticks and auto-resumeAdd auto_recover global config option (default: true) for opt-outWire Action::Restart to use recovery::try_resume_worker()Add integration tests for jig resume commandAdd RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structsAdd ResolvedNudgeConfig with resolver for per-type configThread resolved config through nudge classify, dispatch, and executeApply per-type cooldown to both idle/stalled nudges and PR nudgesDisplay effective nudge config in jig config showFixes PR nudge burst bug by enforcing per-type cooldownsSpawning worker names shown below the ps table during setupWorkerStatus::Initializing variant for future usespawn_labels config in jig.tomlThree new issues (config-show-auto-spawn, worker-initializing-state,auto-column-checkmark)Status symbol constants (✓, →, ✗, !)Formatted output helpers (success, progress, failure, warning, detail, header)Color helpers (highlight, bold, dim) that respect plain modeTable builder helper (new_table) for consistent table creationGlobal –plain flag for scriptable output (no colors, no decorations)Error display with cause chain formattingjig ls now shows a table with name, branch, and commits aheadjig ls -g shows tables grouped by repo with bold headersAdd --plain/-p flag for bare name output (old behavior)Shell completions use --plain and fall back to -gp outside a repoBranch column only shown when it differs from worktree namejig issues CLI command with –ids flag for scriptingIssuesConfig in jig.toml for configurable issues directoryISSUE column in ps –watch table (shortened last path segment)Shell completions for –issue in bash, zsh, and fishissue_ref tests in reducer and daemon roundtripTMUX column (●/○/✗) for session livenessSTATE column from event-derived WorkerStatusNUDGES count and PR number from event logConfigurable interval: jig ps -w 5 for 5s refreshDiscovers workers by scanning event log directoriesReplays events to derive current WorkerState per workerCompares old vs new state to dispatch actionsExecutes nudges via tmux and notifications via hooksPersists state to workers.json between ticksHook wrapper templates that chain jig logic with user hooksRegistry tracking installed hooks at jig-hooks.jsonIdempotent init with backup/restore of existing hooksPost-commit/merge handlers that emit events to worker logsUninstall with rollback to original user hooksEvent schema with typed EventType enum and flat JSONL serializationEventLog append-only reader/writer with per-worker JSONL filesClaude Code hook templates (PostToolUse, Notification, Stop)jig hooks install-claude CLI command to install hooks to ~/.claude/hooks/Detect installation method (script, cargo, source, unknown)Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)Relation mapping was inverted: “blocks” → “is_blocked_by” sodependency checking now correctly identifies blocked issuesRemove unused SearchData typeHide completed issues by default (use –all to include them)Interactive mode (-i) uses alternate screen buffer like less/git diffAdd ui::with_alternate_screen() reusable helperInteractive mode: scrolling, auto indicator, title truncation, G/g navAdd proactive PR discovery: daemon queries GitHub for open PRs onworker branches when pr_url is unknown, emits PrOpened events tomake state durable across restartsCreate per-repo GitHub clients via registry path lookup instead ofambient remote detection (fixes multi-repo daemon)Extract real branch name from spawn events for tmux window lookup(spawn creates windows with slashes, e.g. feature/foo, not dashes)Run all four PR checks (CI, conflicts, reviews, commits) on open PRsNudge on every tick, not just state transitions, so polling daemonretries delivery until max_nudgesCollapse multiline nudge templates to single line before tmux sendto prevent premature submission in TUIsFix tracing init: RUST_LOG now properly overrides default warn levelAdd stderr tick summary in continuous daemon mode for visibilitywithout RUST_LOGAdd debug logging for tmux window misses and notification pipelineState file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 add issue lifecycle commands (create, status, complete, stats)Add CLI subcommands to jig issues for managing issue state:For Linear issues, status changes go through the API (not file editing).Backwards-compatible: jig issues with no subcommand still lists/browses.Also adds “in-progress” (hyphenated) to loose status parsing. add ps -w observability (cooldowns, nudge messages, timers) and fix GitHub rate limitingAdd three display features to make ps -w a proper dashboard:Fix two bugs discovered during implementation: add daemon crash recovery and worker resume show nudge state in jig ps outputAdd NUDGE column between STATE and COMMITS in the ps table.Displays nudge count as count/max (e.g. 2/3), with color coding:grey dash for zero, yellow for in-progress, red for exhausted.Adds max_nudges field to WorkerDisplayInfo so the UI can renderthe denominator from the resolved health config. add per-repo nudge configuration in jig.tomlAdd [health] section to jig.toml supporting per-repo overrides ofsilence_threshold_seconds and max_nudges, plus per-nudge-type[health.nudge.<type>] sections with independent max andcooldown_seconds settings.Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]global config > defaults. When cooldown_seconds is not set, fallsback to silence_threshold_seconds. add jig home command to print base repo rootAdds jig home (alias jig h) that prints the base repository rootpath, enabling cd $(jig home) navigation from worktrees. communicate worker initialization state and on-create failuresAdd Initializing event type and worker status to make the workerlifecycle visible during setup. When the daemon auto-spawns a worker,it now registers the worker as Initializing before running theon-create hook, then transitions to Spawned on success or Failed onhook failure. show auto-spawn config in jig config and add jig issues --autoDisplay auto-spawn settings (enabled, auto-start, max workers, pollinterval, spawn labels) in jig config show with source attribution.Add --auto flag to jig issues to filter to only daemon-eligibleauto-spawn candidates using the existing list_spawnable method. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. move auto-spawn to background thread to keep ps -w responsiveThe on-create hook (e.g. pnpm install) was running synchronously onthe tick thread, freezing the ps –watch UI for the entire duration.Introduces a spawn_actor following the same pattern as prune_actor,issue_actor, etc. The tick now sends spawnable issues to the backgroundthread and drains results on the next tick.Also adds: add labels field for issue tagging and filteringAdd labels: Vec<String> to Issue and IssueFilter types. Linearprovider now passes all label names through from GraphQL (auto fieldderivation unchanged). File provider parses **Labels:** comma-separatedfrontmatter. CLI gains --label/-l flag for filtering (all must match).Shell completions updated for bash, zsh, and fish. add shared UI module with consistent formatting and –plain flagExpand ui.rs into a centralized formatting module with:Migrate all 20 command files from inline colored::Colorize calls toshared ui:: helpers. Add –plain support to list, repos, and issuescommands with tab-separated output for piping. add AUTO column to jig issues table outputShow a green dot indicator for issues tagged for auto-spawn, making itvisible at a glance whether file-provider Auto flag or Linear jig-autolabel is set. support -g/–global flag for attaching from anywhereAdd run_global implementation to the Attach command so users can attachto a worktree from outside the owning repo using jig attach <name> -g.Resolves the owning repo via GlobalCtx::repo_for_worktree. jig init –audit launches agent in tmux to populate docs–audit now spawns the configured agent in a jig-init:<repo> tmuxsession with the audit prompt instead of just printing instructions.–backup enhances the prompt to reference .backup/ files. –auditaccepts an optional string for extra instructions. block auto-spawn on unresolved dependenciesAdd is_spawnable_with_deps() to IssueProvider trait that checks alldepends_on entries resolve to Complete before allowing spawn. Applied inboth FileProvider and LinearProvider’s list_spawnable(). Also adds–blocked/–unblocked flags to jig issues CLI for filtering. group workers by repo in jig ps -g outputAdd repo field to WorkerDisplayInfo and render grouped tables withbold repo headers when running in global mode (jig ps -g / jig ps -gw).Local jig ps output is unchanged. daemon periodically prunes stale worktreesWorkers in terminal state (merged/archived/failed) with dead tmuxsessions now get their git worktrees, event logs, and global stateentries cleaned up automatically. Prune runs every 120s during watchmode. Pruned workers are reported in the tick status and log view.Also includes snake_case fixes for auto-spawn-filtering ticket. default table view for jig ls and pretty grouped jig ls -g show draft vs review state, document PR nudge behaviorWorkers with draft PRs now show “draft” (blue) in the STATE columninstead of “review” (cyan). This makes it visually clear which workerswill receive PR nudges (draft) vs which are in human review (non-draft).Add PR Nudges section to daemon docs explaining the draft/non-draftnudge policy and what each health check means. unify daemon/ps tick loops and add log toggle to watch modeExtract run_with() callback API from daemon so ps –watch shares thesame setup code path instead of duplicating Daemon/Notifier/TmuxClientconstruction. The callback controls inter-tick delay and can signalstop, which enables keypress handling during the sleep window.Add log view toggle to watch mode: press ‘l’ to see timestamped daemonactivity (nudges fired, PR check results, errors), ‘t’ to switch backto the table, ‘q’ to quit cleanly. Uses crossterm raw mode with 100mspoll intervals for responsive input.Also allows spawned workers to transition to stalled (previouslySpawned status was excluded from silence detection). surface PR health in ps –watch displayAdd a HEALTH column to the watch table showing per-worker PR checkresults (ci, conflicts, reviews, commits) so problems are visible at aglance without needing RUST_LOG=debug. Upgrade silent debug-level PRerrors to info-level logging. add –base flag to spawn and create for custom branch baseAllow overriding the default base branch (from jig.toml) per-commandwith –base/-b. Includes shell completions for branch names acrossbash, zsh, and fish. Also fixes spawn status message to show theactual base branch used instead of the current branch. wire issues into spawn pipeline with –issue flagAdd jig spawn --issue <id> to resolve file-based issues and use theirbody as Claude context. Thread issue_ref through the full pipeline:spawn CLI → register() → Spawn event → WorkerState reducer → daemonworkers.json → ps watch table.Also adds: add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.Improve issues command UX: accept trailing args in git hook subcommandsGit passes arguments to hooks (e.g. post-merge receives a squash flag“0” or “1”), which the hook wrapper forwards via “$@”. The CLIsubcommands rejected these unexpected args. Add trailing_var_arg toPostCommit, PostMerge, and PreCommit to accept and ignore them. output cd command from jig home instead of bare pathMatches the pattern used by jig open and jig exit — outputscd '/path' to stdout for shell eval, not just the path. recover from stale git worktree registrations on spawn and pruneWhen a worktree directory is removed but git still tracks the entry,git worktree add fails with “missing but already registered”. Nowcreate_worktree runs git worktree prune first, and prune_actorhandles the missing-directory case instead of skipping cleanup.Also extracts prune_actor into its own module and adds urgent issueto replace git CLI shelling with git2. add issues command to shell completionsThe issues command was missing from all three shells’ command listsand had no flag/argument completions. Adds command entry, issue IDpositional completions, and flag completions (status, priority,category, interactive, ids) for bash, zsh, and fish. use if-let instead of unwrap to satisfy clippy daemon PR discovery, tmux targeting, and nudge delivery register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.2.0 (2026-03-11)

<csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/>
<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/>
<csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/>
<csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/>
<csr-id-85d9e1de3500d926401b726017ee07199e5ff863/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>
<csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/>

### Chore

 - <csr-id-bd3db9f58cf9e9100bbfe7a1ee3480cfc3c4e566/> bump version to 1.2.0
 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
 - <csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/> bump version to 0.5.0
 - <csr-id-d38e493e16a264b81885608389452aa889ddfc6b/> remove jig-tui crate and wt references
   - Remove jig-tui crate entirely (was just a stub)
   - Remove Tui command from CLI
   - Rename all wt references to jig throughout codebase
   - Remove outdated wiki docs and spawn guides
   - Remove deprecated .claude/commands (replaced by skills)
   - Update tests to use jig binary name and init claude arg
   - Remove wt.toml (replaced by jig.toml)

### Documentation

 - <csr-id-3520041197353776dd5999f805866d7c18da9298/> audit and update docs/wiki for release, remove PROJECT_LAYOUT.md and GRINDER-ANALYSIS.md
   Update daemon.md with actor architecture, tmux timeouts, and per-repo config.
   Add actor pattern to PATTERNS.md. Fix SUCCESS_CRITERIA.md pre-commit claim.
   Update wiki with correct commands, new worker states, and actor details.
   Remove PROJECT_LAYOUT.md (derivable from codebase) and GRINDER-ANALYSIS.md
   (historical, all functionality now integrated) from docs, templates, init,
   skills, and all references.
 - <csr-id-b0f93bcf7cc499835d82f0944a93ccb4a4d3e3b9/> document overlapping branch name behavior in run_global
   Add doc comment explaining that when multiple repos have a worktree with
   the same name, the first match (in repo discovery order) is used,
   consistent with other global commands.

### New Features

<csr-id-37a59d2d8c02dbc87bbff0fcf4f92aef768bd996/>
<csr-id-df0a3be811b27f8afce047bd088cad410d09e081/>
<csr-id-45fe8b1e0bf4d16c7d8fc267c150d8dfb506f914/>
<csr-id-52208bf4ef7efc35cac4726bd4fa73e2713b7bb5/>
<csr-id-bd1a1faeca5a7634224bef836154791819b4903b/>
<csr-id-462f05eaf29929899631125c733738cd8f93e558/>
<csr-id-d5f79bd94e27cf82bc4e5b70f977eea258b62a92/>
<csr-id-2e4d781ae7aeda559884ea980cacdd5fae423d0c/>
<csr-id-1f4553fd7f0a7e21cfb5234e3800a8152f6dcca1/>
<csr-id-3ab73b1d1c7e20c25898ed021a50d7aebf2d0dd1/>
<csr-id-5745c0d00da47a05a7a4b98d1bca6d9985afc25b/>
<csr-id-5217bfa9423f54a27b9e0badef98c4a72e2e273e/>
<csr-id-057e8dc3675610e75e826910d051774f32f63cee/>
<csr-id-feb9d6068256ec7e2298a08e798a5913396a615d/>
<csr-id-23c5b4f9f732bb70616b95e95c5b1d7c946e43d1/>
<csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/>
<csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/>
<csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/>
<csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/>
<csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/>
<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-4be2cafcf2b1c7bcb9a42192a636aaf84d6fbcfc/> add per-repo nudge configuration in jig.toml
   Add [health] section to jig.toml supporting per-repo overrides of
   silence_threshold_seconds and max_nudges, plus per-nudge-type
   [health.nudge.<type>] sections with independent max and
   cooldown_seconds settings.
   
   Resolution order: jig.toml [health.nudge.<type>] > jig.toml [health]
   > global config > defaults. When cooldown_seconds is not set, falls
   back to silence_threshold_seconds.
   
   - Add RepoHealthConfig, NudgeTypeConfigs, NudgeTypeConfig structs

### Bug Fixes

<csr-id-57e94d35e5e961a5fc68624b2646720f315327a2/>
<csr-id-46949f98b3f25a53067d7845b8e85e299e7e1909/>
<csr-id-c970409ad61f8b48cbeb51dfe99371a225f9a4f7/>
<csr-id-1a36eb384a4ca2b5aab12a518e98daa472022859/>
<csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/>
<csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/>
<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-cd7bb28acc3d02336db120485a99f31297e75e80/> remove useless io::Error conversion in with_alternate_screen
 - <csr-id-8fbdbae9b48196658aa61eca9c60e4492adbc7f5/> Linear issue discovery and issues UX improvements
   Fix three bugs in Linear provider:
   - get_issue GraphQL query had mismatched braces and used deprecated
   issueSearch; rewrite to use issues query with team+number filter

### Other

 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-e33f0420bcba0c6abd5758cfafd756fff91515ad/> remove auto field, normalize on label-based spawn filtering
   Remove the `auto: bool` field from the `Issue` struct and the `**Auto:**`
   frontmatter / `jig-auto` label special-casing. Auto-spawn eligibility is
   now determined purely by `spawn_labels` in `[issues]` config in jig.toml.
   
   - Remove `auto` field from Issue struct
   - Remove `**Auto:** true` parsing from file provider
   - Remove `jig-auto` label detection from Linear client
   - Remove `i.auto` filter from list_spawnable in both providers
   - Remove `**Auto:**` from all existing issue files
   - Add `**Labels:**` field to issue templates
   - Update issues README with Labels field in format example
   - Update wiki skill-examples to reference spawn_labels config
   - Update auto-spawn-filtering issue to reflect new state
 - <csr-id-7bdb392c7ffa5727e951f18377022e7d596c4151/> consolidate worktree management into Worktree struct
   Make Worktree the single abstraction for a worker's physical state —
   repo, branch, path, tmux session, spawn context, and lifecycle.
   
   - Expand Worktree struct with repo_root, session_name, auto_spawned fields
   - Add lifecycle methods: launch(), resume(), register(), unregister()
   - Add tmux methods: has_tmux_window(), is_agent_running()
   - Add orphan detection: is_orphaned()
   - Add Resume event type to EventType enum and handle in reducer/derive
   - Fix derive_worker_name() to preserve category prefixes (features/foo)
   - Fix Repo::remove_worktree() to accept optional repo_root, avoiding
     Repo::discover() in daemon paths
   - Update daemon auto_spawn_worker to use Worktree::create + wt.register
     + wt.launch
   - Update CLI create/remove/spawn commands to use Worktree methods
   - Remove all spawn::register/spawn::launch_tmux_window calls outside
     Worktree
   - Eliminate Repo::discover() from all daemon code paths
 - <csr-id-1345768accb7711e0a333c0c7a3da55dfb3afd1d/> wrap git2 in Repo struct, remove Git(String) error variant
   Address PR feedback:
   - Wrap git2::Repository in a `Repo` struct with domain methods instead
     of free functions passing repo handles around
   - Remove Error::Git(String) variant — use Error::Git2(#[from] git2::Error)
     directly instead of mapping git2 errors to strings
   - Add Error::MergeConflict for merge-specific errors
   - DRY up duplicated prune_stale_worktrees and find_worktree_by_path
     code — prune_actor.rs now calls Repo methods from git.rs
   - Update all call sites across CLI commands, daemon, worktree, spawn,
     and context modules
   - Update PATTERNS.md and PROJECT_LAYOUT.md to reflect git2 usage
 - <csr-id-85d9e1de3500d926401b726017ee07199e5ff863/> move spawn daemon settings to global config with per-repo overrides
   Per-developer settings (auto_spawn, max_concurrent_workers,
   auto_spawn_interval) now live in ~/.config/jig/config.toml instead of
   jig.toml, since they shouldn't be committed to the repo. Per-repo
   jig.toml can still override via optional fields.
 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### Style

 - <csr-id-f7e016757eb9b899cd43b37b42b01164c8bd0fc7/> fix rustfmt formatting in init command

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELLFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changes<csr-unknown>
<csr-unknown>
Add ResolvedNudgeConfig with resolver for per-type configThread resolved config through nudge classify, dispatch, and executeApply per-type cooldown to both idle/stalled nudges and PR nudgesDisplay effective nudge config in jig config showFixes PR nudge burst bug by enforcing per-type cooldownsSpawning worker names shown below the ps table during setupWorkerStatus::Initializing variant for future usespawn_labels config in jig.tomlThree new issues (config-show-auto-spawn, worker-initializing-state,auto-column-checkmark)Status symbol constants (✓, →, ✗, !)Formatted output helpers (success, progress, failure, warning, detail, header)Color helpers (highlight, bold, dim) that respect plain modeTable builder helper (new_table) for consistent table creationGlobal –plain flag for scriptable output (no colors, no decorations)Error display with cause chain formattingjig ls now shows a table with name, branch, and commits aheadjig ls -g shows tables grouped by repo with bold headersAdd --plain/-p flag for bare name output (old behavior)Shell completions use --plain and fall back to -gp outside a repoBranch column only shown when it differs from worktree namejig issues CLI command with –ids flag for scriptingIssuesConfig in jig.toml for configurable issues directoryISSUE column in ps –watch table (shortened last path segment)Shell completions for –issue in bash, zsh, and fishissue_ref tests in reducer and daemon roundtripTMUX column (●/○/✗) for session livenessSTATE column from event-derived WorkerStatusNUDGES count and PR number from event logConfigurable interval: jig ps -w 5 for 5s refreshDiscovers workers by scanning event log directoriesReplays events to derive current WorkerState per workerCompares old vs new state to dispatch actionsExecutes nudges via tmux and notifications via hooksPersists state to workers.json between ticksHook wrapper templates that chain jig logic with user hooksRegistry tracking installed hooks at jig-hooks.jsonIdempotent init with backup/restore of existing hooksPost-commit/merge handlers that emit events to worker logsUninstall with rollback to original user hooksEvent schema with typed EventType enum and flat JSONL serializationEventLog append-only reader/writer with per-worker JSONL filesClaude Code hook templates (PostToolUse, Notification, Stop)jig hooks install-claude CLI command to install hooks to ~/.claude/hooks/Detect installation method (script, cargo, source, unknown)Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)Relation mapping was inverted: “blocks” → “is_blocked_by” sodependency checking now correctly identifies blocked issuesRemove unused SearchData typeHide completed issues by default (use –all to include them)Interactive mode (-i) uses alternate screen buffer like less/git diffAdd ui::with_alternate_screen() reusable helperInteractive mode: scrolling, auto indicator, title truncation, G/g navAdd proactive PR discovery: daemon queries GitHub for open PRs onworker branches when pr_url is unknown, emits PrOpened events tomake state durable across restartsCreate per-repo GitHub clients via registry path lookup instead ofambient remote detection (fixes multi-repo daemon)Extract real branch name from spawn events for tmux window lookup(spawn creates windows with slashes, e.g. feature/foo, not dashes)Run all four PR checks (CI, conflicts, reviews, commits) on open PRsNudge on every tick, not just state transitions, so polling daemonretries delivery until max_nudgesCollapse multiline nudge templates to single line before tmux sendto prevent premature submission in TUIsFix tracing init: RUST_LOG now properly overrides default warn levelAdd stderr tick summary in continuous daemon mode for visibilitywithout RUST_LOGAdd debug logging for tmux window misses and notification pipelineState file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 add jig home command to print base repo rootAdds jig home (alias jig h) that prints the base repository rootpath, enabling cd $(jig home) navigation from worktrees. communicate worker initialization state and on-create failuresAdd Initializing event type and worker status to make the workerlifecycle visible during setup. When the daemon auto-spawns a worker,it now registers the worker as Initializing before running theon-create hook, then transitions to Spawned on success or Failed onhook failure. show auto-spawn config in jig config and add jig issues --autoDisplay auto-spawn settings (enabled, auto-start, max workers, pollinterval, spawn labels) in jig config show with source attribution.Add --auto flag to jig issues to filter to only daemon-eligibleauto-spawn candidates using the existing list_spawnable method. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. use checkmark instead of asterisk for AUTO columnChange the AUTO column indicator in jig issues from * to ✓for better readability. move auto-spawn to background thread to keep ps -w responsiveThe on-create hook (e.g. pnpm install) was running synchronously onthe tick thread, freezing the ps –watch UI for the entire duration.Introduces a spawn_actor following the same pattern as prune_actor,issue_actor, etc. The tick now sends spawnable issues to the backgroundthread and drains results on the next tick.Also adds: add labels field for issue tagging and filteringAdd labels: Vec<String> to Issue and IssueFilter types. Linearprovider now passes all label names through from GraphQL (auto fieldderivation unchanged). File provider parses **Labels:** comma-separatedfrontmatter. CLI gains --label/-l flag for filtering (all must match).Shell completions updated for bash, zsh, and fish. add shared UI module with consistent formatting and –plain flagExpand ui.rs into a centralized formatting module with:Migrate all 20 command files from inline colored::Colorize calls toshared ui:: helpers. Add –plain support to list, repos, and issuescommands with tab-separated output for piping. add AUTO column to jig issues table outputShow a green dot indicator for issues tagged for auto-spawn, making itvisible at a glance whether file-provider Auto flag or Linear jig-autolabel is set. support -g/–global flag for attaching from anywhereAdd run_global implementation to the Attach command so users can attachto a worktree from outside the owning repo using jig attach <name> -g.Resolves the owning repo via GlobalCtx::repo_for_worktree. jig init –audit launches agent in tmux to populate docs–audit now spawns the configured agent in a jig-init:<repo> tmuxsession with the audit prompt instead of just printing instructions.–backup enhances the prompt to reference .backup/ files. –auditaccepts an optional string for extra instructions. block auto-spawn on unresolved dependenciesAdd is_spawnable_with_deps() to IssueProvider trait that checks alldepends_on entries resolve to Complete before allowing spawn. Applied inboth FileProvider and LinearProvider’s list_spawnable(). Also adds–blocked/–unblocked flags to jig issues CLI for filtering. group workers by repo in jig ps -g outputAdd repo field to WorkerDisplayInfo and render grouped tables withbold repo headers when running in global mode (jig ps -g / jig ps -gw).Local jig ps output is unchanged. daemon periodically prunes stale worktreesWorkers in terminal state (merged/archived/failed) with dead tmuxsessions now get their git worktrees, event logs, and global stateentries cleaned up automatically. Prune runs every 120s during watchmode. Pruned workers are reported in the tick status and log view.Also includes snake_case fixes for auto-spawn-filtering ticket. default table view for jig ls and pretty grouped jig ls -g show draft vs review state, document PR nudge behaviorWorkers with draft PRs now show “draft” (blue) in the STATE columninstead of “review” (cyan). This makes it visually clear which workerswill receive PR nudges (draft) vs which are in human review (non-draft).Add PR Nudges section to daemon docs explaining the draft/non-draftnudge policy and what each health check means. unify daemon/ps tick loops and add log toggle to watch modeExtract run_with() callback API from daemon so ps –watch shares thesame setup code path instead of duplicating Daemon/Notifier/TmuxClientconstruction. The callback controls inter-tick delay and can signalstop, which enables keypress handling during the sleep window.Add log view toggle to watch mode: press ‘l’ to see timestamped daemonactivity (nudges fired, PR check results, errors), ‘t’ to switch backto the table, ‘q’ to quit cleanly. Uses crossterm raw mode with 100mspoll intervals for responsive input.Also allows spawned workers to transition to stalled (previouslySpawned status was excluded from silence detection). surface PR health in ps –watch displayAdd a HEALTH column to the watch table showing per-worker PR checkresults (ci, conflicts, reviews, commits) so problems are visible at aglance without needing RUST_LOG=debug. Upgrade silent debug-level PRerrors to info-level logging. add –base flag to spawn and create for custom branch baseAllow overriding the default base branch (from jig.toml) per-commandwith –base/-b. Includes shell completions for branch names acrossbash, zsh, and fish. Also fixes spawn status message to show theactual base branch used instead of the current branch. wire issues into spawn pipeline with –issue flagAdd jig spawn --issue <id> to resolve file-based issues and use theirbody as Claude context. Thread issue_ref through the full pipeline:spawn CLI → register() → Spawn event → WorkerState reducer → daemonworkers.json → ps watch table.Also adds: add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.Improve issues command UX: accept trailing args in git hook subcommandsGit passes arguments to hooks (e.g. post-merge receives a squash flag“0” or “1”), which the hook wrapper forwards via “$@”. The CLIsubcommands rejected these unexpected args. Add trailing_var_arg toPostCommit, PostMerge, and PreCommit to accept and ignore them. output cd command from jig home instead of bare pathMatches the pattern used by jig open and jig exit — outputscd '/path' to stdout for shell eval, not just the path. recover from stale git worktree registrations on spawn and pruneWhen a worktree directory is removed but git still tracks the entry,git worktree add fails with “missing but already registered”. Nowcreate_worktree runs git worktree prune first, and prune_actorhandles the missing-directory case instead of skipping cleanup.Also extracts prune_actor into its own module and adds urgent issueto replace git CLI shelling with git2. add issues command to shell completionsThe issues command was missing from all three shells’ command listsand had no flag/argument completions. Adds command entry, issue IDpositional completions, and flag completions (status, priority,category, interactive, ids) for bash, zsh, and fish. use if-let instead of unwrap to satisfy clippy daemon PR discovery, tmux targeting, and nudge delivery register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.1.1 (2026-03-04)

<csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/>
<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/>
<csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/>
<csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>

### Chore

 - <csr-id-3d809c6a1b58f3d438c3d279592005947ad50438/> bump version to 1.1.1
 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
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

<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/> show draft vs review state, document PR nudge behavior
   Workers with draft PRs now show "draft" (blue) in the STATE column
   instead of "review" (cyan). This makes it visually clear which workers
   will receive PR nudges (draft) vs which are in human review (non-draft).
   
   Add PR Nudges section to daemon docs explaining the draft/non-draft
   nudge policy and what each health check means.
 - <csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/> unify daemon/ps tick loops and add log toggle to watch mode
   Extract run_with() callback API from daemon so ps --watch shares the
   same setup code path instead of duplicating Daemon/Notifier/TmuxClient
   construction. The callback controls inter-tick delay and can signal
   stop, which enables keypress handling during the sleep window.
   
   Add log view toggle to watch mode: press 'l' to see timestamped daemon
   activity (nudges fired, PR check results, errors), 't' to switch back
   to the table, 'q' to quit cleanly. Uses crossterm raw mode with 100ms
   poll intervals for responsive input.
   
   Also allows spawned workers to transition to stalled (previously
   Spawned status was excluded from silence detection).
 - <csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/> surface PR health in ps --watch display
   Add a HEALTH column to the watch table showing per-worker PR check
   results (ci, conflicts, reviews, commits) so problems are visible at a
   glance without needing RUST_LOG=debug. Upgrade silent debug-level PR
   errors to info-level logging.
 - <csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/> add --base flag to spawn and create for custom branch base
   Allow overriding the default base branch (from jig.toml) per-command
   with --base/-b. Includes shell completions for branch names across
   bash, zsh, and fish. Also fixes spawn status message to show the
   actual base branch used instead of the current branch.
 - <csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/> wire issues into spawn pipeline with --issue flag
   Add `jig spawn --issue <id>` to resolve file-based issues and use their
   body as Claude context. Thread issue_ref through the full pipeline:
   spawn CLI → register() → Spawn event → WorkerState reducer → daemon
   workers.json → ps watch table.
   
   Also adds:
   - `jig issues` CLI command with --ids flag for scripting

### Bug Fixes

<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-d720fcaa0d1f1e0a327ae5d3c90dfe49323b198a/> use if-let instead of unwrap to satisfy clippy
 - <csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/> daemon PR discovery, tmux targeting, and nudge delivery
   - Add proactive PR discovery: daemon queries GitHub for open PRs on
   worker branches when pr_url is unknown, emits PrOpened events to
   make state durable across restarts

### Other

 - <csr-id-8abff4b7ca2031d3232127b93febb92eb07cd9c5/> fmt
 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-12f9c10b9f61aa2054a2d5c2d559553d3af50069/> remove `jig status` command (redundant with `jig ps`)
 - <csr-id-f694a0ce3f1a96ad9fc8b38d1c947924e6acaeaf/> drop -g support from attach/merge/review, deduplicate ps
   attach, merge, and review don't make sense in global mode — worktree
   names can conflict across repos. Extract shared ps logic into
   execute_ps() helper to eliminate duplication between run/run_global.
 - <csr-id-a0c69ed63f57649a00d0484505bafc9c644ca7e9/> split Op trait into run/run_global for -g flag dispatch
   Replace OpContext (single struct with global bool + repos vec) with two
   distinct context types: RepoCtx for single-repo operations and GlobalCtx
   for cross-repo -g mode. The Op trait now has run() and run_global()
   methods, with the default run_global() rejecting unsupported commands.
   
   11 global commands (list, ps, kill, remove, review, merge, attach,
   status, nuke, issues, open) implement both methods. 14 non-global
   commands only implement run(). The command_enum! macro dispatches both,
   and main.rs branches on cli.global to build the right context.
 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELLFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changes<csr-unknown>
<csr-unknown>
IssuesConfig in jig.toml for configurable issues directoryISSUE column in ps –watch table (shortened last path segment)Shell completions for –issue in bash, zsh, and fishissue_ref tests in reducer and daemon roundtripTMUX column (●/○/✗) for session livenessSTATE column from event-derived WorkerStatusNUDGES count and PR number from event logConfigurable interval: jig ps -w 5 for 5s refreshDiscovers workers by scanning event log directoriesReplays events to derive current WorkerState per workerCompares old vs new state to dispatch actionsExecutes nudges via tmux and notifications via hooksPersists state to workers.json between ticksHook wrapper templates that chain jig logic with user hooksRegistry tracking installed hooks at jig-hooks.jsonIdempotent init with backup/restore of existing hooksPost-commit/merge handlers that emit events to worker logsUninstall with rollback to original user hooksEvent schema with typed EventType enum and flat JSONL serializationEventLog append-only reader/writer with per-worker JSONL filesClaude Code hook templates (PostToolUse, Notification, Stop)jig hooks install-claude CLI command to install hooks to ~/.claude/hooks/Detect installation method (script, cargo, source, unknown)Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)Create per-repo GitHub clients via registry path lookup instead ofambient remote detection (fixes multi-repo daemon)Extract real branch name from spawn events for tmux window lookup(spawn creates windows with slashes, e.g. feature/foo, not dashes)Run all four PR checks (CI, conflicts, reviews, commits) on open PRsNudge on every tick, not just state transitions, so polling daemonretries delivery until max_nudgesCollapse multiline nudge templates to single line before tmux sendto prevent premature submission in TUIsFix tracing init: RUST_LOG now properly overrides default warn levelAdd stderr tick summary in continuous daemon mode for visibilitywithout RUST_LOGAdd debug logging for tmux window misses and notification pipelineState file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list. register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.1.0 (2026-03-03)

<csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/>
<csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/>
<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/>
<csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/>
<csr-id-80401de003d427eeb057c8f64805b91060278fe5/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>

### Chore

 - <csr-id-0d3f00fefd29350c51e4671b9de14d230b809931/> bump version to 1.1.0
 - <csr-id-639e712803a8d13d5f8c84728d0410a17b47561e/> bump all outdated crates to latest major versions
   - thiserror 1 → 2 (no API changes needed)
   - colored 2 → 3 (MSRV bump only, dropped lazy_static)
   - dirs 5 → 6 (API compatible)
   - toml 0.8 → 1.0 (API compatible)
   - handlebars 5 → 6 (RenderError refactored, no impact on our usage)
   - which 6 → 8 (API compatible)
   - nix 0.28 → 0.31 (no breaking changes for process feature)
   - flume 0.11 → 0.12 (API compatible)
 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
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

<csr-id-d790a8101173e5797d7f331b56e0a0f5b06566a4/>
<csr-id-1a8faafa772e7c9014347f6802936d7d9a817bcb/>
<csr-id-73dc3fbbf0178af964a9f0481a5e85fc0e66cde1/>
<csr-id-13e44044ea08a91eb24e4b1b38c43c695a2fadc4/>
<csr-id-1bb57f9c0543cd7af986dd2303f34395980019f4/>
<csr-id-82c654ab1137ec963121638f6741617c59ee0c04/>
<csr-id-d878b9792a36f7c0d1157296401ca80af7f86f30/>
<csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/>
<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-780632c2fff774e3f968ee8254f5b57a46abaa55/> show draft vs review state, document PR nudge behavior
   Workers with draft PRs now show "draft" (blue) in the STATE column
   instead of "review" (cyan). This makes it visually clear which workers
   will receive PR nudges (draft) vs which are in human review (non-draft).
   
   Add PR Nudges section to daemon docs explaining the draft/non-draft
   nudge policy and what each health check means.
 - <csr-id-61339c359884180d22d04a206be57d7b28d6fa9a/> unify daemon/ps tick loops and add log toggle to watch mode
   Extract run_with() callback API from daemon so ps --watch shares the
   same setup code path instead of duplicating Daemon/Notifier/TmuxClient
   construction. The callback controls inter-tick delay and can signal
   stop, which enables keypress handling during the sleep window.
   
   Add log view toggle to watch mode: press 'l' to see timestamped daemon
   activity (nudges fired, PR check results, errors), 't' to switch back
   to the table, 'q' to quit cleanly. Uses crossterm raw mode with 100ms
   poll intervals for responsive input.
   
   Also allows spawned workers to transition to stalled (previously
   Spawned status was excluded from silence detection).
 - <csr-id-c34254a3c119de72e0c472c5bf814059547fdbd6/> surface PR health in ps --watch display
   Add a HEALTH column to the watch table showing per-worker PR check
   results (ci, conflicts, reviews, commits) so problems are visible at a
   glance without needing RUST_LOG=debug. Upgrade silent debug-level PR
   errors to info-level logging.
 - <csr-id-8c92e5a1faa6992a14fb494640fb263d6cbc7049/> add --base flag to spawn and create for custom branch base
   Allow overriding the default base branch (from jig.toml) per-command
   with --base/-b. Includes shell completions for branch names across
   bash, zsh, and fish. Also fixes spawn status message to show the
   actual base branch used instead of the current branch.
 - <csr-id-e33ab3dfa06347d2aee13dc6d53d422cc462117c/> wire issues into spawn pipeline with --issue flag
   Add `jig spawn --issue <id>` to resolve file-based issues and use their
   body as Claude context. Thread issue_ref through the full pipeline:
   spawn CLI → register() → Spawn event → WorkerState reducer → daemon
   workers.json → ps watch table.
   
   Also adds:
   - `jig issues` CLI command with --ids flag for scripting

### Bug Fixes

<csr-id-378031a0afe019f57edc9bae469bf8168e05de29/>
<csr-id-61dd7ff112e0cb63885649b399e764578f99e4b2/>
<csr-id-a41b92cb77141469539658c133da79f79f714452/>
<csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/>
<csr-id-196774225c8eba52fdb9382f98418ecf82c48567/>

 - <csr-id-52c77af3da99153a3ff98e580f419a70f8500d93/> daemon PR discovery, tmux targeting, and nudge delivery
   - Add proactive PR discovery: daemon queries GitHub for open PRs on
   worker branches when pr_url is unknown, emits PrOpened events to
   make state durable across restarts

### Other

 - <csr-id-f7c5d5451126c55a29a5742b0ac55e5d2357dc36/> fmt

### Refactor

 - <csr-id-78cff84a46db59e266f2fa4affdaafb3c5857708/> unify CLI rendering with shared ui module and daemon-backed ps
   Extract duplicated table rendering, color mappings, and truncation into
   a shared crates/jig-cli/src/ui.rs module. Non-watch `jig ps` now uses a
   single daemon tick (once:true) to get the same rich WorkerDisplayInfo as
   watch mode — same columns (WORKER/STATE/COMMITS/PR/HEALTH/ISSUE) for
   both paths. Merge tmux status indicator into the WORKER name cell
   (colored dot prefix) instead of a separate cryptic column.
   
   Also includes: actor-based daemon runtime, issue/github/sync actors,
   Linear integration, session management, and various daemon improvements
   that were pending on this branch.
 - <csr-id-80401de003d427eeb057c8f64805b91060278fe5/> extract daemon.rs into struct-based daemon/ submodule
   Split the 675-line daemon.rs into a daemon/ directory with three files:
   - mod.rs: Daemon struct with tick/process_worker/sync_repos methods
   - discovery.rs: worker discovery and directory name splitting
   - pr.rs: PrMonitor struct for PR lifecycle checks
   
   This eliminates #[allow(clippy::too_many_arguments)] by moving shared
   state into the Daemon struct. All 7 tests preserved, public API updated
   from daemon::tick() to Daemon::new().tick().
 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELLFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changes<csr-unknown>
<csr-unknown>
IssuesConfig in jig.toml for configurable issues directoryISSUE column in ps –watch table (shortened last path segment)Shell completions for –issue in bash, zsh, and fishissue_ref tests in reducer and daemon roundtripTMUX column (●/○/✗) for session livenessSTATE column from event-derived WorkerStatusNUDGES count and PR number from event logConfigurable interval: jig ps -w 5 for 5s refreshDiscovers workers by scanning event log directoriesReplays events to derive current WorkerState per workerCompares old vs new state to dispatch actionsExecutes nudges via tmux and notifications via hooksPersists state to workers.json between ticksHook wrapper templates that chain jig logic with user hooksRegistry tracking installed hooks at jig-hooks.jsonIdempotent init with backup/restore of existing hooksPost-commit/merge handlers that emit events to worker logsUninstall with rollback to original user hooksEvent schema with typed EventType enum and flat JSONL serializationEventLog append-only reader/writer with per-worker JSONL filesClaude Code hook templates (PostToolUse, Notification, Stop)jig hooks install-claude CLI command to install hooks to ~/.claude/hooks/Detect installation method (script, cargo, source, unknown)Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)Create per-repo GitHub clients via registry path lookup instead ofambient remote detection (fixes multi-repo daemon)Extract real branch name from spawn events for tmux window lookup(spawn creates windows with slashes, e.g. feature/foo, not dashes)Run all four PR checks (CI, conflicts, reviews, commits) on open PRsNudge on every tick, not just state transitions, so polling daemonretries delivery until max_nudgesCollapse multiline nudge templates to single line before tmux sendto prevent premature submission in TUIsFix tracing init: RUST_LOG now properly overrides default warn levelAdd stderr tick summary in continuous daemon mode for visibilitywithout RUST_LOGAdd debug logging for tmux window misses and notification pipelineState file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 add watch mode to ps command for live dashboardjig ps --watch clears and refreshes the worker table every 2s.Shows enriched state from event logs alongside tmux status: add daemon loop to orchestrate event-driven pipelineThe missing conductor: jig daemon runs a periodic loop that:Supports –once for single-pass mode and –interval for tuning. add git hook management (install, uninstall, handlers)Implements the git-hooks epic (tickets 0-4): expand WorkerStatus with event-driven statesAdd Idle, WaitingInput, Stalled variants. Make all variants unit types(remove associated data from WaitingReview/Failed). Add needs_attention(),is_active(), is_terminal(), from_legacy() methods. Snake_case serialization. add event log format and Claude Code hooksImplement event-system tickets 1 and 2: add global state infrastructure for cross-repo aggregationIntroduces ~/.config/jig/ directory structure with structured TOML config,aggregated JSON worker state, and event log directories for the event-drivenpipeline. Ensures global dirs are created at CLI startup. introduce RepoContext and thread repo state through all operationsDerive repo_root, worktrees_dir, git_common_dir, base_branch, andsession_name once at startup via RepoContext::from_cwd(), eliminatingredundant git subprocess calls (e.g. spawn called get_base_repo() 8x).OpContext now holds Option<RepoContext>, and all jig-core functionsaccept &RepoContext instead of re-deriving from cwd. Also adds reporegistry for global mode auto-registration, removes dead spawn::kill(),and updates docs/patterns/issue status. implement smart jig update commandRewrite update command to: prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list. register Claude hooks in settings.json, add kill –all and nukeClaude Code hooks were installed as scripts but never registered in~/.claude/settings.json, so they never fired. Now jig init registersthem properly. Also fixes: hook templates read JSON from stdin (notenv vars), spawned workers no longer nudged as stalled, event logsreset on respawn, row ordering stabilized in ps –watch, kill/unregistercleans up event logs, and nuke command added for full repo cleanup. address review findings and wire up event pipeline end-to-endFix 6 issues from code review: UTF-8 safe truncate, stable statusserialization via as_str/from_legacy, stuck nudge sends message afterauto-approve, notification errors logged, branch names URL-encoded,tmux commands check exit status.Wire up missing pipeline links: jig spawn emits Spawn event, jig initauto-installs git+Claude hooks (idempotent on re-run), ps –watch runsdaemon tick on each refresh for integrated orchestration.Add docs/daemon.md with background service setup for launchd, systemd,OpenRC, and generic nohup. remove unnecessary return statement make –audit print command instead of trying to launch claudeSpawning claude programmatically was causing terminal issues and hangs.Now –audit just prints the command for the user to run manually. prevent shell-setup from corrupting shell config filesThe previous byte-slicing approach in find_path_line_end() calculatedoffsets incorrectly because lines() strips newlines but the code assumed+1 byte per line. This could corrupt or truncate config files.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v1.0.0 (2026-02-20)

<csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/>
<csr-id-72ff9fcf89d38f5e74d6d06c128226d2f094feb1/>
<csr-id-d38e493e16a264b81885608389452aa889ddfc6b/>
<csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/>

### Chore

 - <csr-id-f39d6b5fb56180c8cc9f40adf812138f8824b64d/> bump version to 1.0.0
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

<csr-id-357f9a6dfb6ab792078fc900f9b1bb956b3a4e4a/>
<csr-id-a685a48ac6c1b1d693e440d4e565e0bbd3ea49c0/>
<csr-id-823eeb1a83ac668fe54b7dbb28a0d062c4f91e9a/>
<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-5b776f40ef697de1ecb06c16e97feb4102b23103/> implement smart jig update command
   Rewrite update command to:
   - Detect installation method (script, cargo, source, unknown)

### Bug Fixes

 - <csr-id-a41b92cb77141469539658c133da79f79f714452/> remove unnecessary return statement
 - <csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/> make --audit print command instead of trying to launch claude
   Spawning claude programmatically was causing terminal issues and hangs.
   Now --audit just prints the command for the user to run manually.
 - <csr-id-196774225c8eba52fdb9382f98418ecf82c48567/> prevent shell-setup from corrupting shell config files
   The previous byte-slicing approach in find_path_line_end() calculated
   offsets incorrectly because lines() strips newlines but the code assumed
   +1 byte per line. This could corrupt or truncate config files.

### Refactor

 - <csr-id-225e9a6d7b8837652cae0da672f7b4b6a0cd069b/> implement Op trait and command_enum! macro for CLI
   Introduce a trait-based pattern for CLI commands that provides:
   - Typed errors per command (vs anyhow::Result everywhere)
   - Typed output per command (Display impl for stdout)
   - Unified execution via command_enum! macro
   - Infallible commands use std::convert::Infallible
   
   The macro generates Command enum, OpOutput, OpError, and Op impl,
   reducing boilerplate in main.rs dispatch. Doc comments on Args structs
   are picked up by clap (no duplication needed in cli.rs).
   
   Adds thiserror dependency to jig-cli for per-command error enums.
   Updates docs/PATTERNS.md to document the new pattern.

### New Features (BREAKING)

 - <csr-id-0f3fd3073b7b06f30e4cb6c0ebe1320433a68dff/> restructure jig state directory from .worktrees/ to .jig/
   Move all jig-managed worktrees from <repo>/.worktrees/ to <repo>/.jig/
   and state files to <repo>/.jig/.state/state.json. This provides a
   cleaner directory layout with state files separated from worktrees.
   
   Key changes:
   - Worktrees now live under .jig/ instead of .worktrees/

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELLFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changes<csr-unknown>
<csr-unknown>
Check latest version from GitHub releases APIAuto-update for script installations (~/.local/bin)Prompt dev builds to install release binariesOffer cleanup of old cargo bin after source build updatesAdd –force flag to skip version checkAdd Op trait in crates/jig-cli/src/op.rsRewrite ps command with PsOutput, PsError, and Op implAdd comfy-table dependency for dynamic table renderingUpdate main.rs dispatch to use Op::execute()Add docs/ui/STDOUT-FORMATTING.md documenting the patternworktree.base — base branch for new worktrees (overrides global)worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)State file moved to .jig/.state/state.jsonAuto-migration from .worktrees/ layout on first loadjig kill/unregister now removes workers from state entirely(instead of archiving them)jig ps auto-cleans stale workers whose tmux windows are goneHidden directories (.state) are skipped when listing worktrees.jig/.state/ added to .gitignore, .jig/ added to git exclude<csr-unknown>
 prettify jig ps with Op pattern and comfy-tableIntroduce the Op trait to separate command logic from presentation.Rewrite jig ps as the first adopter: ops return typed data, Displayimpls own all formatting via comfy-table with terminal-width-awarecolumn layout and color-coded status indicators. add worktree.copy for gitignored filesAdds worktree.copy config to copy gitignored files (like .env)to new worktrees:toml[worktree]
copy = [".env", ".env.local"]
Files are copied after worktree creation, before on_create hook runs. add worktree config to jig.tomljig.toml now supports worktree configuration: restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

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

<csr-id-8cce0fba090be552af7b0186f96ad03ffa8b5d81/>
<csr-id-4c9f3184c27cab9ddfc835fdde711ba6af2539ca/>
<csr-id-60460d876900a1fca4dda6e7763127965d7dcb50/>
<csr-id-7bf25cd45434e6c0c9388ac70aadf0cc85cec04e/>
<csr-id-badb4164208b05b288a36391ef046cb7b643ca3e/>
<csr-id-80f3bccb70cdd146ab2eccbeec224a8104db8c61/>
<csr-id-4dd791fdfc3ce463b6642ae45d57062e10f9026b/>
<csr-id-3a78670c102178f25db9dc4020b534370fc36f84/>
<csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/>
<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

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

### Bug Fixes

 - <csr-id-bd9a6c99600670089a646b2e32cb6448d0b234bd/> make --audit print command instead of trying to launch claude
   Spawning claude programmatically was causing terminal issues and hangs.
   Now --audit just prints the command for the user to run manually.
 - <csr-id-196774225c8eba52fdb9382f98418ecf82c48567/> prevent shell-setup from corrupting shell config files
   The previous byte-slicing approach in find_path_line_end() calculated
   offsets incorrectly because lines() strips newlines but the code assumed
   +1 byte per line. This could corrupt or truncate config files.

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELLFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changes<csr-unknown>
<csr-unknown>
worktree.on_create — command to run after worktree creationAdd directory-based issue organization (epics/, features/, bugs/, chores/)Add issue templates (_templates/): standalone.md, epic-index.md, ticket.mdCreate plan-and-execute epic for orchestration visionUpdate issues/README.md with comprehensive documentationUpdate /issues skill for new directory structureRemove old flat issue files and _template.mdAdd .backup/ to .gitignoreAdd AgentType enum for compile-time safe matchingRename template to PROJECT.md (agent-agnostic name)Dynamic audit prompt uses adapter.project_file and adapter.skills_dirValidate agent is installed before init (warns if not in PATH)Fix settings.json schema URLFix settings.json to use correct schemastore.org URLAdd WebFetch, WebSearch, mcp__, jig: to default permissionsUpdate review skill to check jig-specific docs and skillsUpdate issues skill to reference issues/README.mdAdd adapter module with AgentAdapter struct for pluggable agent supportjig init now requires agent argument: jig init claudejig.toml stores agent type in [agent] sectionspawn command uses adapter to build agent-specific commandsMove settings.json to templates/adapters/claude-code/Backup now copies files to .backup/ directory preserving path structureAudit prompt is detailed and opinionated about what to fill in each docReview skill now checks for documentation and skills updatesMove issue-tracking.md to issues/README.md, fix “wt” → “jig”Rename skills/jig → skills/spawn for consistencyRemove name: field from skill frontmatterAdd skeleton docs: PATTERNS.md, CONTRIBUTING.md, SUCCESS_CRITERIA.md, PROJECT_LAYOUT.mdExpand docs/index.md as documentation hubMake CLAUDE.md template a skeleton with guidance commentsUpgrade settings.json: add $schema, ask tier for destructive ops, better secret patternsAdd issues/_template.md ticket templateAdd skills for check, draft, issues, review, and spawn commandsSimplify .claude/settings.json using wildcard permissionsAdd jig.toml with spawn auto-configurationFix formatting in init.rsEmbed templates from templates/ directory using include_str!Add all 5 skills: check, draft, issues, review, spawnExpand permissions to cover tools used by skillsSet spawn.auto = true by defaultUse exec() on Unix for –audit flag (full terminal control)Add jig shell-setup command to automatically configure shell integrationFinds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)<csr-unknown>
 restructure issue tracking with categories and templates improve adapter architecture and audit templatesAdapter improvements:Template improvements: add agent-agnostic adapter architectureThis architecture allows future support for other agents (cursor, etc.)by adding new adapter constants. improve backup, audit prompt, and review skill upgrade jig init scaffolding to language-agnostic skeletons add Claude Code skills and simplify permissions use actual templates for jig init instead of bare-bones placeholdersThe init command now creates a complete scaffolding that matchesthe documentation, instead of empty placeholder comments. add –audit flag to init command that launches Claude interactivelyUses exec() on Unix to replace the current process with Claude Code,giving it full terminal control for interactive documentation audit. add shell-setup command and fix shell completionsRewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

## v0.4.0 (2026-05-15)

### Style

 - <csr-id-20f50084325dd8efff3ae0484c07027230dc1b89/> format assert in test
 - <csr-id-54d326b275eebbc3ff783d8ecbabb0b2bb28a3db/> cargo fmt

### Refactor

 - <csr-id-12c5ea42f34574dfa9afbd1aa2872752089af72f/> make jig.toml model fields Option<String>; defer to AgentBackend default
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
 - <csr-id-e855492f12a87e96e3d2e4826058f5f5b717e45a/> split per-command Output/Error, nest command_enum!
   Each issues and pr subcommand now owns its Output, Error, and Display.
   The vanilla command_enum! macro is invoked recursively in issues/mod.rs
   and pr/mod.rs to generate Command/OpOutput/OpError for each nesting,
   mirroring the top-level pattern.
 - <csr-id-c3a3b156f83255f6feb0b0e4d5854d0af5edf4f9/> extend command_enum! to nested subcommand enums
   IssuesCommand and PrCommand now use the same macro, sharing the
   project's Output/Error types instead of generating wrappers.
 - <csr-id-38fe1e498e8bfbf3b920933d323c695bcb4cd982/> impl Op for IssuesCommand and PrCommand
   Mirror the command_enum! macro pattern: each subcommand enum
   implements Op itself, dispatch logic lives in the enum's run(),
   and the wrapper structs just delegate to the enum.
 - <csr-id-42acb0015ead348c20e6243fc119da452f8f4b2d/> add ScopedCtx::from_global, jig_toml to RepoCtx, dispatch helpers
   - ScopedCtx::from_global(bool) consolidates the global/local branch in build_context
   - RepoCtx now carries jig_toml loaded from repo root; ps uses r.jig_toml directly
   - Generic dispatch() helpers in issues/mod.rs and pr/mod.rs replace verbose build_context + run pairs
   - resume.rs uses ctx.jig_toml instead of reloading
 - <csr-id-b26c1ddd1c3117efa6d4f1fad18750d25b9ca149/> match ScopedCtx directly in issues list run
 - <csr-id-4b9b47f59c01c76937277b471472bde50a0a0dfa/> add associated Context type to Op trait and migrate all commands
   Each command now declares `type Context`, implements `build_context(&self)`,
   and accepts context via `run(&self, ctx)`. The `command_enum!` macro delegates
   context construction to each subcommand variant, keeping the top-level
   dispatch free of context-building logic.
 - <csr-id-2946d279e3b72fd97ef198894a707b668295a1a6/> decouple ui from ps and split primitives into colors/output
   Split cli::ui (722 lines) into ui/mod.rs + ui/colors.rs + ui/output.rs
   (285 lines total). Move all WorkerStatus/TriageEntry rendering from ui
   into cli/commands/ps/render.rs, colocating domain views with the command
   that owns them. ui/mod.rs re-exports colors::* and output::* so all
   existing call sites compile unchanged.

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
 - <csr-id-7b350e40475d4c612bb05300f7e41769a56929a9/> cargo fmt formatting fixes
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
 - <csr-id-a138f4f5930af98b176c8a89783b425233577259/> fix clippy and formatting errors
   - Remove unit let-bindings (main.rs, pr/mod.rs) flagged by clippy::let_unit_value
   - Suppress clippy::large_enum_variant on ScopedCtx (CLI context enum, not hot path)
   - Run cargo fmt to fix formatting in list.rs and pr/create.rs
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
 - <csr-id-61c8ad3c0fc9e3ef6a47c356e8a880ee1f05afce/> keep from_cwd registry minimal; persist as side-effect
   The previous approach replaced from_cwd's in-memory registry with the
   full global one, which misunderstands the intent: from_cwd is a
   single-repo context and its registry should stay that way.
 - <csr-id-50715c855e00c63600c75031fd6203388aec8e04/> persist repo to disk on every from_cwd call
 - <csr-id-bc6271b116b9ae8ac6599896019f71dc408dadd7/> load jig.local.toml standalone when jig.toml is absent
   Previously JigToml::load returned None immediately if jig.toml did not
   exist, silently ignoring any jig.local.toml present in the repo. Repos
   with only a local config were treated as unconfigured, causing the daemon
   to skip them with no diagnostic.
   
   Now if jig.toml is absent but jig.local.toml exists, the local file is
   parsed as the full standalone config (local_only = true). The overlay
   merge path is unchanged when both files are present.
   
   Also adds JigToml::local_only_exists and a health-check warning (not
   failure) that surfaces the no-base-file situation to the user.
 - <csr-id-c751d2c2df7283e8f787781202c8cef9d1499dbf/> emit PrMerged/PrClosed events from process_pr_report
   The merged/closed branch of process_pr_report was emitting PrOpened (or
   nothing) instead of the terminal events that drive worker cleanup.
   Because no producer ever wrote PrMerged or PrClosed, the reducer could
   never transition a worker to Merged/Failed status, so the recovery prune
   never received a PruneTarget and worktrees/tmux windows were never
   cleaned up.
   
   Split the combined arm into two separate arms that emit the matching
   event variants. Guard with !state.status.is_terminal() to avoid
   re-appending on every subsequent poll once the worker has already
   reached a terminal state.
 - <csr-id-adbce583457c3b46622874c795e8b3b366b5491a/> remove unused import and use struct init syntax in tests
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

 - <csr-id-18caace98edbe08f90d82f1362ece217f5c791ab/> bump version to 0.4.0
 - <csr-id-e1dfd6b90ac3e80c6af9fd46fd1e19fd0207b3f5/> bump version to 0.3.1
 - <csr-id-6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02/> bump version to 0.3.0
 - <csr-id-9f12c306cfcfb1eea60707d32360eb89479993a9/> bump version to 0.2.0

### New Features

<csr-id-0ab34082c061a8ffba63413c3a6b7e397d12de6f/>
<csr-id-5a59d80324580c092cdda14ce2e2faebf535b444/>

 - <csr-id-f05d75ea429a873ac6f749928f49cb9d850b22eb/> add shell-setup command and fix shell completions
   - Add `jig shell-setup` command to automatically configure shell integration
 - <csr-id-71005f8ecce41c281bad27029d1b5a051de44df3/> add RepoCtx, GlobalCtx, and ScopedCtx context types
   Introduces three typed context shapes alongside the existing Context:
   - RepoCtx: single-repo context (repo + config) from cwd
   - GlobalCtx: all-repos context (config + registry + repos)
   - ScopedCtx: enum of Repo/Global for commands with --global
   
   Adds From impls to convert both new types back to the legacy Context,
   enabling gradual migration of downstream code.
 - <csr-id-4d15f3503a4748656801d0fb34758ac1420b1cae/> add RepoConfig::name() and replace all call sites
   Adds a `name()` method to `RepoConfig` that extracts the display name
   from `repo_root.file_name()` with "unknown" as fallback. Replaces all
   open-coded occurrences of this pattern across command files.
 - <csr-id-7f3e4bae3f9bd9f5a1a813d8d80e0d1652a1814d/> add gh CLI binary and auth checks to System section
   Adds `gh` binary detection and `gh auth status` verification to the
   System section of `jig health`, so users are warned before `jig pr`
   fails due to a missing or unauthenticated `gh` CLI.
 - <csr-id-5cff652c9698daa6a067086234c50c6abd884517/> jig — git worktree manager for parallel Claude Code sessions

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 50 commits contributed to the release.
 - 33 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump version to 0.4.0 ([`18caace`](https://github.com/krondor-corp/jig/commit/18caace98edbe08f90d82f1362ece217f5c791ab))
    - Merge pull request #30 from krondor-corp/feature/kro-153-jig-create-must-validate-remote-exists-before-falling-back ([`b0f842d`](https://github.com/krondor-corp/jig/commit/b0f842d7cbfaeae96e1da03bae8061384fecbeec))
    - Merge pull request #34 from krondor-corp/feature/kro-152-consolidate-model-defaults-to-agentbackend-make-jigtoml ([`05e93ce`](https://github.com/krondor-corp/jig/commit/05e93ce96c1f00065af92aa5b1370eaed48420f1))
    - Document branch resolution order; log when base is ignored ([`a359f75`](https://github.com/krondor-corp/jig/commit/a359f75431686c36f688a905ec9c353c50bf2929))
    - Check out remote branch when origin/<branch> already exists ([`28aca79`](https://github.com/krondor-corp/jig/commit/28aca7958072029c21c92bd8782e6dc51a01b7fe))
    - Make jig.toml model fields Option<String>; defer to AgentBackend default ([`12c5ea4`](https://github.com/krondor-corp/jig/commit/12c5ea42f34574dfa9afbd1aa2872752089af72f))
    - Merge pull request #31 from krondor-corp/feature/kro-149-add-generic-eventk-in-jig-core-consolidate ([`5f96611`](https://github.com/krondor-corp/jig/commit/5f966116cf0960ce09f24db54a3b4733cf6fe4a2))
    - Cargo fmt formatting fixes ([`7b350e4`](https://github.com/krondor-corp/jig/commit/7b350e40475d4c612bb05300f7e41769a56929a9))
    - Add generic Event<K> and consolidate event scaffolds ([`0a996ce`](https://github.com/krondor-corp/jig/commit/0a996ced56110b176b3de348cbbd0f510b4873ac))
    - Validate remote exists before creating worktree; drop HEAD fallback ([`e732f67`](https://github.com/krondor-corp/jig/commit/e732f6773fa0575835719ea68aeaac8219db33b8))
    - Merge pull request #26 from krondor-corp/feature/kro-148-refactor-op-trait-to-associated-context-type-and-add ([`c0c2980`](https://github.com/krondor-corp/jig/commit/c0c29807c931a1cb3495fbf40e7fc4475cd6e8e6))
    - Split per-command Output/Error, nest command_enum! ([`e855492`](https://github.com/krondor-corp/jig/commit/e855492f12a87e96e3d2e4826058f5f5b717e45a))
    - Extend command_enum! to nested subcommand enums ([`c3a3b15`](https://github.com/krondor-corp/jig/commit/c3a3b156f83255f6feb0b0e4d5854d0af5edf4f9))
    - Impl Op for IssuesCommand and PrCommand ([`38fe1e4`](https://github.com/krondor-corp/jig/commit/38fe1e498e8bfbf3b920933d323c695bcb4cd982))
    - Add ScopedCtx::from_global, jig_toml to RepoCtx, dispatch helpers ([`42acb00`](https://github.com/krondor-corp/jig/commit/42acb0015ead348c20e6243fc119da452f8f4b2d))
    - Match ScopedCtx directly in issues list run ([`b26c1dd`](https://github.com/krondor-corp/jig/commit/b26c1ddd1c3117efa6d4f1fad18750d25b9ca149))
    - Fix clippy and formatting errors ([`a138f4f`](https://github.com/krondor-corp/jig/commit/a138f4f5930af98b176c8a89783b425233577259))
    - Add associated Context type to Op trait and migrate all commands ([`4b9b47f`](https://github.com/krondor-corp/jig/commit/4b9b47f59c01c76937277b471472bde50a0a0dfa))
    - Add RepoCtx, GlobalCtx, and ScopedCtx context types ([`71005f8`](https://github.com/krondor-corp/jig/commit/71005f8ecce41c281bad27029d1b5a051de44df3))
    - Add RepoConfig::name() and replace all call sites ([`4d15f35`](https://github.com/krondor-corp/jig/commit/4d15f3503a4748656801d0fb34758ac1420b1cae))
    - Merge pull request #25 from krondor-corp/feature/kro-147-decouple-cliui-from-ps-and-split-primitives-into ([`1da8b63`](https://github.com/krondor-corp/jig/commit/1da8b631675f6adacb1e2a621ddcced319272b15))
    - Decouple ui from ps and split primitives into colors/output ([`2946d27`](https://github.com/krondor-corp/jig/commit/2946d279e3b72fd97ef198894a707b668295a1a6))
    - Merge pull request #21 from krondor-corp/release-automation ([`6e5e121`](https://github.com/krondor-corp/jig/commit/6e5e1218c422440dfe296c5a3aade37f7247f1d8))
    - Bump jig-core v0.3.1, jig-cli v0.3.1 ([`c834648`](https://github.com/krondor-corp/jig/commit/c83464870957aedcebc37cd4136472cd404c5537))
    - Bump version to 0.3.1 ([`e1dfd6b`](https://github.com/krondor-corp/jig/commit/e1dfd6b90ac3e80c6af9fd46fd1e19fd0207b3f5))
    - Merge pull request #19 from krondor-corp/release-automation ([`eb8e28c`](https://github.com/krondor-corp/jig/commit/eb8e28c03456fcab820c9281ccbed24c9800feb5))
    - Bump jig-core v0.3.0, jig-cli v0.3.0 ([`ea44da2`](https://github.com/krondor-corp/jig/commit/ea44da2fd4eda8b11533f33b23f753eef5fdcdc0))
    - Bump version to 0.3.0 ([`6bd4cd5`](https://github.com/krondor-corp/jig/commit/6bd4cd521f95bfd6fbd8fdba97a0a642e1725a02))
    - Merge pull request #14 from krondor-corp/fix/claude-allowed-tools-and-triage-tracker ([`4aa4eb7`](https://github.com/krondor-corp/jig/commit/4aa4eb704216a1c4967a90a25f5d82dffbea0d40))
    - Merge pull request #11 from krondor-corp/feature/kro-131-reporegistryregister-is-in-memory-only-no-command-persists ([`79bc2ab`](https://github.com/krondor-corp/jig/commit/79bc2abb8ee17d5ec5942df03e84f8c21bc3353b))
    - Pass prompt via --allowed-tools= form and clear tracker after run ([`50c054d`](https://github.com/krondor-corp/jig/commit/50c054d030c0d8a42fa6f105c4771c606c94810d))
    - Keep from_cwd registry minimal; persist as side-effect ([`61c8ad3`](https://github.com/krondor-corp/jig/commit/61c8ad3c0fc9e3ef6a47c356e8a880ee1f05afce))
    - Merge pull request #10 from krondor-corp/feature/kro-133-jigtomlload-returns-none-when-only-jiglocaltoml-exists-local ([`197f0df`](https://github.com/krondor-corp/jig/commit/197f0dff4c4462835bf56fb12cc31693d41d43ee))
    - Format assert in test ([`20f5008`](https://github.com/krondor-corp/jig/commit/20f50084325dd8efff3ae0484c07027230dc1b89))
    - Merge pull request #9 from krondor-corp/feature/kro-132-mergedclosed-prs-never-trigger-worker-cleanup ([`007362c`](https://github.com/krondor-corp/jig/commit/007362c96932c28a2e2f4c1f1d6eb1a6d65a8477))
    - Persist repo to disk on every from_cwd call ([`50715c8`](https://github.com/krondor-corp/jig/commit/50715c855e00c63600c75031fd6203388aec8e04))
    - Load jig.local.toml standalone when jig.toml is absent ([`bc6271b`](https://github.com/krondor-corp/jig/commit/bc6271b116b9ae8ac6599896019f71dc408dadd7))
    - Emit PrMerged/PrClosed events from process_pr_report ([`c751d2c`](https://github.com/krondor-corp/jig/commit/c751d2c2df7283e8f787781202c8cef9d1499dbf))
    - Merge pull request #7 from krondor-corp/feature/kro-137-test-verify-daemon-spawn-for-jig ([`fd5f56c`](https://github.com/krondor-corp/jig/commit/fd5f56cf36f050394e49120842eaca2527c47c7c))
    - Remove unused import and use struct init syntax in tests ([`adbce58`](https://github.com/krondor-corp/jig/commit/adbce583457c3b46622874c795e8b3b366b5491a))
    - Resolve clippy warnings in issues list and format pr create ([`cc11b78`](https://github.com/krondor-corp/jig/commit/cc11b783045bb5d4d897e5e7660557ced9c41cc5))
    - Pass --head flag to gh pr create for git worktree compatibility ([`9e3dedd`](https://github.com/krondor-corp/jig/commit/9e3deddc1a6070d619d13874f63a6343b7e6ab02))
    - Merge pull request #5 from krondor-corp/feature/kro-127-jig-health-does-not-check-for-local-gh-cli-health ([`3e45439`](https://github.com/krondor-corp/jig/commit/3e454394d53cbf66e1117321d98e564c2765f0ec))
    - Add gh CLI binary and auth checks to System section ([`7f3e4ba`](https://github.com/krondor-corp/jig/commit/7f3e4bae3f9bd9f5a1a813d8d80e0d1652a1814d))
    - Resolve clippy warnings for CI ([`ba3ee31`](https://github.com/krondor-corp/jig/commit/ba3ee31ba372af56cf1ed953128792b433deaf8c))
    - Merge pull request #2 from krondor-corp/release-automation ([`80abbce`](https://github.com/krondor-corp/jig/commit/80abbcee7ebe9c83aa89b557859ea7ae36fa9b31))
    - Bump jig-core v0.2.0, jig-cli v0.2.0 ([`678f85a`](https://github.com/krondor-corp/jig/commit/678f85ac899e871014b67a93ec4b26fe33693465))
    - Bump version to 0.2.0 ([`9f12c30`](https://github.com/krondor-corp/jig/commit/9f12c306cfcfb1eea60707d32360eb89479993a9))
    - Cargo fmt ([`54d326b`](https://github.com/krondor-corp/jig/commit/54d326b275eebbc3ff783d8ecbabb0b2bb28a3db))
    - Jig — git worktree manager for parallel Claude Code sessions ([`5cff652`](https://github.com/krondor-corp/jig/commit/5cff652c9698daa6a067086234c50c6abd884517))
</details>

<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
<csr-unknown>
Detects user’s shell from $SHELL<csr-unknown>
Finds appropriate config file (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish)Adds eval line with markers for easy identificationPlaces integration after PATH setup when possibleSupports –dry-run flag to preview changesjig open/attach/review/merge/kill/status <TAB> shows actual worktreesContext-aware completions for all subcommandsSimplified zsh completion using _arguments -CAdd quick setup section for shell-setup commandAdd troubleshooting section for common issuesRemove stale sc alias references (legacy from “scribe” name)<csr-unknown>
Rewrite shell completions with dynamic worktree completionUpdate docs/usage/shell-integration.md rewrite health check to validate repo setup and agent scaffoldingReplace terminal-detection-focused health check with structured validationof system deps (git, tmux, claude), repository config (jig.toml, basebranch, .worktrees), and agent scaffolding (CLAUDE.md, settings, skills).Remove unused jq/gh dependency checks and dead required field. Exitnon-zero when checks fail. add shell completions for bash, zsh, and fishShell completions are now emitted alongside the shell wrapper functionin jig shell-init. Completions cover all subcommands, aliases,per-command flags, nested config subcommands, and dynamic worktreename completion via command jig list.<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>
<csr-unknown/>

