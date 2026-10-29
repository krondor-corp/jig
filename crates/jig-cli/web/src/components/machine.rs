//! One machine's work view: its heading, then everything `jig ps` shows for
//! it. Verbs from here go to this machine, wherever the page was served.

use yew::prelude::*;

use jig_api::{Machine, Status};

use super::{FailedSetups, RepoTable, Triages};
use crate::api::Action;

#[derive(Properties, PartialEq)]
pub struct MachineViewProps {
    pub machine: Machine,
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
    html! {
        <section class="machine">
            <h2 class="machine-name">
                { &machine.name }
                <span class="c-dim">{ format!(" · {whose}") }</span>
            </h2>
            {
                match (&machine.status, &machine.error) {
                    (Some(status), _) => view(status, machine.is_local(), &on_command, props.busy),
                    (None, Some(e)) => html! { <p class="c-red">{ format!("Not answering: {e}") }</p> },
                    (None, None) => html! { <p class="empty">{ "No answer" }</p> },
                }
            }
        </section>
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
