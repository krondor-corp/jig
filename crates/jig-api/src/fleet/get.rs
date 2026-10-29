//! `GET /api/v0/fleet`: this machine and every peer, each with its frame.

use reqwest::{Client as HttpClient, RequestBuilder, Url};

use crate::ApiRequest;

use super::Fleet;

pub const PATH: &str = "/api/v0/fleet";

/// Polls every peer before answering, so it takes as long as the slowest
/// one (bounded by the daemon). **Daemon route:** [`PATH`].
pub struct FleetRequest;

impl ApiRequest for FleetRequest {
    type Response = Fleet;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        http.get(base.join(PATH).expect("static path"))
    }
}
