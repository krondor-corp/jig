//! What needs a human, on every machine: the page's landing view.
//!
//! The daemon decides what needs attention (`Worker::attention`, the same
//! rule everywhere); this lists it, machine by machine, with the buttons to
//! act on it there. A machine that isn't answering is listed too.

use yew::prelude::*;

use jig_api::{Fleet, Machine, Worker};

use super::{verbs_for, Actions};
use crate::api::Action;
use crate::route::View;

#[derive(Properties, PartialEq)]
pub struct OverviewProps {
    pub fleet: Fleet,
    pub on_command: Callback<Action>,
    pub busy: bool,
}

#[function_component(Overview)]
pub fn overview(props: &OverviewProps) -> Html {
    let machines = &props.fleet.machines;
    let workers: usize = machines
        .iter()
        .filter_map(|m| m.status.as_ref())
        .map(|s| s.workers().count())
        .sum();
    let rows: Vec<Html> = machines
        .iter()
        .flat_map(|m| {
            let down = (!m.is_up()).then(|| down_row(m));
            let flagged = m
                .attention()
                .map(|w| worker_row(m, w, &props.on_command, props.busy));
            down.into_iter().chain(flagged).collect::<Vec<_>>()
        })
        .collect();
    html! {
        <section class="overview">
            <h2 class="machine-name">{ "Needs attention" }</h2>
            <p class="summary">{ format!(
                "{} machines · {workers} workers · {} need you",
                machines.len(), rows.len(),
            ) }</p>
            if rows.is_empty() {
                <p class="empty">{ "Nothing needs you right now." }</p>
            } else {
                <div class="table-wrap"><table>
                    <thead><tr>
                        <th>{ "MACHINE" }</th><th>{ "WORKER" }</th><th>{ "WHY" }</th><th></th>
                    </tr></thead>
                    <tbody>{ for rows }</tbody>
                </table></div>
            }
        </section>
    }
}

fn machine_link(m: &Machine) -> Html {
    html! { <a href={View::Machine(m.name.clone()).href()}>{ &m.name }</a> }
}

fn down_row(m: &Machine) -> Html {
    let why = match &m.last_seen {
        Some(ago) => format!("not answering (last seen {ago} ago)"),
        None => "not answering".to_string(),
    };
    html! {
        <tr key={format!("down:{}", m.name)}>
            <td>{ machine_link(m) }</td>
            <td class="c-dim">{ "—" }</td>
            <td class="c-yellow" title={m.error.clone().unwrap_or_default()}>{ why }</td>
            <td></td>
        </tr>
    }
}

fn worker_row(m: &Machine, w: &Worker, on_command: &Callback<Action>, busy: bool) -> Html {
    // Verbs go to the machine that owns the worker.
    let node = m.node.clone();
    let on_command = on_command.reform(move |action: Action| action.on(node.clone()));
    html! {
        <tr key={format!("{}:{}", m.name, w.key)}>
            <td>{ machine_link(m) }</td>
            <td title={w.name.clone()}>{ &w.key }</td>
            <td class="c-red">{ w.attention.clone().unwrap_or_default() }</td>
            <td class="actions">
                <Actions
                    worker={w.key.clone()}
                    verbs={verbs_for(w.failed_setup.is_some())}
                    {on_command}
                    {busy}
                />
            </td>
        </tr>
    }
}
