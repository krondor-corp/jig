//! The work view's pieces. Each renders a slice of [`jig_api::Fleet`];
//! none of them fetch.

mod actions;
mod failed_setups;
mod issues;
mod machine;
mod overview;
mod repo_table;
mod sidebar;
mod site_header;
pub mod toasts;
mod triages;

pub use actions::{verbs_for, Actions};
pub use failed_setups::FailedSetups;
pub use issues::IssuesTab;
pub use machine::MachineView;
pub use overview::Overview;
pub use repo_table::RepoTable;
pub use sidebar::Sidebar;
pub use site_header::SiteHeader;
pub use toasts::{ToastStack, Toasts};
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
