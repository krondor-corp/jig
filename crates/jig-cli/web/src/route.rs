//! Which view the page shows, kept in the URL's hash so a reload or a
//! bookmark lands in the same place: `#/` is the overview, `#/m/<name>` a
//! machine. The back button works because navigation is plain links and a
//! `hashchange` listener.

/// What the main pane shows.
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    /// What needs a human, across every machine.
    Overview,
    /// One machine, by its fleet name.
    Machine(String),
}

impl View {
    /// The view the URL names now.
    pub fn current() -> Self {
        let hash = web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .unwrap_or_default();
        Self::parse(&hash)
    }

    fn parse(hash: &str) -> Self {
        match hash.strip_prefix("#/m/") {
            Some(name) if !name.is_empty() => View::Machine(decode(name)),
            _ => View::Overview,
        }
    }

    /// The link to this view.
    pub fn href(&self) -> String {
        match self {
            View::Overview => "#/".into(),
            View::Machine(name) => format!("#/m/{}", encode(name)),
        }
    }
}

/// Machine names are hostnames or config strings: escaping `%` and `/` is
/// all a hash path needs.
fn encode(name: &str) -> String {
    name.replace('%', "%25").replace('/', "%2F")
}

fn decode(name: &str) -> String {
    name.replace("%2F", "/").replace("%25", "%")
}
