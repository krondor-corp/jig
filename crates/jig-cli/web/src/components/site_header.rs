//! jig.krondor.org's header: the grid logo, `JIG`, and the theme toggle.
//! The theme is saved like the site saves it: `localStorage.theme`, set as
//! `data-theme` on `<html>`.

use yew::prelude::*;

const THEME_KEY: &str = "theme";

#[derive(Properties, PartialEq)]
pub struct SiteHeaderProps {
    /// This machine's fleet name, beside the wordmark.
    pub machine: Option<AttrValue>,
}

#[function_component(SiteHeader)]
pub fn site_header(props: &SiteHeaderProps) -> Html {
    use_effect_with((), |_| {
        if let Some(theme) = saved_theme() {
            set_theme(&theme);
        }
        || ()
    });
    let toggle = Callback::from(|_| {
        let next = if is_dark() { "light" } else { "dark" };
        set_theme(next);
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(THEME_KEY, next);
        }
    });
    html! {
        <header class="site-header">
            <div class="header-inner">
                <span class="site-title">
                    { logo() }
                    <span class="site-title-text">{ "JIG" }</span>
                    if let Some(machine) = &props.machine {
                        <span class="site-title-machine">{ machine }</span>
                    }
                </span>
                <div class="site-nav">
                    <button class="theme-toggle" type="button" onclick={toggle} aria-label="Toggle theme">
                        { theme_icon() }
                    </button>
                </div>
            </div>
        </header>
    }
}

/// The site's 3×3 grid mark, in the brand color.
fn logo() -> Html {
    let cells = [
        (0, 0, 1.0),
        (13, 0, 1.0),
        (26, 0, 0.55),
        (0, 13, 0.15),
        (13, 13, 1.0),
        (26, 13, 0.15),
        (0, 26, 1.0),
        (13, 26, 0.35),
        (26, 26, 0.15),
    ];
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 36 36" width="24" height="24" aria-hidden="true">
            { for cells.iter().map(|(x, y, opacity)| html! {
                <rect x={x.to_string()} y={y.to_string()} width="10" height="10" rx="2"
                    fill="currentColor" opacity={opacity.to_string()} />
            }) }
        </svg>
    }
}

/// A half-filled circle: one icon for both directions.
fn theme_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="9" />
            <path d="M12 3a9 9 0 0 1 0 18z" fill="currentColor" />
        </svg>
    }
}

fn saved_theme() -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(THEME_KEY)
        .ok()?
}

/// Whether the page is dark now: the saved choice, else the system's.
fn is_dark() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let chosen = window
        .document()
        .and_then(|d| d.document_element())
        .and_then(|e| e.get_attribute("data-theme"));
    match chosen.as_deref() {
        Some("dark") => true,
        Some("light") => false,
        _ => window
            .match_media("(prefers-color-scheme: dark)")
            .ok()
            .flatten()
            .is_some_and(|m| m.matches()),
    }
}

fn set_theme(theme: &str) {
    if let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = root.set_attribute("data-theme", theme);
    }
}
