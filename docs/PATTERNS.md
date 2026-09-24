# Coding Patterns

Document your team's coding patterns and conventions here. This helps AI agents and contributors follow consistent practices.

## Error Handling

- **jig-core**: Use `thiserror` for typed errors with `#[derive(Error)]`
  - Define domain-specific errors in `crates/jig-core/src/error.rs`
  - Return `Result<T>` using the crate's custom `Result` type alias
  - Errors should have clear, user-facing messages

- **jig-cli**: Use the `Op` trait with typed errors per command
  - Each command has its own error enum wrapping core errors
  - Infallible commands use `std::convert::Infallible`
  - Main function catches errors and prints to stderr with color
  - Exit with code 1 on any error

```rust
// In jig-core (typed errors)
#[derive(Error, Debug)]
pub enum Error {
    #[error("Worktree '{0}' does not exist")]
    WorktreeNotFound(String),
}

// In jig-cli (Op trait with typed output and errors)
#[derive(Args, Debug, Clone)]
pub struct Create { /* args */ }

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error(transparent)]
    Core(#[from] jig_core::Error),
}

impl Op for Create {
    type Context = RepoCtx;
    type Error = CreateError;
    type Output = CreateOutput;

    fn build_context(&self, paths: &AppPaths) -> Result<RepoCtx, CreateError> {
        Ok(RepoCtx::from_cwd(paths)?)
    }

    fn run(&self, ctx: RepoCtx) -> Result<Self::Output, Self::Error> {
        // ...
    }
}
```

## Module Organization

- **Workspace structure**: Separate crates for different concerns
  - `jig-core` — Pure library: git, GitHub, issues, mux, agents, prompt
  - `jig-cli` — CLI binary with config, daemon, worker, hooks, notify

- **jig-core modules**: One submodule directory per domain
  - `git/` — Git operations via git2: `Repo`, `Worktree`, `Branch`, `WorktreeRef`
  - `mux/` — `Mux` trait + backends (`TmuxMux`, `HerdrMux`)
  - `issues/` — Issue provider trait + Linear implementation
  - `agents/` — Agent adapters (Claude Code)
  - `github/` — GitHub API client and queries
  - `prompt/` — Handlebars-based prompt rendering

- **jig-cli modules**:
  - `cli/` — CLI framework: `op.rs` (Op trait + command_enum! macro), `ui/` (rendering primitives: `colors.rs`, `output.rs`), `commands/` (one file/dir per command)
  - `config/` — Configuration loading and management
  - `worker/` — Worker state, lifecycle, events
  - `daemon/` — Background daemon with actor threads
  - `hooks/` — Git and agent hook management
  - `notify/` — Notification system

- **Commands**: One file per CLI command in `crates/jig-cli/src/cli/commands/`
  - Each command implements the `Op` trait from `crates/jig-cli/src/cli/op.rs`
  - Commands are registered via `command_enum!` macro in `cli/mod.rs`
  - Doc comments on Args struct become CLI help text (no duplication)

## Naming Conventions

- **Files/modules**: `snake_case.rs`
- **Types/structs**: `PascalCase`
- **Functions/methods**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **CLI command names**: kebab-case (e.g., `shell-init`, `shell-setup`)

## Output Conventions

- **stderr**: Status messages, progress, errors (with color)
  - Use shared helpers from `crates/jig-cli/src/cli/ui` instead of inline `colored` calls
  - `ui::success("msg")` — green ✓ prefix
  - `ui::progress("msg")` — cyan → prefix
  - `ui::warning("msg")` — yellow ! prefix
  - `ui::failure("msg")` — red ✗ prefix
  - `ui::detail("msg")` — indented → for sub-items
  - `ui::header("msg")` — bold section header
  - `ui::highlight("val")`, `ui::bold("val")`, `ui::dim("val")` — inline color helpers
  - All helpers respect `--plain` flag (no colors when enabled)

- **stdout**: Machine-readable output only
  - Shell commands that need to be eval'd (e.g., `cd '/path'`)
  - Data that might be piped to other tools
  - Never include ANSI color codes in stdout

- **Tracing**: `tracing::warn!` and friends go where `Op::log_sink` says — stderr (level `warn`) by default, so warnings from one-off commands are seen. Only commands that own the terminal or have none (`ps --watch`, `daemon start`) override it to `LogSink::File`. `RUST_LOG` overrides the level.

- **`--plain` flag**: Global flag for scriptable output
  - Disables all colors and decorations
  - Tables output as tab-separated values
  - Status symbols still appear but without color

```rust
// Status message (stderr) — use ui helpers
ui::success(&format!("Created worktree '{}'", ui::highlight(name)));

// Tables — use ui::new_table for consistent styling
let mut table = ui::new_table(&["NAME", "BRANCH", "COMMITS"]);

// Machine-readable output (stdout)
println!("cd '{}'", canonical.display());
```

## Testing Patterns

- **No process-wide state in tests**: never `std::env::set_var` or `set_current_dir` — tests run in parallel. Code under test takes an `AppPaths` instead of reading the environment.

- **Tests that touch the filesystem live in `tests/`**, not in a `#[cfg(test)]` module. Unit tests are for pure logic: reducers, parsing, wire round-trips, decisions. The exception is a test that can only reach a private helper — keep that one beside the code and say why.

- **Unit tests**: Inline in source files with `#[cfg(test)]` modules
  - Test pure functions and internal logic
  - Located at bottom of the file being tested
  - For anything touching jig's files, build paths from a fixture: `let fixture = Fixture::with_repos(2); let paths = AppPaths::from(&fixture);` (`jig_core::test_support`)

- **Integration tests**: In `tests/`, grouped by area — one directory per area with a `main.rs` that declares its modules, so each area is a single test binary:

```text
tests/
├── common/mod.rs          # Sandbox, pulled in per area with #[path]
├── cli/                   # driving the binary: attach, commit, notify, startup…
├── context/               # config, repo config, log tailer
├── daemon/                # ipc, pidfile, actors/prune
├── hooks/                 # install, uninstall, registry, handlers
└── notify/                # queue, notifier
```

  A bare `tests/*.rs` file is fine when an area has just one; subdirectories without a `main.rs` are not compiled.
  - `Sandbox` wraps a `Fixture` — its own `XDG_CONFIG_HOME`, `XDG_RUNTIME_DIR`, and any number of repos — and runs the binary against it with `assert_cmd`
  - `Sandbox::with_repos(n)`, `sandbox.jig()` / `sandbox.jig_in(sandbox.repo(1))`, `sandbox.start_daemon()`

```rust
#[test]
fn test_create_worktree() {
    let sandbox = Sandbox::with_repo();
    sandbox.jig()
        .args(["create", "test1"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Created worktree"));
}
```

## Actor Pattern (Daemon)

The daemon (`crates/jig-cli/src/daemon/`) uses background actor threads for blocking I/O. Each actor implements the `Actor` trait:

```rust
pub trait Actor: Default + Send + Sync + 'static {
    type Request: Send + 'static;
    type Response: Send + 'static;

    const NAME: &'static str;
    const QUEUE_SIZE: usize;

    fn handle(&self, req: Self::Request) -> Self::Response;
}
```

Actors are managed via `ActorHandle<A>`, which owns the channel pair, background thread, and pending state.

Key conventions:
- Actor owns its own resources (e.g., `GitHubClient`)
- Communication is non-blocking on the tick thread (`try_send`, `try_recv`)
- Drop requests on backpressure when appropriate (nudges are best-effort)
- Bounded channels prevent unbounded memory growth

## Common Idioms

- **Git operations**: Use `git::Repo` wrapper around `git2::Repository`
  - Instance methods for operations requiring repo context (branch, worktree, merge)
  - Associated functions for path-scoped operations (diff, status, commits ahead)
  - Errors propagate via `#[from] git2::Error` in the `Error` enum

- **Multiplexer abstraction**: `Mux` trait in `jig-core/src/mux/`, dyn-dispatched (`Box<dyn Mux>`, `&dyn Mux`)
  - Backends: `TmuxMux` (default) and `HerdrMux` (herdr — persistent PTYs, remote attach, live agent state)
  - Backend choice is a `MuxKind` (`tmux`/`herdr`) read from global config (`jig config mux`), `JIG_MUX` env overrides per-run
  - Construct via `mux::for_repo`/`for_repo_with_prefix`/`from_group_name` factories — never call `TmuxMux::new`/`HerdrMux::new` directly outside `mux/`
  - `attach`/`attach_window` are default-method compositions of `focus`/`focus_window` (point the backend at a target) and `connect` (bring a client in front of the user) — implement the primitives, not the composed methods, when adding a backend
  - `agent_state(&self, name) -> Option<AgentState>` lets a backend report live `idle/working/blocked/done` classification; default `None` (tmux has no such signal)

- **jig's own files**: `AppPaths` (`context/paths.rs`) is resolved from `XDG_CONFIG_HOME` / `XDG_RUNTIME_DIR` once, in `main`, and reaches commands through `Op::build_context(&self, paths)` and the context structs (`ctx.paths`). Anything that reads or writes jig state takes `&AppPaths` — don't read XDG variables anywhere else.

- **Path handling**: Use `PathBuf` for owned paths, `&Path` for references
  - Canonicalize paths before displaying to users
  - Use `to_string_lossy()` when converting to string for git commands

- **Config cascading**: Repo-specific > Global > Default
  - Check repo config first, fall back to global, then hardcoded default

- **Context types**: Each command declares `type Context` in its `Op` impl
  - `RepoCtx` — single repo from cwd (`RepoCtx::from_cwd(paths)`); fields: `paths`, `repo: RepoConfig`, `config: Config`, `jig_toml: JigToml`
  - `GlobalCtx` — all tracked repos (`GlobalCtx::load(paths)`); fields: `paths`, `repos`, `config`, `registry`
  - `ScopedCtx` — enum for commands with `--global`; use `ScopedCtx::from_global(paths, self.global)` in `build_context`
  - `AppPaths` — for commands that need jig's files but no repo (`daemon`, `notify`, `attach`)
  - `()` — no context needed (pure commands like `version`, `which`, `shell-init`)
  - Both `RepoCtx` and `GlobalCtx` convert to `Context` via `From` impls for daemon/legacy code

- **Agent adapters**: Use `Agent` struct for agent-specific behavior
  - Defined in `crates/jig-core/src/agents/`
  - Currently supports Claude Code, extensible for others
