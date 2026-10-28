//! The work view's pieces. Each renders a slice of [`jig_api::Status`];
//! none of them fetch.

mod actions;
mod failed_setups;
mod repo_table;
mod site_header;
mod triages;

pub use actions::{verbs_for, Actions};
pub use failed_setups::FailedSetups;
pub use repo_table::RepoTable;
pub use site_header::SiteHeader;
pub use triages::Triages;

use jig_api::Tone;

/// The stylesheet class for a `ps` color.
pub fn tone_class(tone: Tone) -> &'static str {
    match tone {
        Tone::Plain => "",
        Tone::Green => "c-green",
        Tone::Red => "c-red",
        Tone::Yellow => "c-yellow",
        Tone::Blue => "c-blue",
        Tone::Cyan => "c-cyan",
        Tone::Magenta => "c-magenta",
        Tone::Dim => "c-dim",
    }
}
