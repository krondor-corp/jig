//! The client's errors, and the body the daemon refuses with.

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// The daemon refused, with its reason (an [`ErrorBody`]).
    #[error("{1}")]
    Refused(StatusCode, String),
    /// A non-2xx without an [`ErrorBody`]: something other than the API
    /// answered.
    #[error("HTTP {0}: {1}")]
    HttpStatus(StatusCode, String),
    #[error("decode response: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

/// The JSON body of every refusal the daemon sends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}
