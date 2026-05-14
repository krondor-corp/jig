use colored::Colorize;

use super::is_plain;

/// Success symbol (green check mark).
pub const SYM_OK: &str = "✓";
/// Progress / action-in-flight symbol.
pub const SYM_ARROW: &str = "→";
/// Failure symbol.
pub const SYM_FAIL: &str = "✗";
/// Warning symbol.
pub const SYM_WARN: &str = "!";

/// Print a success line to stderr: `✓ message`
pub fn success(msg: &str) {
    if is_plain() {
        eprintln!("{}", msg);
    } else {
        eprintln!("{} {}", SYM_OK.green(), msg);
    }
}

/// Print a progress line to stderr: `→ message`
pub fn progress(msg: &str) {
    if is_plain() {
        eprintln!("{}", msg);
    } else {
        eprintln!("{} {}", SYM_ARROW.cyan(), msg);
    }
}

/// Print a failure line to stderr: `✗ message`
#[allow(dead_code)]
pub fn failure(msg: &str) {
    if is_plain() {
        eprintln!("{}", msg);
    } else {
        eprintln!("{} {}", SYM_FAIL.red(), msg);
    }
}

/// Print a warning line to stderr: `! message`
pub fn warning(msg: &str) {
    if is_plain() {
        eprintln!("{}", msg);
    } else {
        eprintln!("{} {}", SYM_WARN.yellow(), msg);
    }
}

/// Print an indented detail line to stderr: `  → detail`
pub fn detail(msg: &str) {
    if is_plain() {
        eprintln!("  {}", msg);
    } else {
        eprintln!("  {} {}", SYM_ARROW.dimmed(), msg);
    }
}

/// Print a section header to stderr.
pub fn header(msg: &str) {
    if is_plain() {
        eprintln!("{}", msg);
    } else {
        eprintln!("{}", msg.bold());
    }
}

/// Print a formatted error chain to stderr.
pub fn print_error(e: &dyn std::error::Error) {
    if is_plain() {
        eprintln!("error: {e}");
    } else {
        eprintln!("{} {e}", "error:".red().bold());
    }
    let mut source = e.source();
    while let Some(cause) = source {
        if is_plain() {
            eprintln!("  caused by: {cause}");
        } else {
            eprintln!("  {} {cause}", "caused by:".yellow());
        }
        source = cause.source();
    }
}
