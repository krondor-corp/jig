//! Shared rendering utilities for CLI output.
//!
//! Provides consistent status symbols, color helpers, truncation, table builders,
//! and a global plain-mode flag for scriptable output.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use comfy_table::{presets, Attribute, Cell, ContentArrangement, Table};
use crossterm::terminal;

pub mod colors;
pub mod output;

pub use colors::*;
pub use output::*;

// ---------------------------------------------------------------------------
// Plain mode
// ---------------------------------------------------------------------------

static PLAIN_MODE: AtomicBool = AtomicBool::new(false);

/// Enable or disable plain (no-color, no-decoration) output.
pub fn set_plain(enabled: bool) {
    PLAIN_MODE.store(enabled, Ordering::Relaxed);
}

/// Returns true when `--plain` was passed.
pub fn is_plain() -> bool {
    PLAIN_MODE.load(Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// Table helpers
// ---------------------------------------------------------------------------

/// Create a new table with the standard preset (no borders) and dynamic arrangement.
pub fn new_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(presets::NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(
            headers
                .iter()
                .map(|h| Cell::new(*h).add_attribute(Attribute::Bold))
                .collect::<Vec<_>>(),
        );
    table
}

// ---------------------------------------------------------------------------
// Truncation
// ---------------------------------------------------------------------------

/// Truncate a string to `max` characters, appending ellipsis if needed (UTF-8 safe).
pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .nth(max - 1)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}…", &s[..end])
    }
}

// ---------------------------------------------------------------------------
// Duration formatting
// ---------------------------------------------------------------------------

/// Format seconds as a short human-readable duration: `45s`, `3m12s`, `1h5m`.
pub fn format_duration_short(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        let m = secs / 60;
        let s = secs % 60;
        if s == 0 {
            format!("{}m", m)
        } else {
            format!("{}m{}s", m, s)
        }
    } else {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        if m == 0 {
            format!("{}h", h)
        } else {
            format!("{}h{}m", h, m)
        }
    }
}

// ---------------------------------------------------------------------------
// Alternate screen
// ---------------------------------------------------------------------------

/// Run a closure in the alternate screen with raw mode enabled.
///
/// Enters the alternate screen buffer (like `less` or `git diff`), enables
/// raw mode for keypress handling, then runs `f`. On return (or error),
/// raw mode and the alternate screen are always restored.
pub fn with_alternate_screen<F, T, E>(f: F) -> Result<T, E>
where
    F: FnOnce(&mut io::Stderr) -> Result<T, E>,
    E: From<io::Error>,
{
    let mut w = io::stderr();
    crossterm::execute!(w, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let result = f(&mut w);

    let _ = terminal::disable_raw_mode();
    let _ = crossterm::execute!(w, terminal::LeaveAlternateScreen);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_short_seconds() {
        assert_eq!(format_duration_short(0), "0s");
        assert_eq!(format_duration_short(1), "1s");
        assert_eq!(format_duration_short(45), "45s");
        assert_eq!(format_duration_short(59), "59s");
    }

    #[test]
    fn format_duration_short_minutes() {
        assert_eq!(format_duration_short(60), "1m");
        assert_eq!(format_duration_short(61), "1m1s");
        assert_eq!(format_duration_short(192), "3m12s");
        assert_eq!(format_duration_short(300), "5m");
        assert_eq!(format_duration_short(3599), "59m59s");
    }

    #[test]
    fn format_duration_short_hours() {
        assert_eq!(format_duration_short(3600), "1h");
        assert_eq!(format_duration_short(3660), "1h1m");
        assert_eq!(format_duration_short(7260), "2h1m");
        assert_eq!(format_duration_short(7200), "2h");
    }
}
