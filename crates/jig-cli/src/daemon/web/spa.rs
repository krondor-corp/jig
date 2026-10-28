//! Serves the Yew app (`crates/jig-cli/web`, built by Trunk into
//! `web/dist/`) embedded in the binary.
//!
//! Mounted as the router's fallback, after `/api`. An embedded file is
//! served as itself; any other path gets `index.html`, so the app owns
//! every page URL. Nothing here is gated: the app is public, and only the
//! API's commands need the session token.
//!
//! A debug build reads `web/dist/` from disk on each request (rust-embed's
//! default), so `make build-web` shows up without rebuilding jig. A release
//! build embeds it.

use std::sync::OnceLock;

use axum::extract::OriginalUri;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rust_embed::RustEmbed;
use sha2::{Digest, Sha256};

#[derive(RustEmbed)]
#[folder = "web/dist/"]
struct Dist;

const NOT_BUILT: &str =
    "jig's web view is not built into this binary — run `make build-web`, then rebuild jig";

pub async fn serve(OriginalUri(uri): OriginalUri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if !path.is_empty() {
        if let Some(file) = Dist::get(path) {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            return (
                [(header::CONTENT_TYPE, mime.as_ref())],
                file.data.into_owned(),
            )
                .into_response();
        }
    }
    match Dist::get("index.html") {
        Some(file) => (
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            file.data.into_owned(),
        )
            .into_response(),
        None => (StatusCode::SERVICE_UNAVAILABLE, NOT_BUILT).into_response(),
    }
}

/// The Content Security Policy for every response.
///
/// Scripts come only from us. `'wasm-unsafe-eval'` lets the bundle compile
/// its wasm module, and Trunk's inline loader script is allowed by its
/// hash, so `'unsafe-inline'` is never needed.
pub fn csp() -> &'static str {
    static CSP: OnceLock<String> = OnceLock::new();
    CSP.get_or_init(|| {
        let index = Dist::get("index.html")
            .map(|f| String::from_utf8_lossy(&f.data).into_owned())
            .unwrap_or_default();
        policy(&index)
    })
}

/// The policy for an app whose `index.html` is `index`.
fn policy(index: &str) -> String {
    let hashes: String = inline_scripts(index)
        .map(|script| format!(" 'sha256-{}'", STANDARD.encode(Sha256::digest(script))))
        .collect();
    format!(
        "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'{hashes}; style-src 'self'; \
         connect-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; \
         form-action 'none'"
    )
}

/// The bodies of `<script>` elements without a `src`, exactly as written.
/// Those bytes are what a CSP hash covers.
fn inline_scripts(html: &str) -> impl Iterator<Item = &str> {
    html.split("<script").skip(1).filter_map(|rest| {
        let (open, after) = rest.split_once('>')?;
        if open.contains("src=") {
            return None;
        }
        after.split_once("</script>").map(|(body, _)| body)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_scripts_are_allowed_by_hash_and_sourced_ones_are_not() {
        let html =
            r#"<head><script src="/app.js"></script><script type="module">init()</script></head>"#;
        let scripts: Vec<&str> = inline_scripts(html).collect();
        assert_eq!(scripts, ["init()"]);

        let csp = policy(html);
        let hash = STANDARD.encode(Sha256::digest("init()"));
        assert!(csp.contains(&format!("'sha256-{hash}'")), "{csp}");
        assert!(!csp.contains("unsafe-inline"));
        assert!(csp.contains("'wasm-unsafe-eval'"));
    }

    #[test]
    fn a_page_without_inline_scripts_has_no_hashes() {
        assert!(!policy("<html></html>").contains("sha256"));
    }
}
