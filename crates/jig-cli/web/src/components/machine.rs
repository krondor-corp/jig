//! One machine's work view: its heading, then everything `jig ps` shows for
//! it. Verbs from here go to this machine, wherever the page was served.

use yew::prelude::*;

use jig_api::{Machine, Status};

use super::{FailedSetups, IssuesTab, RepoTable, Triages};
use crate::api::Action;
use crate::route::{Tab, View};

#[derive(Properties, PartialEq)]
pub struct MachineViewProps {
    pub machine: Machine,
    pub tab: Tab,
    pub on_command: Callback<Action>,
    pub busy: bool,
}

#[function_component(MachineView)]
pub fn machine_view(props: &MachineViewProps) -> Html {
    let machine = &props.machine;
    // Every verb from this section names this machine, so the daemon that
    // served the page forwards it to the owner.
    let on_command = {
        let node = machine.node.clone();
        props
            .on_command
            .reform(move |action: Action| action.on(node.clone()))
    };
    let whose = match &machine.node {
        None => "this machine".to_string(),
        Some(node) => node.clone(),
    };
    let stale = match (&machine.error, &machine.last_seen) {
        (Some(e), Some(ago)) => Some(format!(
            "Not answering: {e}. Showing what it said {ago} ago."
        )),
        (Some(e), None) => Some(format!("Not answering: {e}")),
        (None, _) => None,
    };
    html! {
        <section class={classes!("machine", stale.is_some().then_some("stale"))}>
            <h2 class="machine-name">
                { &machine.name }
                <span class="c-dim">{ format!(" · {whose}") }</span>
            </h2>
            if let Some(stale) = stale {
                <p class="c-yellow">{ stale }</p>
            }
            { tabs(&machine.name, props.tab) }
            {
                match props.tab {
                    Tab::Workers => match &machine.status {
                        Some(status) => view(status, machine.is_local(), &on_command, props.busy),
                        None => html! {},
                    },
                    Tab::Issues => html! {
                        <IssuesTab
                            machine={machine.node.clone()}
                            name={machine.name.clone()}
                            {on_command}
                            busy={props.busy}
                        />
                    },
                }
            }
        </section>
    }
}

fn tabs(name: &str, current: Tab) -> Html {
    let link = |tab: Tab, label: &str| {
        let class = classes!("tab", (tab == current).then_some("current"));
        html! { <a {class} href={View::Machine(name.to_string(), tab).href()}>{ label.to_string() }</a> }
    };
    html! {
        <nav class="tabs">
            { link(Tab::Workers, "Workers") }
            { link(Tab::Issues, "Issues") }
        </nav>
    }
}

fn view(status: &Status, local: bool, on_command: &Callback<Action>, busy: bool) -> Html {
    let workers = status.workers().count();
    html! {
        <>
            <p class="summary">{ format!(
                "{workers} workers · {} triages · next poll in {} · daemon pid {} · v{}",
                status.triages.len(), status.next_poll, status.pid, status.version,
            ) }</p>
            <FailedSetups status={status.clone()} {local} on_command={on_command.clone()} {busy} />
            if workers == 0 && status.triages.is_empty() {
                <p class="empty">{ "No spawned sessions" }</p>
            }
            { for status.repos.iter().map(|repo| html! {
                <RepoTable
                    key={repo.name.clone()}
                    repo={repo.clone()}
                    headers={status.headers.clone()}
                    on_command={on_command.clone()}
                    {busy}
                />
            }) }
            <Triages triages={status.triages.clone()} />
            if !status.spawning.is_empty() {
                <p class="spawning">
                    <span class="c-dim">{ "spawning: " }</span>
                    <span class="c-yellow">{ status.spawning.join(", ") }</span>
                </p>
            }
        </>
    }
}
