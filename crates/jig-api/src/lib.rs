//! jig's web view API: what the daemon serves and the web app calls.
//!
//! ## The pattern
//!
//! zim-api's (krondor-corp/zim). Every endpoint's **request type** implements
//! [`ApiRequest`]: it owns its method, path and body by building a
//! [`reqwest::RequestBuilder`]. The [`Client`] is generic over the trait:
//! `client.call(KillRequest { .. }).await?` sends it and decodes the typed
//! [`ApiRequest::Response`].
//!
//! A module per subject, a file per request. Each request file also exports
//! its route's `PATH`, and the daemon mounts the matching handler on it, so a
//! route can't move on one side only.
//!
//! Auth is a property of the client, not the request: commands need the
//! session token, and [`Client::with_token`] attaches it to every request.
//!
//! ## Who depends on what
//!
//! - **jig-cli** (daemon): its axum routes (`daemon/web/api/`, laid out like
//!   this crate) deserialize these requests and serialize their responses.
//! - **crates/jig-cli/web** (wasm app): calls these requests over `fetch`.
//! - **jig-cli's tests** drive a real daemon with the same [`Client`].
//!
//! Everything here compiles for every consumer, wasm32 included: reqwest
//! target-gates its native stack.

mod error;
pub mod session;
pub mod status;
pub mod workers;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client as HttpClient, RequestBuilder};
use serde::de::DeserializeOwned;

pub use error::{ApiError, ErrorBody};
/// Re-exported so callers (notably the wasm app) can build a base [`Url`]
/// without a `url` dependency of their own.
pub use reqwest::Url;
pub use session::{Session, SessionRequest};
pub use status::{Cell, FailedSetup, Repo, Status, StatusRequest, Tone, Triage, Worker};
pub use workers::{
    CommandOutcome, KillRequest, ModelRequest, NudgeRequest, PruneRequest, RetrySetupRequest,
    SetupLog, SetupLogRequest,
};

/// The header a [`Client::with_token`] client carries the session token in.
pub const TOKEN_HEADER: &str = "x-jig-token";

/// A typed HTTP request. The implementor owns everything about how the
/// request is formed (method, path, query, body) by returning a fully built
/// [`RequestBuilder`]. The [`Client`] only sends it and decodes
/// [`Self::Response`].
pub trait ApiRequest {
    type Response: DeserializeOwned;
    fn build_request(self, base_url: &Url, client: &HttpClient) -> RequestBuilder;
}

/// Generic typed client: a base URL and a reqwest client, executing any
/// [`ApiRequest`].
#[derive(Debug, Clone)]
pub struct Client {
    base: Url,
    http: HttpClient,
}

impl Client {
    /// A client for the read-only routes.
    pub fn new(base: &Url) -> Result<Self, ApiError> {
        Ok(Self {
            base: base.clone(),
            http: HttpClient::builder().build()?,
        })
    }

    /// A client that sends `token` (from [`SessionRequest`]) in
    /// [`TOKEN_HEADER`] on every request, which commands require.
    pub fn with_token(base: &Url, token: &str) -> Result<Self, ApiError> {
        let mut value = HeaderValue::from_str(token).map_err(|e| ApiError::Other(e.to_string()))?;
        value.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(TOKEN_HEADER, value);
        Ok(Self {
            base: base.clone(),
            http: HttpClient::builder().default_headers(headers).build()?,
        })
    }

    pub fn base(&self) -> &Url {
        &self.base
    }

    /// Send `req` and decode the typed reply. A refusal with an
    /// [`ErrorBody`] is [`ApiError::Refused`] with the daemon's reason; any
    /// other non-2xx is [`ApiError::HttpStatus`].
    pub async fn call<R: ApiRequest>(&self, req: R) -> Result<R::Response, ApiError> {
        let response = req.build_request(&self.base, &self.http).send().await?;
        let status = response.status();
        let body = response.bytes().await?;
        if status.is_success() {
            return Ok(serde_json::from_slice(&body)?);
        }
        Err(match serde_json::from_slice::<ErrorBody>(&body) {
            Ok(refusal) => ApiError::Refused(status, refusal.error),
            Err(_) => ApiError::HttpStatus(status, String::from_utf8_lossy(&body).into_owned()),
        })
    }
}
