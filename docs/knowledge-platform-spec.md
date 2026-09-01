# jig knowledge platform spec — pack as the shared brain, jig as the always-fresh install layer

Status: accepted direction (2026-09-01). Spec only — no code yet. Promotes
[pack-integration-plan.md](./pack-integration-plan.md)'s v0 from "design spike" to
jig's primary differentiator, and resolves two design questions that plan left open.

## The differentiator

Every repo today accrues its own conventions, gotchas, and patterns in files an agent
reads — `AGENTS.md`, `.claude/skills/`. That's fine for genuinely repo-specific
knowledge. It's the wrong shape for knowledge that **spans repos**: "we always do X
with retries," "this framework's Y footgun bites every project," "here's our house
style for error handling." Today that either gets copy-pasted repo to repo and drifts,
or lives in someone's head.

The differentiator: **pack is a live, curated, cross-repo knowledge base for agent
context, and jig is the layer that keeps every repo's agent automatically current with
it** — no PR, no copy-paste, no remembering to sync.

## Why this doesn't map onto version control

Git conventions are repo-scoped and require a commit + PR to change. A pattern that
belongs to no single repo, and that a human wants to tweak fluidly and have every
agent everywhere pick up immediately, doesn't fit that model — there's no natural repo
to PR against, and "edit, then wait for N repos to each merge the update" defeats the
purpose. Pack already isn't git: documents are mutable, edited in place, with their own
`document_version` history (`pack-integration-plan.md` §"What pack provides"). That's
the right substrate. jig's job is to be a faithful, boring pipe from it into every
repo's live agent surface — not to reinvent a second version-control layer on top.

## Identity

- **pack = the platform of record.** Humans author and edit patterns/conventions/
  gotchas there, directly and collaboratively, tagged for scope (see below). pack owns
  the only editing surface — jig builds none.
- **jig = the sync/install layer.** It pulls what a repo subscribes to and projects it
  into that repo's live agent context (skills today; `AGENTS.md` fragments and prompts
  next), automatically, at the moment an agent is about to work — not on a schedule
  someone has to remember.

## Decision: curation is human-only

Considered and rejected for v0/v1:
- **Agent self-report** (agent calls a "remember this" tool mid-session) — rejected.
  What one session in one repo notices isn't yet a cross-repo pattern; deciding that
  requires human judgment, not an agent's in-the-moment guess.
- **Session-end extraction** (jig proposes docs from a finished worker's diff/log) —
  rejected for the same reason, plus it couples pack writes to jig's event pipeline,
  which is exactly the coupling this spec avoids.

jig **never writes to pack** in v0/v1. A later, explicitly speculative phase (Phase D
below) might have jig *suggest* candidates for a human to curate — never write
unattended.

## Decision: pack-sourced content is generated, never hand-edited in-repo

The conflict: jig already writes real, git-tracked files into a worktree
(`AGENTS.md`, `.claude/skills/`) that a human might also hand-edit. Once pack content
flows into the same files, a human edit and the next sync collide silently.

**Rule: pack-sourced content always lands in its own files, never merged into a
human-owned file.** One pack skill doc → one skill directory jig fully owns
(`.claude/skills/<name>/SKILL.md`, matching pack's doc-per-path model and jig's
existing skill-directory-per-skill convention). Every jig-managed file gets a banner:

```
<!-- Synced from pack — edit at {pack_url}/{path}, not here. Overwritten on next `jig sync`. -->
```

Repo-native conventions (genuinely specific to this one repo) stay exactly as they are
today: written once at `jig init`, human-owned, evolve via normal repo PRs, never
touched by sync. No shared file ever mixes both — that's what makes overwrite-on-sync
safe instead of a merge problem.

## Decision: freshness is automatic, at spawn time

"Fluid and collaborative" means a human edits a pattern in pack and expects the next
agent anywhere to have it — not "after someone remembers to run `jig sync`."

**v0/v1: sync runs automatically as part of `jig spawn` / `jig resume`**, before the
agent's context is assembled. Zero new infrastructure — no daemon, no background
process, just one more step in a path jig already runs. Manual `jig sync [--tags ...]`
stays as an explicit re-sync / dry-run escape hatch for a long-lived session.

Background re-sync for a worker that's been running for hours (so it picks up a
mid-session pack edit without a restart) is a later, optional phase — and explicitly
does **not** require the `fleet-spec.md` daemon-service refactor to happen first; it
would ride the existing tick loop if and when that work lands, but isn't blocked on it.

## Scoping model

Already sketched in `pack-integration-plan.md` §"Tag/scoping convention" — now
load-bearing, not incidental, since it's the core mechanic:

```
scope:global   → every repo pulls this
project:<name> → this repo only
profile:<name> → named group of docs a repo subscribes to
skill / rust / ... → kind + topic, for organizing within a scope
```

Because this is now central: `jig init` and `jig config` should make "what profiles/
tags does this repo subscribe to" a first-class, discoverable setting — not a toml key
you have to already know to write.

## Phases

**Phase A — the core loop** (mechanically = `pack-integration-plan.md` v0, reframed as
the product, not a spike):
- Pack client, mirroring the Linear provider pattern already speced in
  `pack-integration-plan.md` §"The integration on the jig side" — adopt as-is.
- `jig sync`: tag-AND resolution over the OR-only `fs_find_by_tags` (client-side
  intersect, per that plan's documented workaround), install matched docs as
  jig-managed skill directories per the boundary rule above.
- New vs. the original plan: **wire sync into `jig spawn`/`jig resume`** so it's
  automatic, not a separate manual step.
- `jig init`/`jig config`: first-class profile/tag subscription UX.

**Phase B — more target types**: `AGENTS.md` fragments and prompt templates as
separate pack-managed files (the original plan's v1), same ownership boundary as
skills — never merged into a human-owned file.

**Phase C — multi-user, only if needed**: org-wide sharing and exclusive/scoped tags
(`pack-integration-plan.md`'s feature ladder #1/#2). Not needed for a single-operator
setup; defer until it's actually a bottleneck.

**Phase D — speculative, not committed**: jig notices repeated friction across repos
(same gotcha hit N times) and *suggests* a pack doc for a human to write — a
suggestion feed, never an auto-write. Only worth building once there's enough volume
that noticing patterns manually is the bottleneck.

## Relationship to fleet-spec.md

Orthogonal axis, no dependency in either direction. `fleet-spec.md` is about *where
and how work runs* (mux backends, multi-machine control plane). This spec is about
*what agents know* while they work. The daemon-as-service PR from `fleet-spec.md` can
proceed on its own schedule — nothing here is blocked on it, and nothing there is
blocked on this.

## Non-goals

- Any jig-side editing UI for pack documents — pack owns authoring, full stop.
- Agent- or session-driven write-back to pack (rejected above).
- CRDT / bidirectional sync (`pack-integration-plan.md`'s v2) — not part of this
  differentiator's core loop; revisit only if human-only curation becomes a
  bottleneck.
- Coupling freshness to the daemon-service refactor.
