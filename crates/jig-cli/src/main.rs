//! jig CLI - Git worktree manager for parallel Claude Code sessions

mod cli;
pub mod context;
pub mod daemon;
pub mod hooks;
pub mod notify;
pub mod prompts;
pub mod terminal;
pub mod worker;

use std::io::IsTerminal;

use clap::{CommandFactory, Parser};

use cli::op::{LogSink, Op};
use cli::ui;
use cli::Cli;

fn main() {
    if let Err(e) = run() {
        ui::print_error(e.as_ref());
        std::process::exit(1);
    }
}

/// Route tracing per the command's [`LogSink`]. `RUST_LOG` overrides the
/// level either way.
fn init_tracing(sink: LogSink, paths: &context::AppPaths) {
    use tracing_subscriber::prelude::*;

    let file = match sink {
        LogSink::Stderr => None,
        LogSink::File => {
            let path = paths.new_session_log();
            let file = std::fs::File::create(&path).ok();
            if file.is_some() {
                context::log::set_session_log(path);
            }
            file
        }
    };

    let default_level = if file.is_some() { "info" } else { "warn" };
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level));

    let file_layer = file.map(|file| {
        tracing_subscriber::fmt::layer()
            .with_writer(std::sync::Mutex::new(file))
            .with_ansi(false)
    });
    // Never both: a file sink exists because stderr is unusable (the watch
    // view's table) or unwatched (a daemon).
    let stderr_layer = file_layer
        .is_none()
        .then(|| tracing_subscriber::fmt::layer().with_writer(std::io::stderr));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer)
        .with(file_layer)
        .init();
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Set global plain mode before any output
    ui::set_plain(cli.plain);

    // The `colored` crate checks stdout for TTY detection, but all jig
    // output goes to stderr. Override colorization based on stderr instead.
    if !cli.plain && std::io::stderr().is_terminal() {
        colored::control::set_override(true);
    }

    // The one place jig reads XDG variables; everything below gets `paths`.
    let paths = context::AppPaths::from_env()?;

    // Best-effort global directory setup
    let _ = paths.ensure();

    let sink = cli
        .command
        .as_ref()
        .map_or(LogSink::Stderr, |c| c.log_sink());
    init_tracing(sink, &paths);

    match cli.command {
        None => {
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
        Some(ref command) => {
            let paths = command.build_context(&paths)?;
            let output = command.run(paths)?;
            let output_str = output.to_string();
            if !output_str.is_empty() {
                println!("{}", output_str);
            }
            Ok(())
        }
    }
}
