//! The machine list: the overview first, then every machine with whether
//! it is answering, its worker count, and how many of those need a human.

use yew::prelude::*;

use jig_api::{Fleet, Machine};

use crate::route::View;

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub fleet: Fleet,
    pub view: View,
}

#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let needing: usize = props
        .fleet
        .machines
        .iter()
        .map(|m| m.attention().count() + usize::from(!m.is_up()))
        .sum();
    html! {
        <nav class="sidebar" aria-label="Machines">
            { item(
                &View::Overview,
                &props.view,
                html! { <span class="sidebar-overview">{ "Overview" }</span> },
                needing,
            ) }
            <p class="sidebar-label">{ "Machines" }</p>
            { for props.fleet.machines.iter().map(|m| machine(m, &props.view)) }
        </nav>
    }
}

fn machine(m: &Machine, current: &View) -> Html {
    // Shape as well as color: gruvbox's green and yellow are close at this
    // size.
    let (glyph, dot, state) = match (m.is_up(), &m.status) {
        (true, _) => ("●", "c-green", "up".to_string()),
        (false, Some(_)) => (
            "○",
            "c-yellow",
            format!("stale {}", m.last_seen.as_deref().unwrap_or("")),
        ),
        (false, None) => ("✕", "c-red", "not answering".to_string()),
    };
    let workers = m.status.as_ref().map(|s| s.workers().count()).unwrap_or(0);
    let detail = if m.is_local() {
        format!("this machine · {workers} workers")
    } else {
        format!("{state} · {workers} workers")
    };
    let label = html! {
        <>
            <span class={classes!("dot", dot)} title={state}>{ glyph }</span>
            <span class="sidebar-name">{ &m.name }</span>
            <span class="sidebar-detail">{ detail }</span>
        </>
    };
    item(
        &View::machine(&m.name),
        current,
        label,
        m.attention().count(),
    )
}

/// A link to `view`, marked when it is `current`, with a count of what
/// needs a human there.
fn item(view: &View, current: &View, label: Html, needing: usize) -> Html {
    // A machine is current on any of its tabs.
    let here = view.machine_name() == current.machine_name();
    let class = classes!("sidebar-item", here.then_some("current"));
    html! {
        <a {class} href={view.href()}>
            { label }
            if needing > 0 {
                <span class="badge" title="need attention">{ needing }</span>
            }
        </a>
    }
}
