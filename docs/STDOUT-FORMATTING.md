# CLI Output Formatting

## The Op Pattern

Every CLI command implements the `Op` trait, cleanly separating context-building, command logic, and presentation:

```rust
pub trait Op {
    type Context;
    type Error: std::error::Error + Send + Sync + 'static;
    type Output: std::fmt::Display;

    fn build_context(&self, app: AppCtx) -> Result<Self::Context, Self::Error>;
    fn run(&self, ctx: Self::Context) -> Result<Self::Output, Self::Error>;

    /// Where this invocation's tracing goes. Override only when stderr
    /// would be wrong — see `LogSink::File`.
    fn log_sink(&self) -> LogSink { LogSink::Stderr }
}
```

**Ops never write to stdout**, never use color directly, and never check
terminal width.
**Display impls own:** all color/styling, table layout, human-readable formatting.
**The boundary (main.rs) owns:** arg parsing, calling build_context + run,
printing the `Output`, exit codes.

### stdout is the contract; stderr is the conversation

The invariant that matters is **stdout purity**. The shell wrapper `eval`s
jig's stdout, so anything on it that is not the command's `Output` gets
executed. stdout therefore carries exactly one thing: the `Display` of the
`Output`, printed by `main`. Resolved values (`jig config base`, `jig home`,
`jig list --plain`) go there; tables, prose and progress do not.

stderr is not held to the same rule, and deliberately so. A long or
multi-step op — `spawn`, `update`, `init`, `hooks`, `daemon install` —
**may** stream progress from inside `run` using the `ui::` helpers
(`ui::progress`, `ui::success`, `ui::warning`, `ui::detail`, `ui::header`).
Deferring that to a `Display` impl would withhold every line until the work
finished, making those commands look hung. Such a command returns `NoOutput`
when it has no machine-readable result, which is a real answer, not a
placeholder.

Use the `ui::` helpers rather than bare `eprintln!` where one fits: they are
the layer that honors `--plain`, so an op never decides about color itself.

A command whose result is a *value* or a *table* should still build it into
the `Output` type and let `Display` render it — streaming is for progress,
not for results.

## Adding a New Command

1. **Command struct** — clap parses args into this (may be a unit struct for no-arg commands)
2. **Error enum** — one variant per failure mode, derives `thiserror::Error`
3. **Output struct** — typed data, no formatting
4. **`Op` impl** — declares `type Context`, builds it in `build_context`, does work in `run`
5. **`Display` impl** — all presentation logic lives here

Choose the right context type (see docs/PATTERNS.md for the full list):
- `()` — no context needed (pure commands: `version`, `which`, `shell-init`)
- `Ctx` — one repo (`Ctx::here`) or every tracked repo (`Ctx::everywhere`), and
  `Ctx::scoped(app, self.global)` for commands with `-g` (`list`, `ps`, `kill`,
  `nudge`, `prune`, `model`, `retry-setup`, `nuke`, `remove`)
- `AppPaths` — jig's files but no repo (`daemon`, `notify`, `config`)

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
    type Context = Ctx;
    type Error = MyCmdError;
    type Output = MyCmdOutput;

    fn build_context(&self, app: AppCtx) -> Result<Ctx, MyCmdError> {
        Ok(Ctx::here(app)?)
    }

    fn run(&self, ctx: Ctx) -> Result<Self::Output, Self::Error> {
        // do work using ctx.repo()? and ctx.config
    }
}

impl fmt::Display for MyCmdOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // format output using comfy-table, colored, etc.
    }
}
```

In `main.rs`, the dispatch is thin. The top-level `Command` enum uses
`type Context = AppCtx` and passes it down, so each variant builds its own
context from the same resolved `AppCtx`:

```rust
Some(ref command) => {
    let app = command.build_context(app)?;  // each variant builds its own ctx internally
    let output = command.run(app)?;
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
