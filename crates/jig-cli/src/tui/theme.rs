//! The TUI's palette and pane chrome.
//!
//! Ported from octorus (ushironoko/octorus, MIT), `src/ui/theme.rs` and
//! `src/ui/chrome.rs`: six semantic inks, a focused/dim [`Attention`] that
//! decides a pane's border and title, and the row marker. jig adds `brand`
//! (jig.krondor.org's orange) and the mapping from `ps`'s colors, so a
//! worker row reads the same here as in `jig ps`.

use comfy_table::Color as PsColor;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders};

/// The marker before a selected row in the focused pane, and in a dim one.
pub const FOCUSED_MARKER: &str = "▌ ";
pub const DIM_MARKER: &str = "▏ ";

/// One semantic color, as a style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ink {
    style: Style,
}

impl Ink {
    pub fn style(self) -> Style {
        self.style
    }

    pub fn bold(self) -> Style {
        self.style.add_modifier(Modifier::BOLD)
    }

    fn plain(color: Color) -> Self {
        Self {
            style: Style::default().fg(color),
        }
    }

    fn modifier(modifier: Modifier) -> Self {
        Self {
            style: Style::default().add_modifier(modifier),
        }
    }
}

impl From<Ink> for Style {
    fn from(ink: Ink) -> Self {
        ink.style
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub muted: Ink,
    pub accent: Ink,
    pub warning: Ink,
    pub danger: Ink,
    pub success: Ink,
    pub info: Ink,
    /// jig's orange: the wordmark and the focused pane.
    pub brand: Ink,
    /// Draw `ps` colors, or leave cells plain (`--plain`, `NO_COLOR`).
    color: bool,
}

impl Theme {
    /// The dark palette, or attributes only when `no_color`.
    pub fn new(no_color: bool) -> Self {
        if no_color {
            return Self {
                muted: Ink::modifier(Modifier::DIM),
                accent: Ink::modifier(Modifier::BOLD),
                warning: Ink::modifier(Modifier::empty()),
                danger: Ink::modifier(Modifier::BOLD),
                success: Ink::modifier(Modifier::empty()),
                info: Ink::modifier(Modifier::empty()),
                brand: Ink::modifier(Modifier::BOLD),
                color: false,
            };
        }
        Self {
            muted: Ink::plain(Color::DarkGray),
            accent: Ink::plain(Color::Cyan),
            warning: Ink::plain(Color::Yellow),
            danger: Ink::plain(Color::Red),
            success: Ink::plain(Color::Green),
            info: Ink::plain(Color::Cyan),
            brand: Ink::plain(Color::Rgb(254, 128, 25)),
            color: true,
        }
    }

    /// A `jig ps` cell color, as the TUI draws it.
    pub fn ps(self, color: PsColor) -> Style {
        if !self.color {
            return Style::default();
        }
        let color = match color {
            PsColor::Green | PsColor::DarkGreen => Color::Green,
            PsColor::Red | PsColor::DarkRed => Color::Red,
            PsColor::Yellow | PsColor::DarkYellow => Color::Yellow,
            PsColor::Blue | PsColor::DarkBlue => Color::Blue,
            PsColor::Cyan | PsColor::DarkCyan => Color::Cyan,
            PsColor::Magenta | PsColor::DarkMagenta => Color::Magenta,
            PsColor::DarkGrey | PsColor::Grey => Color::DarkGray,
            _ => return Style::default(),
        };
        Style::default().fg(color)
    }
}

/// Whether a pane has the keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attention {
    Focused,
    Dim,
}

impl Attention {
    pub fn when(focused: bool) -> Self {
        if focused {
            Self::Focused
        } else {
            Self::Dim
        }
    }

    /// The marker before the selected row.
    pub fn marker(self) -> &'static str {
        match self {
            Self::Focused => FOCUSED_MARKER,
            Self::Dim => DIM_MARKER,
        }
    }

    /// The selected row's style.
    pub fn selected(self, theme: Theme) -> Style {
        match self {
            Self::Focused => theme.brand.bold(),
            Self::Dim => Style::default().add_modifier(Modifier::BOLD),
        }
    }

    fn frame(self, theme: Theme) -> Block<'static> {
        let (border_type, style) = match self {
            Self::Focused => (BorderType::Thick, theme.brand.style()),
            Self::Dim => (BorderType::Plain, theme.muted.style()),
        };
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(style)
    }
}

/// A pane's bordered block: a thick brand border and `▶ LABEL detail` when
/// it has the keys, a plain dim one and `label detail` when not.
pub fn pane_block(theme: Theme, label: &str, detail: &str, attention: Attention) -> Block<'static> {
    let (text, style) = match attention {
        Attention::Focused => (
            format!(" ▶ {} {detail} ", label.to_uppercase()),
            theme.brand.bold(),
        ),
        Attention::Dim => (format!(" {label} {detail} "), theme.muted.style()),
    };
    attention
        .frame(theme)
        .title(Line::from(Span::styled(text, style)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn block_text(block: Block<'static>) -> String {
        let mut terminal = Terminal::new(TestBackend::new(40, 3)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(block, frame.area()))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect()
    }

    #[test]
    fn a_focused_pane_says_so_in_its_title() {
        let text = block_text(pane_block(
            Theme::new(false),
            "Workers",
            "(3)",
            Attention::Focused,
        ));
        assert!(text.contains("▶ WORKERS (3)"), "{text}");
    }

    #[test]
    fn a_dim_pane_keeps_its_case_and_has_no_arrow() {
        let text = block_text(pane_block(
            Theme::new(false),
            "Workers",
            "(3)",
            Attention::Dim,
        ));
        assert!(text.contains("Workers (3)"), "{text}");
        assert!(!text.contains('▶'), "{text}");
    }

    #[test]
    fn without_color_ps_cells_are_plain() {
        assert_eq!(Theme::new(true).ps(PsColor::Red), Style::default());
        assert_eq!(
            Theme::new(false).ps(PsColor::Red),
            Style::default().fg(Color::Red)
        );
    }
}
