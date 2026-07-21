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

use cli::op::Op;
use cli::ui;
use cli::Cli;

fn main() {
    if let Err(e) = run() {
        ui::print_error(e.as_ref());
        std::process::exit(1);
    }
}

fn init_tracing(log_file: Option<std::path::PathBuf>) {
    use tracing_subscriber::prelude::*;

    let default_level = if log_file.is_some() { "info" } else { "warn" };
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level));

    let file_layer = log_file.and_then(|path| {
        std::fs::File::create(&path).ok().map(|file| {
            tracing_subscriber::fmt::layer()
                .with_writer(std::sync::Mutex::new(file))
                .with_ansi(false)
        })
    });

    // Only write to stderr when there's no log file — the watch mode
    // reads logs from the file via LogTailer, and stderr output corrupts
    // the table display.
    let stderr_layer = if file_layer.is_none() {
        Some(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
    } else {
        None
    };

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

    // Best-effort global directory setup
    let _ = context::ensure_global_dirs();

    // Every command gets a session log file
    let log_file = context::new_daemon_log_path().ok();
    init_tracing(log_file);

    match cli.command {
        None => {
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
        Some(ref command) => {
            command.build_context()?;
            let output = command.run(())?;
            let output_str = output.to_string();
            if !output_str.is_empty() {
                println!("{}", output_str);
            }
            Ok(())
        }
    }
}
