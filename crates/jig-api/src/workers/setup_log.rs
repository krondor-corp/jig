//! `GET /api/v0/workers/setup-log?worker=<repo/branch>`: the end of a
//! worker's `setup.log`.

use reqwest::{Client as HttpClient, RequestBuilder, Url};
use serde::{Deserialize, Serialize};

use crate::ApiRequest;

pub const PATH: &str = "/api/v0/workers/setup-log";

/// The daemon only reads a log path it recorded itself: the request names a
/// worker, never a file. **Daemon route:** [`PATH`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetupLogRequest {
    /// `repo/branch`, as in [`crate::Worker::key`].
    pub worker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetupLog {
    /// Up to the last megabyte of the log.
    pub text: String,
}

impl ApiRequest for SetupLogRequest {
    type Response = SetupLog;
    fn build_request(self, base: &Url, http: &HttpClient) -> RequestBuilder {
        http.get(base.join(PATH).expect("static path"))
            .query(&[("worker", self.worker.as_str())])
    }
}
