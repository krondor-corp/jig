//! Which view the page shows, kept in the URL's hash so a reload or a
//! bookmark lands in the same place: `#/` is the overview, `#/m/<name>` a
//! machine's workers and `#/m/<name>/issues` its issues. The back button
//! works because navigation is plain links and a `hashchange` listener.

/// What the main pane shows.
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    /// What needs a human, across every machine.
    Overview,
    /// One machine, by its fleet name, on one of its tabs.
    Machine(String, Tab),
}

/// A machine page's tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Workers,
    Issues,
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
        let Some(rest) = hash.strip_prefix("#/m/") else {
            return View::Overview;
        };
        // Names are encoded, so the only raw `/` is the tab's.
        let (name, tab) = match rest.split_once('/') {
            Some((name, "issues")) => (name, Tab::Issues),
            Some((name, _)) => (name, Tab::Workers),
            None => (rest, Tab::Workers),
        };
        if name.is_empty() {
            View::Overview
        } else {
            View::Machine(decode(name), tab)
        }
    }

    /// The link to this view.
    pub fn href(&self) -> String {
        match self {
            View::Overview => "#/".into(),
            View::Machine(name, Tab::Workers) => format!("#/m/{}", encode(name)),
            View::Machine(name, Tab::Issues) => format!("#/m/{}/issues", encode(name)),
        }
    }

    /// A machine's workers.
    pub fn machine(name: &str) -> Self {
        View::Machine(name.to_string(), Tab::Workers)
    }

    /// The machine this view is about, if any.
    pub fn machine_name(&self) -> Option<&str> {
        match self {
            View::Overview => None,
            View::Machine(name, _) => Some(name),
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
