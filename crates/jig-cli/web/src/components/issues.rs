//! A machine's Issues tab: its repos' planned and backlog issues, from that
//! machine's own trackers, with a button to spawn each one there.
//!
//! The list is the machine's cached fetch, so polling it is cheap; Refresh
//! asks the machine to fetch again. Spawning goes out through `on_command`
//! like any verb, already addressed to this machine.

use gloo_timers::callback::Interval;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use jig_api::{Issue, Issues, RefreshIssuesRequest, RepoIssues, SpawnRequest};

use crate::api::{self, Action};

/// How often the list is re-read from the machine's cache.
const REFRESH_MS: u32 = 5_000;

#[derive(Properties, PartialEq)]
pub struct IssuesTabProps {
    /// The machine's node id; `None` for the one serving the page.
    pub machine: Option<String>,
    /// Its fleet name, for the spawn confirmation.
    pub name: AttrValue,
    /// Addressed to this machine already.
    pub on_command: Callback<Action>,
    pub busy: bool,
}

#[function_component(IssuesTab)]
pub fn issues_tab(props: &IssuesTabProps) -> Html {
    let list = use_state(|| None::<Result<Issues, String>>);
    {
        let list = list.clone();
        use_effect_with(props.machine.clone(), move |machine| {
            let fetch = {
                let machine = machine.clone();
                move || {
                    let list = list.clone();
                    let machine = machine.clone();
                    spawn_local(async move { list.set(Some(api::issues(machine).await)) });
                }
            };
            fetch();
            let interval = Interval::new(REFRESH_MS, fetch);
            move || drop(interval)
        });
    }

    let refresh = {
        let on_command = props.on_command.clone();
        Callback::from(move |_| {
            on_command.emit(Action::RefreshIssues(RefreshIssuesRequest::default()))
        })
    };

    let body = match &*list {
        None => html! { <p class="empty">{ "Loading issues…" }</p> },
        Some(Err(e)) => html! { <p class="c-red">{ format!("Can't list issues: {e}") }</p> },
        Some(Ok(issues)) if issues.repos.is_empty() => html! {
            <p class="empty">{ "No repo here has an issue tracker ([issues.linear] in its jig.toml)." }</p>
        },
        Some(Ok(issues)) => html! {
            <>{ for issues.repos.iter().map(|repo| repo_issues(repo, props)) }</>
        },
    };
    let fetched = match &*list {
        Some(Ok(Issues {
            fetched: Some(ago), ..
        })) => format!("fetched {ago} ago"),
        Some(Ok(_)) => "not fetched yet".to_string(),
        _ => String::new(),
    };
    html! {
        <section class="issues">
            <p class="summary">
                { format!("planned and backlog · {fetched} ") }
                <button type="button" disabled={props.busy} onclick={refresh}>{ "refresh" }</button>
            </p>
            { body }
        </section>
    }
}

fn repo_issues(repo: &RepoIssues, props: &IssuesTabProps) -> Html {
    html! {
        <section class="repo" key={repo.repo.clone()}>
            <h2>{ &repo.repo }</h2>
            if let Some(e) = &repo.error {
                <p class="c-red">{ e }</p>
            } else if repo.issues.is_empty() {
                <p class="empty">{ "Nothing planned or in the backlog." }</p>
            } else {
                <div class="table-wrap"><table>
                    <thead><tr>
                        <th>{ "ISSUE" }</th><th>{ "TITLE" }</th><th>{ "STATUS" }</th>
                        <th>{ "PRIORITY" }</th><th>{ "LABELS" }</th><th></th>
                    </tr></thead>
                    <tbody>
                        { for repo.issues.iter().map(|issue| row(&repo.repo, issue, props)) }
                    </tbody>
                </table></div>
            }
        </section>
    }
}

fn row(repo: &str, issue: &Issue, props: &IssuesTabProps) -> Html {
    let status_class = if issue.status == "planned" {
        "c-cyan"
    } else {
        "c-dim"
    };
    let priority_class = match issue.priority.as_str() {
        "urgent" => "c-red",
        "high" => "c-yellow",
        _ => "c-dim",
    };
    let action = if issue.spawned {
        html! { <span class="c-green">{ "spawned" }</span> }
    } else if issue.parent {
        html! { <span class="c-dim" title="spawn its sub-issues instead">{ "has sub-issues" }</span> }
    } else {
        let spawn = SpawnRequest {
            repo: repo.to_string(),
            issue: issue.id.clone(),
            machine: None,
        };
        let question = format!(
            "Spawn {} on {}? This starts an agent there, which uses real tokens.",
            issue.id, props.name
        );
        let on_command = props.on_command.clone();
        let onclick = Callback::from(move |_| {
            let confirmed = web_sys::window()
                .and_then(|w| w.confirm_with_message(&question).ok())
                .unwrap_or(false);
            if confirmed {
                on_command.emit(Action::Spawn(spawn.clone()));
            }
        });
        html! { <button type="button" disabled={props.busy} {onclick}>{ "spawn" }</button> }
    };
    html! {
        <tr key={issue.id.clone()}>
            <td class="c-cyan" title={issue.branch.clone()}>{ &issue.id }</td>
            <td class="issue-title">{ &issue.title }</td>
            <td class={status_class}>{ &issue.status }</td>
            <td class={priority_class}>{ &issue.priority }</td>
            <td class="c-dim">
                { issue.labels.join(", ") }
                if issue.auto {
                    <span class="tag" title="the daemon spawns it on its own once planned">{ "auto" }</span>
                }
            </td>
            <td class="actions">{ action }</td>
        </tr>
    }
}
