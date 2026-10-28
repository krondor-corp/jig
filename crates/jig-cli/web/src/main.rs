//! jig web view: a Yew (Rust/WASM) app served by the daemon.
//!
//! The daemon owns the data: every row, cell and color in
//! [`jig_api::Status`] is what `jig ps` would print. This app lays it out,
//! adds links and command buttons, and polls for a fresh frame.
//!
//! Structure: `api` (typed calls to `/api/v0`) · `components` (the view's
//! pieces). `main.rs` loads the session, polls, and runs commands.

mod api;
mod components;

use std::cell::Cell;
use std::rc::Rc;

use gloo_timers::callback::Interval;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use components::{FailedSetups, RepoTable, SiteHeader, Triages};
use jig_api::{CommandRequest, Status};

/// How often the view refreshes.
const REFRESH_MS: u32 = 5_000;

/// The last command's result, shown above the view.
#[derive(Clone, PartialEq)]
struct Banner {
    ok: bool,
    message: String,
}

#[function_component(App)]
fn app() -> Html {
    // `None` until the first frame arrives.
    let status = use_state(|| None::<Result<Status, String>>);
    let token = use_state(|| None::<String>);
    let banner = use_state(|| None::<Banner>);
    // Commands in flight. A refresh that lands mid-command would show the
    // old state, so refreshes wait. Shared with the interval, which only
    // sees the value from the first render.
    let pending: Rc<Cell<u32>> = use_memo((), |_| Cell::new(0));

    let refresh = {
        let status = status.clone();
        let pending = pending.clone();
        Callback::from(move |_: ()| {
            if pending.get() > 0 {
                return;
            }
            let status = status.clone();
            spawn_local(async move { status.set(Some(api::status().await)) });
        })
    };

    {
        let token = token.clone();
        let refresh = refresh.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match api::session().await {
                    Ok(session) => token.set(Some(session.token)),
                    Err(e) => log_error(&format!("no session: {e}")),
                }
            });
            refresh.emit(());
            let interval = Interval::new(REFRESH_MS, move || refresh.emit(()));
            move || drop(interval)
        });
    }

    let on_command = {
        let token = token.clone();
        let banner = banner.clone();
        let refresh = refresh.clone();
        let pending = pending.clone();
        Callback::from(move |request: CommandRequest| {
            let Some(token) = (*token).clone() else {
                banner.set(Some(Banner {
                    ok: false,
                    message: "no session with the daemon yet — reload the page".into(),
                }));
                return;
            };
            banner.set(Some(Banner {
                ok: true,
                message: format!("{} {} …", request.command.name(), request.worker),
            }));
            pending.set(pending.get() + 1);
            let banner = banner.clone();
            let refresh = refresh.clone();
            let pending = pending.clone();
            spawn_local(async move {
                let result = api::command(&token, &request).await;
                pending.set(pending.get() - 1);
                banner.set(Some(match result {
                    Ok(message) => Banner { ok: true, message },
                    Err(message) => Banner { ok: false, message },
                }));
                refresh.emit(());
            });
        })
    };
    let busy = pending.get() > 0;

    let body = match &*status {
        None => html! { <p class="empty">{ "Loading…" }</p> },
        Some(Err(e)) => html! {
            <p class="c-red">{ format!("Can't reach the daemon: {e}. Retrying…") }</p>
        },
        Some(Ok(status)) => view(status, &on_command, busy),
    };

    let machine = match &*status {
        Some(Ok(s)) => s.machine.clone().map(AttrValue::from),
        _ => None,
    };
    html! {
        <>
            <SiteHeader {machine} />
            <main>
                if let Some(b) = &*banner {
                    <div id="result" role="status" class={if b.ok { "ok" } else { "err" }}>{ &b.message }</div>
                }
                { body }
            </main>
        </>
    }
}

fn view(status: &Status, on_command: &Callback<CommandRequest>, busy: bool) -> Html {
    let workers = status.workers().count();
    html! {
        <>
            <p class="summary">{ format!(
                "{workers} workers · {} triages · next poll in {} · daemon pid {} · v{}",
                status.triages.len(), status.next_poll, status.pid, status.version,
            ) }</p>
            <FailedSetups status={status.clone()} on_command={on_command.clone()} {busy} />
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

fn log_error(message: &str) {
    web_sys::console::error_1(&message.into());
}

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
