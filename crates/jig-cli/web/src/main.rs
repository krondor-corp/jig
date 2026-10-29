//! jig web view: a Yew (Rust/WASM) app served by the daemon.
//!
//! The daemon owns the data: every row, cell and color in
//! [`jig_api::Status`] is what `jig ps` would print. This app lays it out,
//! adds links and command buttons, and polls for a fresh frame. It polls the
//! fleet: the daemon that served the page answers for itself and every peer,
//! so any machine's page shows (and drives) all of them.
//!
//! Layout: a sidebar of machines (`Sidebar`) beside the selected view: the
//! overview of what needs a human anywhere (`Overview`, where the page
//! lands), or one machine (`MachineView`). The selection is in the URL hash
//! (`route`).
//!
//! Structure: `api` (typed calls to `/api/v0`) · `components` (the view's
//! pieces) · `route` (which view). `main.rs` loads the session, polls, and
//! runs commands.

mod api;
mod components;
mod route;

use std::cell::Cell;
use std::rc::Rc;

use gloo_events::EventListener;
use gloo_timers::callback::Interval;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use api::Action;
use components::{MachineView, Overview, Sidebar, SiteHeader};
use jig_api::Fleet;
use route::View;

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
    let fleet = use_state(|| None::<Result<Fleet, String>>);
    let token = use_state(|| None::<String>);
    let view = use_state(View::current);
    {
        let view = view.clone();
        use_effect_with((), move |_| {
            let listener = web_sys::window().map(|window| {
                EventListener::new(&window, "hashchange", move |_| view.set(View::current()))
            });
            move || drop(listener)
        });
    }
    let banner = use_state(|| None::<Banner>);
    // Commands in flight. A refresh that lands mid-command would show the
    // old state, so refreshes wait. Shared with the interval, which only
    // sees the value from the first render.
    let pending: Rc<Cell<u32>> = use_memo((), |_| Cell::new(0));

    let refresh = {
        let fleet = fleet.clone();
        let pending = pending.clone();
        Callback::from(move |_: ()| {
            if pending.get() > 0 {
                return;
            }
            let fleet = fleet.clone();
            spawn_local(async move { fleet.set(Some(api::fleet().await)) });
        })
    };

    {
        let token = token.clone();
        let refresh = refresh.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match api::session().await {
                    Ok(session) => token.set(Some(session)),
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
        Callback::from(move |action: Action| {
            let Some(token) = (*token).clone() else {
                banner.set(Some(Banner {
                    ok: false,
                    message: "no session with the daemon yet — reload the page".into(),
                }));
                return;
            };
            banner.set(Some(Banner {
                ok: true,
                message: format!("{} {} …", action.name(), action.worker()),
            }));
            pending.set(pending.get() + 1);
            let banner = banner.clone();
            let refresh = refresh.clone();
            let pending = pending.clone();
            spawn_local(async move {
                let result = action.send(&token).await;
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

    let body = match &*fleet {
        None => html! { <p class="empty">{ "Loading…" }</p> },
        Some(Err(e)) => html! {
            <p class="c-red">{ format!("Can't reach the daemon: {e}. Retrying…") }</p>
        },
        Some(Ok(fleet)) => html! {
            <div class="layout">
                <Sidebar fleet={fleet.clone()} view={(*view).clone()} />
                <div class="pane">
                    if let Some(e) = &fleet.peers_error {
                        <p class="c-yellow">{ format!("Can't list the other machines: {e}") }</p>
                    }
                    { pane(fleet, &view, &on_command, busy) }
                </div>
            </div>
        },
    };

    // The machine serving the page is always first.
    let machine = match &*fleet {
        Some(Ok(f)) => f.machines.first().map(|m| AttrValue::from(m.name.clone())),
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

/// The selected view.
fn pane(fleet: &Fleet, view: &View, on_command: &Callback<Action>, busy: bool) -> Html {
    match view {
        View::Overview => html! {
            <Overview fleet={fleet.clone()} on_command={on_command.clone()} {busy} />
        },
        View::Machine(name) => match fleet.machines.iter().find(|m| &m.name == name) {
            Some(machine) => html! {
                <MachineView
                    key={name.clone()}
                    machine={machine.clone()}
                    on_command={on_command.clone()}
                    {busy}
                />
            },
            None => html! {
                <p class="empty">
                    { format!("No machine called {name} in this fleet. ") }
                    <a href={View::Overview.href()}>{ "Back to the overview" }</a>
                </p>
            },
        },
    }
}

fn log_error(message: &str) {
    web_sys::console::error_1(&message.into());
}

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
