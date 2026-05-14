# Project Guide

Git worktree manager for parallel Claude Code sessions.

## Quick Reference

```bash
cargo build              # Build all crates
cargo test               # Run all tests
cargo clippy             # Lint
cargo fmt --check        # Check formatting
cargo fmt                # Fix formatting
cargo run -- <args>      # Run CLI (e.g., cargo run -- list)
```

## Documentation

**Read `docs/index.md` first.** It has a source-file-aware map of all docs — find the right doc by which files you're touching.

Key docs:
- `docs/PATTERNS.md` — Coding conventions (error handling, Op trait, output, actors)
- `docs/SUCCESS_CRITERIA.md` — CI gate commands

## Inline doc-comment tripwires

Some conventions are documented as `///` doc comments on the canonical implementation (e.g., "this is the single underlying query — do not add a separate one"). Before adding a new query, client method, helper, or abstraction, run the `rustdoc` skill to check whether such a tripwire already exists.

If you're writing a canonical-pattern doc comment yourself, follow the authoring convention in the `rustdoc` skill so future searches find it.
