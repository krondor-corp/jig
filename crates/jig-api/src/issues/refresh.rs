//! `POST /api/v0/issues/refresh`: have a machine fetch its issues again.

use reqwest::{Client as HttpClient, RequestBuilder, Url};
use serde::{Deserialize, Serialize};

use crate::{ApiRequest, CommandOutcome};

pub const PATH: &str = "/api/v0/issues/refresh";

/// Needs the session token. Answers at once; the list changes once the
/// fetch lands, a few seconds later. **Daemon route:** [`PATH`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshIssuesRequest {
    /// A [`crate::Machine::node`]; `None` for the machine serving the page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
}

impl ApiRequest for RefreshIssuesRequest {
    type Response = CommandOutcome;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        http.post(base.join(PATH).expect("static path")).json(&self)
    }
}
