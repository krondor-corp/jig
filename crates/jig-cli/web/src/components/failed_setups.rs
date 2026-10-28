//! Workers whose setup failed, with the end of the output and the whole
//! `setup.log` a click away.

use yew::prelude::*;

use jig_api::{CommandRequest, Status};

use super::Actions;
use crate::api;

#[derive(Properties, PartialEq)]
pub struct FailedSetupsProps {
    pub status: Status,
    pub on_command: Callback<CommandRequest>,
    pub busy: bool,
}

#[function_component(FailedSetups)]
pub fn failed_setups(props: &FailedSetupsProps) -> Html {
    let failed: Vec<_> = props
        .status
        .workers()
        .filter_map(|w| w.failed_setup.as_ref().map(|f| (w, f)))
        .collect();
    if failed.is_empty() {
        return html! {};
    }
    html! {
        <section class="failed">
            <h2>{ "Failed setups" }</h2>
            { for failed.into_iter().map(|(w, f)| html! {
                <article key={w.key.clone()}>
                    <h3>{ &w.key }{ " " }<span class="c-red">{ &f.state }</span></h3>
                    if !f.tail.is_empty() {
                        <pre>{ f.tail.join("\n") }</pre>
                    }
                    <p>
                        if f.has_log {
                            <a href={api::setup_log_url(&w.key)} target="_blank">{ "setup.log" }</a>
                            { " " }
                        }
                        <Actions
                            worker={w.key.clone()}
                            verbs={vec!["retry-setup", "prune"]}
                            on_command={props.on_command.clone()}
                            busy={props.busy}
                        />
                    </p>
                </article>
            }) }
        </section>
    }
}
