//! Toasts: what a command did, in the corner, without moving the page.
//!
//! A command shows a pending toast at once, which becomes its result when
//! the daemon answers. Successes leave on their own; failures stay until
//! dismissed, so a reason is never missed.

use std::rc::Rc;

use gloo_timers::callback::Timeout;
use yew::prelude::*;

/// How long a success stays up.
const SUCCESS_MS: u32 = 4_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Pending,
    Ok,
    Err,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id: u32,
    pub tone: Tone,
    pub message: String,
}

/// Every toast on screen, newest last.
#[derive(Debug, Default, PartialEq)]
pub struct Toasts {
    list: Vec<Toast>,
    next: u32,
}

pub enum ToastAction {
    /// Put up a toast; its id is the next one.
    Show(Tone, String),
    /// A pending toast's outcome.
    Settle(u32, Tone, String),
    Dismiss(u32),
}

impl Toasts {
    /// The id the next [`ToastAction::Show`] will get.
    pub fn next_id(&self) -> u32 {
        self.next
    }
}

impl Reducible for Toasts {
    type Action = ToastAction;

    fn reduce(self: Rc<Self>, action: ToastAction) -> Rc<Self> {
        let mut list = self.list.clone();
        let mut next = self.next;
        match action {
            ToastAction::Show(tone, message) => {
                list.push(Toast {
                    id: next,
                    tone,
                    message,
                });
                next += 1;
            }
            ToastAction::Settle(id, tone, message) => {
                if let Some(toast) = list.iter_mut().find(|t| t.id == id) {
                    toast.tone = tone;
                    toast.message = message;
                }
            }
            ToastAction::Dismiss(id) => list.retain(|t| t.id != id),
        }
        Rc::new(Self { list, next })
    }
}

/// Put up a pending toast for `message`; returns its id for
/// [`settle`].
pub fn show(toasts: &UseReducerHandle<Toasts>, tone: Tone, message: String) -> u32 {
    let id = toasts.next_id();
    toasts.dispatch(ToastAction::Show(tone, message));
    if tone == Tone::Ok {
        expire(toasts, id);
    }
    id
}

/// Turn toast `id` into its outcome. A success leaves after a moment.
pub fn settle(toasts: &UseReducerHandle<Toasts>, id: u32, ok: bool, message: String) {
    let tone = if ok { Tone::Ok } else { Tone::Err };
    toasts.dispatch(ToastAction::Settle(id, tone, message));
    if ok {
        expire(toasts, id);
    }
}

fn expire(toasts: &UseReducerHandle<Toasts>, id: u32) {
    let toasts = toasts.clone();
    Timeout::new(SUCCESS_MS, move || {
        toasts.dispatch(ToastAction::Dismiss(id))
    })
    .forget();
}

#[derive(Properties, PartialEq)]
pub struct ToastStackProps {
    pub toasts: UseReducerHandle<Toasts>,
}

#[function_component(ToastStack)]
pub fn toast_stack(props: &ToastStackProps) -> Html {
    html! {
        <div class="toasts" role="status" aria-live="polite">
            { for props.toasts.list.iter().map(|toast| {
                let class = classes!("toast", match toast.tone {
                    Tone::Pending => "pending",
                    Tone::Ok => "ok",
                    Tone::Err => "err",
                });
                let dismiss = {
                    let toasts = props.toasts.clone();
                    let id = toast.id;
                    Callback::from(move |_| toasts.dispatch(ToastAction::Dismiss(id)))
                };
                html! {
                    <div {class} key={toast.id}>
                        <span class="toast-message">{ &toast.message }</span>
                        <button type="button" class="toast-close" title="dismiss" onclick={dismiss}>{ "×" }</button>
                    </div>
                }
            }) }
        </div>
    }
}
