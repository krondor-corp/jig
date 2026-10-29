//! `GET /api/v0/session`: the command token.

use reqwest::{Client as HttpClient, RequestBuilder, Url};
use serde::{Deserialize, Serialize};

use crate::ApiRequest;

pub const PATH: &str = "/api/v0/session";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Minted when the daemon starts. Build a [`crate::Client::with_token`]
    /// with it to send commands.
    pub token: String,
}

/// Only a page served by the daemon itself can read the answer: there's no
/// CORS, so another origin's request gets nothing back. **Daemon route:**
/// [`PATH`].
pub struct SessionRequest;

impl ApiRequest for SessionRequest {
    type Response = Session;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        http.get(base.join(PATH).expect("static path"))
    }
}
