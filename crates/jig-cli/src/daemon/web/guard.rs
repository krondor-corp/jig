//! The web view's hardening: who may reach it, and what a command must
//! carry. See the module docs in `super` for why each one exists.

use std::sync::Arc;

use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use super::api::refuse;
use super::{spa, Backend, Site};

/// Refuse any request whose `Host` isn't this server. That blocks DNS
/// rebinding before a handler runs.
pub async fn host<B: Backend>(
    State(site): State<Arc<Site<B>>>,
    request: Request,
    next: Next,
) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok());
    if !host_allowed(site.port, host) {
        return (StatusCode::FORBIDDEN, "forbidden host").into_response();
    }
    next.run(request).await
}

/// Headers on every response: never cached, never framed, and no script
/// or style but our own.
pub async fn headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    for (name, value) in [
        (header::CACHE_CONTROL, "no-store"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::X_FRAME_OPTIONS, "DENY"),
        (header::REFERRER_POLICY, "no-referrer"),
    ] {
        headers.insert(name, HeaderValue::from_static(value));
    }
    if let Ok(csp) = HeaderValue::from_str(spa::csp()) {
        headers.insert(header::CONTENT_SECURITY_POLICY, csp);
    }
    response
}

/// Proof a request may run a command: it carries the session token and,
/// when the browser sends an `Origin`, ours. A handler that takes this can't
/// be reached without both.
pub struct Authorized;

impl<B: Backend> FromRequestParts<Arc<Site<B>>> for Authorized {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        site: &Arc<Site<B>>,
    ) -> Result<Self, Self::Rejection> {
        let token_ok = parts
            .headers
            .get(jig_api::TOKEN_HEADER)
            .and_then(|t| t.to_str().ok())
            .is_some_and(|t| constant_time_eq(t.as_bytes(), site.token.as_bytes()));
        if !token_ok {
            return Err(refuse(
                StatusCode::FORBIDDEN,
                "missing or wrong token — reload the page",
            ));
        }
        let origin = parts
            .headers
            .get(header::ORIGIN)
            .map(|o| o.to_str().unwrap_or(""));
        if !origin_allowed(site.port, origin) {
            return Err(refuse(StatusCode::FORBIDDEN, "forbidden origin"));
        }
        Ok(Authorized)
    }
}

/// Whether `host` names this server. Anything else is a page that reached
/// us through a hostname it controls.
pub fn host_allowed(port: u16, host: Option<&str>) -> bool {
    let Some(host) = host else { return false };
    host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
}

/// Whether `origin`, when sent, is one of ours.
fn origin_allowed(port: u16, origin: Option<&str>) -> bool {
    match origin {
        None => true,
        Some(origin) => origin
            .strip_prefix("http://")
            .is_some_and(|host| host_allowed(port, Some(host))),
    }
}

/// Compare without leaking how much of the token matched.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
