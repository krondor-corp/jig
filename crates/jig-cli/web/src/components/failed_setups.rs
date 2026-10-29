//! Workers whose setup failed, with the end of the output and the whole
//! `setup.log` a click away.

use yew::prelude::*;

use jig_api::{FailedSetup, Status};

use super::Actions;
use crate::api::{self, Action};

#[derive(Properties, PartialEq)]
pub struct FailedSetupsProps {
    pub status: Status,
    pub on_command: Callback<Action>,
    pub busy: bool,
}

#[function_component(FailedSetups)]
pub fn failed_setups(props: &FailedSetupsProps) -> Html {
    let failed: Vec<_> = props
        .status
        .workers()
        .filter_map(|w| w.failed_setup.as_ref().map(|f| (w.key.clone(), f.clone())))
        .collect();
    if failed.is_empty() {
        return html! {};
    }
    html! {
        <section class="failed">
            <h2>{ "Failed setups" }</h2>
            { for failed.into_iter().map(|(key, setup)| html! {
                <FailedWorker
                    worker={key.clone()}
                    key={key}
                    {setup}
                    on_command={props.on_command.clone()}
                    busy={props.busy}
                />
            }) }
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct FailedWorkerProps {
    worker: AttrValue,
    setup: FailedSetup,
    on_command: Callback<Action>,
    busy: bool,
}

/// One failed worker. Holds its own full log once fetched: the panel
/// re-renders on every refresh, and the log shouldn't collapse with it.
#[function_component(FailedWorker)]
fn failed_worker(props: &FailedWorkerProps) -> Html {
    // `None` until asked for; then the log, or why it couldn't be read.
    let log = use_state(|| None::<Result<String, String>>);
    let show_log = {
        let log = log.clone();
        let worker = props.worker.to_string();
        Callback::from(move |_| {
            let log = log.clone();
            let worker = worker.clone();
            wasm_bindgen_futures::spawn_local(async move {
                log.set(Some(api::setup_log(&worker).await));
            });
        })
    };
    let setup = &props.setup;
    html! {
        <article>
            <h3>{ &props.worker }{ " " }<span class="c-red">{ &setup.state }</span></h3>
            {
                match &*log {
                    Some(Ok(text)) => html! { <pre>{ text }</pre> },
                    Some(Err(e)) => html! {
                        <>
                            if !setup.tail.is_empty() { <pre>{ setup.tail.join("\n") }</pre> }
                            <p class="c-red">{ e }</p>
                        </>
                    },
                    None if !setup.tail.is_empty() => html! { <pre>{ setup.tail.join("\n") }</pre> },
                    None => html! {},
                }
            }
            <p>
                if setup.has_log && log.is_none() {
                    <button type="button" onclick={show_log}>{ "full setup.log" }</button>
                }
                <Actions
                    worker={props.worker.clone()}
                    verbs={vec!["retry-setup", "prune"]}
                    on_command={props.on_command.clone()}
                    busy={props.busy}
                />
            </p>
        </article>
    }
}
