---
description: Search Rust `///` doc comments across the workspace for canonical-pattern notes, "do not" tripwires, intent docs, or any committed prose. Use before adding a new query/client method/abstraction to check whether existing code already documents the canonical approach.
allowed-tools:
  - Bash(rg:*)
  - Bash(cargo:*)
  - Read
  - Grep
---

Search Rust documentation comments (`///`) for prose that documents intent, conventions, or tripwires.

## When to use this

- You're about to add a new query, client method, helper, or abstraction — check first that existing code doesn't already document a canonical pattern you should reuse.
- You suspect a behavior or rule has been written into a `///` doc comment somewhere and want to find it without reading every file.
- You're investigating "why is it done this way" and the answer is more likely in inline docs than in markdown.

This is **not** for searching project markdown — use the `docs` skill for that.

## Steps

### 1. Search doc comments by distinctive phrase

```
rg --type rust '///.*<KEYWORD>' crates/
```

Returns `file:line:content` matches. Use a phrase the author of the canonical doc would have written — e.g., "single underlying", "do not add", "canonical", "single source of truth". Case-insensitive: add `-i`.

### 2. Show the item being documented

```
rg --type rust -B 1 -A 8 '///.*<KEYWORD>' crates/
```

`-B 1` usually catches the `pub fn` / `pub struct` line that follows the doc comment. `-A 8` shows enough of the doc body to read the full note.

### 3. Multi-line doc blocks

`rg` is line-oriented. For tripwires that wrap, search a single distinctive phrase from inside the block rather than trying to match the whole structure:

```
rg --type rust '///.*Do not add' crates/
```

This works because **the authoring convention (below) keeps a distinctive phrase on at least one line**.

### 4. Fall back to rendered docs for name-based search

For "I don't remember what it's called but it does X" — names + signatures + summaries:

```
cargo doc --no-deps --open
```

Uses rustdoc's built-in search bar. Indexes item names; partial descriptions are matched. Good for discovery, not for tripwire-finding.

## Authoring convention (when writing canonical-pattern doc comments)

For doc comments that should be discoverable by this skill (i.e., they're acting as tripwires, not just explaining what code does), put **a distinctive phrase on at least one line** of the doc body. Examples that work:

- `/// **Do not** add a separate \`GetIssue\` query …` — `rg '///.*Do not add'`
- `/// This is the **single** underlying GraphQL operation …` — `rg '///.*single underlying'`
- `/// NOTE(canonical): the field set lives here.` — `rg 'NOTE\(canonical\)'`

Avoid generic phrases like "this function does X" — too common to filter.

When a tripwire matters enough to enforce, prefer the explicit `NOTE(canonical):` sigil so future searches don't depend on remembering the author's specific wording.
