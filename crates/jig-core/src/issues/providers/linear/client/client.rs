use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::error::{LinearError, Result};
use super::request::LinearRequest;

const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

/// Upper bound on a whole Linear request, connect through body.
///
/// ureq has no timeouts by default, and the daemon's actors call Linear on
/// every poll. A connection that dies without a reset (typically across
/// laptop sleep) would block the actor forever — and since `ActorHandle`
/// drops requests while one is in flight, auto-spawn and triage silently
/// stop until the daemon restarts. Failing after this long just skips a
/// poll; the next one retries.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct LinearClient {
    api_key: String,
    endpoint: String,
    timeout: Duration,
}

#[derive(Serialize)]
struct GqlBody {
    query: &'static str,
    variables: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GqlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Debug, Deserialize)]
struct GqlError {
    message: String,
}

impl LinearClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            endpoint: LINEAR_API_URL.to_string(),
            timeout: REQUEST_TIMEOUT,
        }
    }

    pub fn execute<R: LinearRequest>(&self, request: R) -> Result<R::Output> {
        let variables = request.variables();
        let wrapper: R::Response = self.raw_execute(R::QUERY, variables)?;
        R::extract(wrapper)
    }

    fn raw_execute<T: DeserializeOwned>(
        &self,
        query: &'static str,
        variables: serde_json::Value,
    ) -> Result<T> {
        let body = GqlBody { query, variables };

        let response = ureq::post(&self.endpoint)
            .config()
            .http_status_as_error(false)
            .timeout_global(Some(self.timeout))
            .timeout_connect(Some(CONNECT_TIMEOUT.min(self.timeout)))
            .build()
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .send_json(&body)
            .map_err(|e| LinearError::Http(e.to_string()))?;

        let status = response.status();

        let text = response
            .into_body()
            .read_to_string()
            .map_err(|e| LinearError::ReadBody(e.to_string()))?;

        if status.as_u16() >= 400 {
            return Err(LinearError::Status {
                status: status.as_u16(),
                body: text,
            });
        }

        let gql: GqlResponse<T> = serde_json::from_str(&text).map_err(|e| LinearError::Parse {
            msg: e.to_string(),
            body: text.clone(),
        })?;

        if let Some(errors) = gql.errors {
            let msgs: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            return Err(LinearError::GraphQL(msgs.join("; ")));
        }

        gql.data.ok_or(LinearError::NoData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresponsive_server_times_out_instead_of_hanging() {
        // Accepts the connection, then never answers — like a socket that
        // died across sleep without a reset.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _held: Vec<_> = listener.incoming().collect();
        });

        let client = LinearClient {
            api_key: "test".into(),
            endpoint: format!("http://{addr}/graphql"),
            timeout: Duration::from_millis(300),
        };

        let started = std::time::Instant::now();
        let result: Result<serde_json::Value> =
            client.raw_execute("{ viewer { id } }", serde_json::json!({}));

        assert!(matches!(result, Err(LinearError::Http(_))), "{result:?}");
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
