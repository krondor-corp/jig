//! One repo's workers: the `jig ps` table, plus links and buttons.

use yew::prelude::*;

use jig_api::{CommandRequest, Repo, Worker};

use super::{tone_class, verbs_for, Actions};

/// The `PR` column, which links to the PR when there is one.
const PR_COLUMN: usize = 4;

#[derive(Properties, PartialEq)]
pub struct RepoTableProps {
    pub repo: Repo,
    pub headers: Vec<String>,
    pub on_command: Callback<CommandRequest>,
    pub busy: bool,
}

#[function_component(RepoTable)]
pub fn repo_table(props: &RepoTableProps) -> Html {
    html! {
        <section class="repo">
            <h2>{ &props.repo.name }</h2>
            <div class="table-wrap"><table>
                <thead>
                    <tr>
                        { for props.headers.iter().map(|h| html! { <th>{ h }</th> }) }
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    { for props.repo.workers.iter().map(|w| row(w, &props.on_command, props.busy)) }
                </tbody>
            </table></div>
        </section>
    }
}

fn row(w: &Worker, on_command: &Callback<CommandRequest>, busy: bool) -> Html {
    let cells = w.cells.iter().enumerate().map(|(i, cell)| {
        let class = classes!(tone_class(cell.tone), cell.right.then_some("right"));
        let content = match i {
            // The name is truncated to fit a terminal; the tooltip has it
            // whole, with the agent's live state.
            0 => {
                let agent = w
                    .agent
                    .as_ref()
                    .map(|a| format!(" · agent {a}"))
                    .unwrap_or_default();
                html! { <span title={format!("{}{agent}", w.name)}>{ &cell.text }</span> }
            }
            PR_COLUMN => match &w.pr_url {
                Some(url) => html! {
                    <a href={url.clone()} target="_blank" rel="noopener noreferrer">{ &cell.text }</a>
                },
                None => html! { &cell.text },
            },
            _ => html! { &cell.text },
        };
        html! { <td {class}>{ content }</td> }
    });
    html! {
        <tr key={w.key.clone()}>
            { for cells }
            <td class="actions">
                <Actions
                    worker={w.key.clone()}
                    verbs={verbs_for(w.failed_setup.is_some())}
                    on_command={on_command.clone()}
                    {busy}
                />
            </td>
        </tr>
    }
}
