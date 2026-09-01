# pack integration plan — jig as the edge client of a shared agent platform

Status: design spike (2026-06-28). Tracked in Linear under epic **KRO-164**.

> **2026-09-01: promoted.** This plan's v0 (pull-only doc → skill sync) is now Phase A
> of [knowledge-platform-spec.md](./knowledge-platform-spec.md) — jig's primary
> differentiator, not a spike. That spec also resolves two things left open here:
> curation is human-only (no agent write-back), and sync runs automatically at
> `jig spawn`/`jig resume`, not as a manual step. Read it first; this doc is now the
> client-implementation reference.

## Summary

Today jig orchestrates coding agents from the outside: it creates worktrees, writes the
files an agent expects (`AGENTS.md`, `.claude/skills/`, `.claude/settings.json`), spawns the
agent into a mux window (tmux or herdr), and observes it via event hooks. The agent never knows jig exists
(`crates/jig-core/src/agents/mod.rs:1-31`).

This plan adds a second relationship: jig also becomes a **client of a shared backend
platform** — [`pack`](https://github.com/krondor-corp/pack) — from which it syncs **skills,
prompts, memory, and code-quality conventions**, so your context travels across agents and
machines, and an organization can edit a convention once and roll it out fleet-wide.

**Decision: integrate, don't merge.** pack and jig stay separate services:

- **pack = the platform / source of truth.** A long-running Axum server (Postgres + pgvector +
  auth) that stores markdown documents, embeds them, and exposes an MCP tool surface.
- **jig = the edge client.** A local CLI/daemon that *pulls* profile documents from pack and
  *installs* them into whatever agent, plus drives local git/tmux/worktrees (the part that
  physically cannot move server-side).

The runtimes are incompatible (server vs. local executor); the boundary is a clean HTTP/MCP
API. We keep the name **pack** for the platform.

## What pack provides today (verified against the codebase)

pack needs **no changes** for the v0 sync MVP. Relevant facts:

- **MCP is the only machine API.** Mounted at `POST /mcp`, JSON-RPC 2.0, auth required
  (`crates/app/src/http/mcp/mod.rs`, `handler.rs`). No REST endpoints; the rest is HTMX UI +
  an SSE channel at `/_events` + a CRDT sync channel at `/files/sync/:id`. No standalone CLI.
- **11 tools, full CRUD + search** (`crates/app/src/state.rs:59-71`):
  `ping`, `fs_ls`, `fs_cat`, `fs_stat`, `fs_find_by_tags`, `fs_write`, `fs_edit`, `fs_rm`,
  `search_documents`, `list_tags`, `set_tags`. The write tools are shipped (not stubs).
- **Document model (`fs_node`)**: `{ id, user_id, path, type(document|folder), content,
  tags: text[], created_at, updated_at }` (`crates/core/src/database/models/fs_node.rs:67-78`).
  Paths are POSIX-like and unique per user. A `document_version` table exists
  (`migrations/20260414100000`) → drift detection is possible via `updated_at`.
- **Tags** are free-form strings on a `text[]` column (GIN-indexed). Filtering:
  - `fs_find_by_tags(tags)` → Postgres array-overlap `tags && $2` = **ANY/OR**, returns
    **paths + tag sets only, no content** (`fs.rs:212`, `fs_node.rs:727-743`).
  - `search_documents(query, tags, match)` → embedding search + post-filter with
    `match="all"` (AND) or `"any"` (OR) (`search.rs:298-306`).
  - There is **no tag-based visibility/isolation** — nothing is hidden by default; all scoping
    is per-`user_id`.
- **Embeddings are async.** `fs_write`/`fs_edit` enqueue an Apalis chunk+embed job with a
  **30s debounce** (`crates/app/src/tasks/mod.rs:60-106`). So `search_documents` is
  eventually-consistent; `fs_cat`/`fs_ls`/`fs_find_by_tags` (which hit `fs_node` directly) are
  immediately consistent.
- **Auth**: session cookie (HS256 JWT) **or** a `pk_*` Bearer API token (SHA256-hashed,
  revocable — `migrations/20260415040000`, `api_key.rs:87-123`). **Single-tenant today**:
  every query is `WHERE user_id = $1`, gated behind Google OAuth + admin approval
  (`auth/mod.rs:63-68`).

## The integration on the jig side

All integration code lives in jig; pack ships unchanged. Model it on the **existing Linear
provider**, which is already a "ureq-over-HTTP, adapt an external API to a jig-native shape"
module (`crates/jig-core/src/issues/providers/linear/`).

### Module layout (mirror of Linear)

| Linear (today) | pack (new) |
|---|---|
| `issues/providers/mod.rs` (`IssueBackend` trait + handle) | `crates/jig-core/src/pack/mod.rs` — concrete `Pack` handle. Do **not** put it under `issues/providers`; different domain. No trait until a 2nd backend exists (YAGNI). |
| `linear/mod.rs` — `Linear` struct + config + high-level methods | `pack/mod.rs` — `Pack { endpoint, token }` + methods `find_by_tags`, `cat`, `write`, `edit`, `list_tags` |
| `linear/client/client.rs` — ureq POST, GraphQL envelope | `pack/client.rs` — ureq POST to `/mcp`, JSON-RPC 2.0 envelope |
| `linear/client/{request,types,error}.rs` | same: request envelope, DTOs (`FsNodeMeta`, `Doc`), `PackError` |
| `linear/client/queries/*` + `mutations/*` (one file per GraphQL op) | **collapses to one `call_tool` + ~5 typed wrappers** — MCP is uniform (`tools/call`), so no per-op file sprawl |
| `config.rs` `[linear.profiles.<name>]` → `{ api_key, ... }` | `config.rs` `[pack]` → `{ endpoint, token }` (+ optional `profiles` map for multiple instances) |
| `repo.linear_provider(global)` | `repo.pack_client(global)` — identical wiring |

### The transport (the whole client, essentially)

```rust
// pack/client.rs
fn call_tool(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, PackError> {
    let body = json!({
        "jsonrpc": "2.0", "id": 1,
        "method": "tools/call",
        "params": { "name": name, "arguments": args }
    });
    let resp: JsonRpcResp = ureq::post(&format!("{}/mcp", self.endpoint))
        .set("Authorization", &format!("Bearer {}", self.token))   // pk_* token
        .send_json(body)?
        .into_json()?;
    resp.into_result()                                              // unwrap JSON-RPC {result|error}
}
```

Differences from Linear to respect:
1. **Double-unwrap**: JSON-RPC `{result|error}`, *and* each tool result wraps its payload in
   MCP's `{"content":[{"type":"text","text":"<json string>"}]}`. Parse with typed structs and
   fail loud — no `Value`-poking with silent `unwrap_or` defaults.
2. **No domain trait** for v0 — concrete `Pack` struct only.
3. **Auth** is `Bearer pk_*`; token stored in global `~/.config/jig/config.toml`, never
   committed (mirror Linear's `api_key` handling — `crates/jig-cli/src/context/config.rs`).

## v0 MVP: pull-only doc → skill sync

> A repo subscribes to a profile (a tag-set + target). `jig sync` pulls every matching
> document from pack and installs it as a skill. Edit a doc in pack → re-sync → agent sees it.

Pull-only, read-only, no CRDT, no worktree tracking. Proves the whole thesis (central edit →
fleet rollout) and touches **only jig**.

### Config (`jig.toml`)

```toml
[profiles.rust-skills]
tags   = ["skill", "rust"]   # a document must carry ALL of these
target = "skills"            # skills | prompts | memory

[sync]
profiles = ["rust-skills"]
```

`jig sync --tags skill,rust` is an ad-hoc escape hatch taking the same code path.

### The sync loop, and the AND gotcha

The user's model — "every document tagged `skill` AND `rust`" — needs **AND**, but the
immediately-consistent tool (`fs_find_by_tags`) is **OR-only**. The AND-capable tool
(`search_documents match="all"`) is the eventually-consistent embedding path and needs a query
string — wrong for deterministic sync. Fix: **`fs_find_by_tags` already returns each doc's
full tag set, so intersect client-side** (zero extra calls), then `fs_cat` only the survivors:

```rust
let profile = cfg.profiles["rust-skills"];

// 1. OR query on the cheapest tag — returns paths + tag sets, no content
let candidates = pack.find_by_tags(&[&profile.tags[0]])?;          // ["skill"]

// 2. enforce AND in jig (tags are already in hand)
let matched = candidates.into_iter()
    .filter(|d| profile.tags.iter().all(|t| d.tags.contains(t)));

// 3. fetch content + install only survivors
for d in matched {
    let doc  = pack.cat(&d.path)?;                                 // content now
    let name = skill_name(&d.path);                                // path basename, or frontmatter `name:`
    install_skill(&name, &doc.content)?;                           // .claude/skills/<name>/SKILL.md
}
```

`install_skill` reuses jig's existing skills-install path (the code that projects
`templates/skills/` into `.claude/skills/` during `jig init`, via each backend's
`Agent::skills_dir()`/`skill_file()` — see `crates/jig-cli/src/cli/commands/init.rs:169-280`
and, for Claude specifically, `crates/jig-core/src/agents/claude/mod.rs:76-79`).

### Convention decisions (not pack changes)

- **doc → skill mapping**: start with a path convention (`/skills/<name>/SKILL.md`, name =
  segment). Frontmatter `name:` is the more flexible alternative.
- **profile = fixed tag-set or ad-hoc mix**: support named profiles in `jig.toml` (common
  case) + `--tags` (ad-hoc), one code path.

### Tag/scoping convention (project isolation without pack changes)

```
scope:global   → general standards every project pulls
project:<name> → project-scoped docs
profile:<name> → groups docs into a named profile
skill / rust   → kind + topic
```

Because jig controls the queries it issues, isolation works today with zero pack changes:
syncing repo *jig* calls `find_by_tags(["scope:global", "project:jig"])` (overlap/OR) → gets
global + jig docs, and **never** sees `project:confit` because it never asks. The only leak is
a *different* consumer (pack's chat UI, another agent) doing an unscoped search — that's what
"first-class exclusive tags" (below) is for.

## Roadmap / phases

| Phase | What | Where the work lands |
|---|---|---|
| **v0** | pull-only doc → skill sync (this doc) | jig only |
| v1 | prompt sync (nudge text, spawn/issue-wrap templates externalized to pack docs, rendered via the Handlebars `Prompt` builder with built-in fallback) | jig only |
| v2 | bidirectional / push via the **CRDT** pack already runs for its editor (`crates/crdt`) — conflict-free concurrent edits to skills/prompts/memory | jig + pack |
| v3 | **worktree-session tracking** — jig's daemon pushes worker events/state to pack; pack aggregates a cross-device dashboard (SSE infra at `/_events` already exists) | jig + pack |
| future | **commit review/proxy** — commits already flow to pack via the worktree-event sink; add a review/comment UI; comments become nudges back to the agent | jig + pack |

## pack-side feature ladder (only when going first-class)

The v0/v1 MVP needs none of these. Each is independent.

1. **Org / multi-user sharing** *(the big one)* — pack is single-tenant; org-wide profiles need
   multi-user + shared/published docs. Bridge until then: a dedicated "org" pack account whose
   read-only `pk_*` token is distributed.
2. **Exclusive / scoped tags** — enforced isolation: a visibility flag on `tag_definition`
   (which already has `description`/`color`/`archived_at`, `migrations/20260414002000`) or an
   `@`-prefixed naming rule the search path respects, so project docs are hidden from unscoped
   searches by default.
3. **Worktree-session domain** — new tables + an ingest tool/endpoint + a dashboard view.
4. *(nice-to-have)* a `get_profile` bulk tool (one round-trip vs. `find_by_tags` + N×`fs_cat`)
   and a `match="all"` mode on `fs_find_by_tags` (`tags @> $2`) once doc sets get large.

## References

- jig: `crates/jig-core/src/issues/providers/linear/` (the pattern to mirror),
  `crates/jig-cli/src/cli/commands/init.rs:169-280` (skills-install target),
  `crates/jig-core/src/prompt.rs` (prompt rendering for v1),
  `crates/jig-cli/src/context/config.rs` (config + token storage),
  `crates/jig-cli/src/daemon/` (worktree-event sink for v3), `templates/skills/`.
- pack: `crates/app/src/http/mcp/` (tools), `crates/core/src/database/models/fs_node.rs`
  (doc + tag model), `crates/app/src/tasks/mod.rs` (async embed), `crates/app/src/http/auth/`
  (token model), `crates/crdt/` (v2 push).
