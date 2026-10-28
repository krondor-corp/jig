//! Typed calls to the daemon's `/api/v0`, over `fetch`. Paths and types
//! come from `jig_api`, the crate the daemon serves them with.

use gloo_net::http::{Request, Response};
use serde::de::DeserializeOwned;

use jig_api::{ApiError, CommandOutcome, CommandRequest, Session, Status};

/// The body of a 2xx response, or the daemon's reason for refusing.
async fn decode<T: DeserializeOwned>(response: Response) -> Result<T, String> {
    if response.ok() {
        return response.json().await.map_err(|e| e.to_string());
    }
    let status = response.status();
    match response.json::<ApiError>().await {
        Ok(e) => Err(e.error),
        Err(_) => Err(format!("the daemon answered {status}")),
    }
}

pub async fn session() -> Result<Session, String> {
    let response = Request::get(jig_api::SESSION)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    decode(response).await
}

pub async fn status() -> Result<Status, String> {
    let response = Request::get(jig_api::STATUS)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    decode(response).await
}

/// Run a command. The outcome's message, or why it was refused.
pub async fn command(token: &str, request: &CommandRequest) -> Result<String, String> {
    let response = Request::post(jig_api::COMMAND)
        .header(jig_api::TOKEN_HEADER, token)
        .json(request)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    decode::<CommandOutcome>(response).await.map(|o| o.message)
}

/// Where a worker's whole `setup.log` is.
pub fn setup_log_url(worker: &str) -> String {
    format!("{}?worker={}", jig_api::SETUP_LOG, encode(worker))
}

/// Percent-encode a query value.
fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
