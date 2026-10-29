//! `GET /api/v0/status`: everything the daemon knows right now.

use reqwest::{Client as HttpClient, RequestBuilder, Url};

use crate::ApiRequest;

use super::Status;

pub const PATH: &str = "/api/v0/status";

/// **Daemon route:** [`PATH`].
pub struct StatusRequest;

impl ApiRequest for StatusRequest {
    type Response = Status;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        http.get(base.join(PATH).expect("static path"))
    }
}
