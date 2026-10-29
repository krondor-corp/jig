//! `GET /api/v0/issues?machine=<node>`: a machine's issues, as last fetched.

use reqwest::{Client as HttpClient, RequestBuilder, Url};
use serde::{Deserialize, Serialize};

use crate::ApiRequest;

use super::Issues;

pub const PATH: &str = "/api/v0/issues";

/// **Daemon route:** [`PATH`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssuesRequest {
    /// Whose issues: a [`crate::Machine::node`], or `None` for the machine
    /// serving the page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
}

impl ApiRequest for IssuesRequest {
    type Response = Issues;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        let request = http.get(base.join(PATH).expect("static path"));
        match &self.machine {
            Some(machine) => request.query(&[("machine", machine.as_str())]),
            None => request,
        }
    }
}
