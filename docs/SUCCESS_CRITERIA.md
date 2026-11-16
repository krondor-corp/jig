# Success Criteria

Checks that must pass before code can be merged. This is the CI gate.

## Quick Check

Run all checks in sequence:
```bash
cargo build && cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

## Individual Checks

### Build

```bash
cargo build
```

Build must complete without errors.

### Tests

```bash
cargo test --workspace
```

All tests must pass. Integration tests live under `crates/*/tests/` and build
isolated git repos in temp dirs — see the `Sandbox` fixture in
`crates/jig-cli/tests/common/mod.rs`, which owns temp `XDG_CONFIG_HOME` and
`XDG_RUNTIME_DIR` roots so tests never touch the developer's own state and can
run in parallel.

> **Put integration tests inside a crate, never at the workspace root.** The
> root `Cargo.toml` is a virtual manifest (`[workspace]` with no `[package]`),
> so a top-level `tests/` directory belongs to no package and cargo silently
> never compiles it. A suite was orphaned that way once and rotted unnoticed.
> `cargo test --workspace --no-run` lists every executable that actually
> builds — check there if a test file seems not to run.

### Linting

```bash
cargo clippy -- -D warnings
```

No clippy warnings allowed. Common issues:
- Unused variables or imports
- Unnecessary clones
- Missing error handling

### Formatting

Check formatting:
```bash
cargo fmt --check
```

Fix formatting:
```bash
cargo fmt
```

## Fixing Common Issues

### Formatting Failures

Run the formatter and commit:
```bash
cargo fmt
git add -p  # Review changes
git commit -m "style: format code"
```

### Lint Warnings

Fix the warning in the code, or if it's a false positive, add an allow attribute:
```rust
#[allow(clippy::lint_name)]
```

Only suppress lints with good reason.

### Test Failures

Run a specific test with output:
```bash
cargo test test_name -- --nocapture
```

Debug integration tests by examining the temp directory state.

## Pre-commit

jig installs git hooks (`post-commit`, `post-merge`, `pre-commit`) via `jig init`. These hooks emit events for the daemon — they do not run linters or formatters. Run checks manually before pushing.
