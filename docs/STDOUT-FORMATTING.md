# CLI Output Formatting

## The Op Pattern

Every CLI command implements the `Op` trait, cleanly separating context-building, command logic, and presentation:

```rust
pub trait Op {
    type Context;
    type Error: std::error::Error + Send + Sync + 'static;
    type Output: std::fmt::Display;

    fn build_context(&self) -> Result<Self::Context, Self::Error>;
    fn run(&self, ctx: Self::Context) -> Result<Self::Output, Self::Error>;
}
```

**Ops never:** print, use color, or check terminal width.
**Display impls own:** all color/styling, table layout, human-readable formatting.
**The boundary (main.rs) owns:** arg parsing, calling build_context + run, printing, exit codes.

## Adding a New Command

1. **Command struct** — clap parses args into this (may be a unit struct for no-arg commands)
2. **Error enum** — one variant per failure mode, derives `thiserror::Error`
3. **Output struct** — typed data, no formatting
4. **`Op` impl** — declares `type Context`, builds it in `build_context`, does work in `run`
5. **`Display` impl** — all presentation logic lives here

Choose the right context type:
- `()` — no context needed (pure commands: `version`, `which`, `shell-init`)
- `RepoCtx` — single repo from cwd (most commands)
- `ScopedCtx` — commands with `--global` flag (`list`, `ps`, `kill`, `nuke`, `remove`)

Example skeleton:

```rust
pub struct MyCmd;

#[derive(Debug, thiserror::Error)]
pub enum MyCmdError {
    #[error(transparent)]
    Context(#[from] crate::context::ContextError),
    #[error("something failed: {0}")]
    SomeFailure(#[from] jig_core::Error),
}

pub struct MyCmdOutput { /* typed fields */ }

impl Op for MyCmd {
    type Context = RepoCtx;
    type Error = MyCmdError;
    type Output = MyCmdOutput;

    fn build_context(&self) -> Result<RepoCtx, MyCmdError> {
        Ok(RepoCtx::from_cwd()?)
    }

    fn run(&self, ctx: RepoCtx) -> Result<Self::Output, Self::Error> {
        // do work using ctx.repo and ctx.config
    }
}

impl fmt::Display for MyCmdOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // format output using comfy-table, colored, etc.
    }
}
```

In `main.rs`, the dispatch is thin:

```rust
Some(ref command) => {
    let ctx = command.build_context()?;
    let output = command.run(ctx)?;
    // ...
}
```

## Table Formatting with comfy-table

Use `comfy-table` for tabular output. It auto-detects terminal width and adjusts column sizes.

```rust
use comfy_table::{presets, Attribute, Cell, Color, ContentArrangement, Table};

let mut table = Table::new();
table
    .load_preset(presets::NOTHING)           // no borders
    .set_content_arrangement(ContentArrangement::Dynamic)  // terminal-width-aware
    .set_header(vec![
        Cell::new("COLUMN").add_attribute(Attribute::Bold),
    ]);

table.add_row(vec![Cell::new("value").fg(Color::Cyan)]);
```

Key choices:
- `presets::NOTHING` — clean output with no box-drawing characters
- `ContentArrangement::Dynamic` — columns adapt to terminal width
- Headers are **bold**, data uses semantic colors

## Color Conventions

| Element | Color |
|---------|-------|
| Names/identifiers | Cyan |
| Running/active status | Green |
| Warning/exited status | Yellow |
| Error/dead status | Red |
| Dimmed/inactive values | Dimmed |
| Headers | Bold (no color) |

## Rules

- Status messages go to **stderr** (`eprintln!`), machine-readable output to **stdout** (`println!`)
- Never output raw ANSI codes to stdout — it breaks shell integration
- Ops are pure logic; they never import `colored` or `comfy_table`
- All presentation decisions live in `Display` impls on output types
