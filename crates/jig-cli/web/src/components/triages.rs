//! Issues the daemon is triaging right now.

use yew::prelude::*;

use jig_api::Triage;

#[derive(Properties, PartialEq)]
pub struct TriagesProps {
    pub triages: Vec<Triage>,
}

#[function_component(Triages)]
pub fn triages(props: &TriagesProps) -> Html {
    if props.triages.is_empty() {
        return html! {};
    }
    html! {
        <section>
            <h2>{ "Triages" }</h2>
            <div class="table-wrap"><table>
                <thead>
                    <tr><th>{ "ISSUE" }</th><th>{ "MODEL" }</th><th>{ "ELAPSED" }</th><th>{ "REPO" }</th></tr>
                </thead>
                <tbody>
                    { for props.triages.iter().map(|t| html! {
                        <tr key={t.issue.clone()}>
                            <td class="c-cyan">{ &t.issue }</td>
                            <td>{ &t.model }</td>
                            <td class="right">{ &t.elapsed }</td>
                            <td class="c-dim">{ &t.repo }</td>
                        </tr>
                    }) }
                </tbody>
            </table></div>
        </section>
    }
}
