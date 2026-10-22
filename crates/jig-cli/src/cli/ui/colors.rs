use std::fmt::Display;

use colored::Colorize;

use super::is_plain;

/// Highlight a name/value (cyan). Takes anything printable, so callers pass
/// the value rather than rendering it first.
pub fn highlight(value: impl Display) -> String {
    let s = value.to_string();
    if is_plain() {
        s
    } else {
        s.cyan().to_string()
    }
}

/// Bold text.
pub fn bold(s: &str) -> String {
    if is_plain() {
        s.to_string()
    } else {
        s.bold().to_string()
    }
}

/// Yellow text for warnings (inline, no prefix).
pub fn warn_text(s: &str) -> String {
    if is_plain() {
        s.to_string()
    } else {
        s.yellow().to_string()
    }
}

/// Dimmed text for secondary info.
pub fn dim(s: &str) -> String {
    if is_plain() {
        s.to_string()
    } else {
        s.dimmed().to_string()
    }
}

/// Source attribution text (green).
pub fn source(s: &str) -> String {
    if is_plain() {
        s.to_string()
    } else {
        s.green().to_string()
    }
}
