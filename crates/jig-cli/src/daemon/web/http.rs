//! Just enough HTTP/1.1 for the web view: one request in, one response out,
//! then the connection closes.
//!
//! `httparse` reads the head; everything else — bodies by `Content-Length`,
//! no chunking, no keep-alive — is the smallest server a same-machine
//! browser needs, and keeps the daemon on std threads.

use std::io::{Read, Write};

/// The most a request head may take. A browser's is a few hundred bytes.
const MAX_HEAD: usize = 16 * 1024;

/// The most a request body may take. Commands are a line of JSON.
const MAX_BODY: usize = 64 * 1024;

/// The most headers a request may carry.
const MAX_HEADERS: usize = 64;

/// A parsed request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    /// The path without its query string.
    pub path: String,
    /// The raw query string, without the `?`.
    pub query: String,
    /// Header names lower-cased.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// The first header called `name` (lower-case).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// The query parameter `name`, percent-decoded.
    pub fn query_param(&self, name: &str) -> Option<String> {
        self.query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            (k == name).then(|| percent_decode(v))
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("connection closed before a full request")]
    Closed,
    #[error("request too large")]
    TooLarge,
    #[error("malformed request: {0}")]
    Malformed(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Read one request off `stream`.
pub fn read_request(stream: &mut impl Read) -> Result<HttpRequest, HttpError> {
    let mut buf = Vec::with_capacity(1024);
    let mut chunk = [0u8; 4096];
    let (head_len, mut request) = loop {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Err(HttpError::Closed);
        }
        buf.extend_from_slice(&chunk[..n]);

        let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
        let mut parsed = httparse::Request::new(&mut headers);
        match parsed.parse(&buf) {
            Ok(httparse::Status::Complete(len)) => break (len, head(&parsed)?),
            Ok(httparse::Status::Partial) if buf.len() > MAX_HEAD => {
                return Err(HttpError::TooLarge)
            }
            Ok(httparse::Status::Partial) => continue,
            Err(e) => return Err(HttpError::Malformed(e.to_string())),
        }
    };

    let length = match request.header("content-length") {
        Some(v) => v
            .trim()
            .parse::<usize>()
            .map_err(|_| HttpError::Malformed("bad content-length".into()))?,
        None => 0,
    };
    if length > MAX_BODY {
        return Err(HttpError::TooLarge);
    }
    let mut body = buf.split_off(head_len);
    if body.len() < length {
        let start = body.len();
        body.resize(length, 0);
        stream
            .read_exact(&mut body[start..])
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::UnexpectedEof => HttpError::Closed,
                _ => HttpError::Io(e),
            })?;
    }
    body.truncate(length);
    request.body = body;
    Ok(request)
}

fn head(parsed: &httparse::Request) -> Result<HttpRequest, HttpError> {
    let method = parsed
        .method
        .ok_or_else(|| HttpError::Malformed("no method".into()))?;
    let target = parsed
        .path
        .ok_or_else(|| HttpError::Malformed("no path".into()))?;
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let headers = parsed
        .headers
        .iter()
        .map(|h| {
            (
                h.name.to_ascii_lowercase(),
                String::from_utf8_lossy(h.value).into_owned(),
            )
        })
        .collect();
    Ok(HttpRequest {
        method: method.to_string(),
        path: path.to_string(),
        query: query.to_string(),
        headers,
        body: Vec::new(),
    })
}

/// Decode `%XX` escapes and `+` as space. Invalid escapes pass through.
pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                let hex = |b: u8| (b as char).to_digit(16);
                match (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                    (Some(hi), Some(lo)) => {
                        out.push((hi * 16 + lo) as u8);
                        i += 2;
                    }
                    _ => out.push(b'%'),
                }
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A response, written with `Connection: close`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub headers: Vec<(&'static str, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: u16, content_type: &'static str, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            content_type,
            headers: Vec::new(),
            body: body.into(),
        }
    }

    pub fn html(body: impl Into<Vec<u8>>) -> Self {
        Self::new(200, "text/html; charset=utf-8", body)
    }

    pub fn text(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self::new(status, "text/plain; charset=utf-8", body)
    }

    pub fn json(status: u16, value: &serde_json::Value) -> Self {
        Self::new(status, "application/json", value.to_string())
    }

    pub fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }

    pub fn write_to(&self, stream: &mut impl Write) -> std::io::Result<()> {
        let mut head = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
            self.status,
            reason(self.status),
            self.content_type,
            self.body.len()
        );
        for (name, value) in &self.headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        head.push_str("\r\n");
        stream.write_all(head.as_bytes())?;
        stream.write_all(&self.body)?;
        stream.flush()
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Payload Too Large",
        421 => "Misdirected Request",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> Result<HttpRequest, HttpError> {
        read_request(&mut raw.as_bytes())
    }

    #[test]
    fn a_get_parses_path_query_and_headers() {
        let req =
            parse("GET /setup-log?worker=jig%2Ffeat%2Fx HTTP/1.1\r\nHost: 127.0.0.1:7477\r\n\r\n")
                .unwrap();
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/setup-log");
        assert_eq!(req.header("host"), Some("127.0.0.1:7477"));
        assert_eq!(req.query_param("worker").as_deref(), Some("jig/feat/x"));
        assert_eq!(req.query_param("nope"), None);
    }

    #[test]
    fn a_post_reads_its_body_by_length() {
        let req = parse("POST /command HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello").unwrap();
        assert_eq!(req.body, b"hello");
    }

    #[test]
    fn a_short_body_is_a_closed_connection() {
        assert!(matches!(
            parse("POST /command HTTP/1.1\r\nContent-Length: 10\r\n\r\nhi"),
            Err(HttpError::Closed)
        ));
    }

    #[test]
    fn an_oversized_body_is_refused_before_reading() {
        let raw = format!(
            "POST /command HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            MAX_BODY + 1
        );
        assert!(matches!(parse(&raw), Err(HttpError::TooLarge)));
    }

    #[test]
    fn garbage_is_malformed() {
        assert!(matches!(
            parse("\x01\x02 nonsense\r\n\r\n"),
            Err(HttpError::Malformed(_))
        ));
    }

    #[test]
    fn percent_decoding_handles_bad_escapes() {
        assert_eq!(percent_decode("a%20b+c"), "a b c");
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz"), "%zz");
    }

    #[test]
    fn a_response_closes_the_connection() {
        let mut out = Vec::new();
        HttpResponse::text(404, "not found")
            .with_header("Cache-Control", "no-store")
            .write_to(&mut out)
            .unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(out.starts_with("HTTP/1.1 404 Not Found\r\n"));
        assert!(out.contains("Connection: close\r\n"));
        assert!(out.contains("Content-Length: 9\r\n"));
        assert!(out.contains("Cache-Control: no-store\r\n"));
        assert!(out.ends_with("\r\n\r\nnot found"));
    }
}
