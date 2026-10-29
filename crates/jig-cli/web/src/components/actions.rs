//! A worker's command buttons, and the question each asks first.

use yew::prelude::*;

use jig_api::{KillRequest, ModelRequest, NudgeRequest, PruneRequest, RetrySetupRequest};

use crate::api::Action;

#[derive(Properties, PartialEq)]
pub struct ActionsProps {
    /// `repo/branch`.
    pub worker: AttrValue,
    /// Which verbs to offer, by [`Action::name`].
    pub verbs: Vec<&'static str>,
    pub on_command: Callback<Action>,
    /// A command is in flight: hold off.
    pub busy: bool,
}

/// The verbs every row offers; `retry-setup` joins them when setup failed.
pub fn verbs_for(failed_setup: bool) -> Vec<&'static str> {
    let mut verbs = vec!["nudge", "model", "kill", "prune"];
    if failed_setup {
        verbs.push("retry-setup");
    }
    verbs
}

#[function_component(Actions)]
pub fn actions(props: &ActionsProps) -> Html {
    props
        .verbs
        .iter()
        .map(|&verb| {
            let worker = props.worker.clone();
            let on_command = props.on_command.clone();
            let onclick = Callback::from(move |_| {
                if let Some(action) = ask(verb, &worker) {
                    on_command.emit(action);
                }
            });
            html! {
                <button type="button" disabled={props.busy} {onclick}>{ verb }</button>
            }
        })
        .collect()
}

/// The request `verb` sends, after asking for its argument or a
/// confirmation. `None` when the user backs out.
fn ask(verb: &str, worker: &str) -> Option<Action> {
    let window = web_sys::window()?;
    let key = worker.to_string();
    let prompt = |message: String| {
        window
            .prompt_with_message(&message)
            .ok()
            .flatten()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let confirm = |message: String| window.confirm_with_message(&message).unwrap_or(false);
    match verb {
        "nudge" => prompt(format!("Nudge {worker} with:"))
            .map(|text| Action::Nudge(NudgeRequest { worker: key, text })),
        "model" => prompt(format!("Switch {worker} to model:"))
            .map(|model| Action::Model(ModelRequest { worker: key, model })),
        "kill" => confirm(format!(
            "Kill {worker}? Its window closes; the worktree stays."
        ))
        .then_some(Action::Kill(KillRequest { worker: key })),
        "prune" => confirm(format!(
            "Prune {worker}? Its worktree is removed, uncommitted changes included."
        ))
        .then_some(Action::Prune(PruneRequest { worker: key })),
        "retry-setup" => Some(Action::RetrySetup(RetrySetupRequest { worker: key })),
        _ => None,
    }
}
