//! Error types for the IronProse API client.

use std::fmt;

/// An error returned by the IronProse API or the HTTP transport layer.
#[derive(Debug)]
pub enum ApiError {
    /// HTTP transport failure (network error, timeout, etc.)
    Transport(String),

    /// API returned a non-200 status code.
    Http { status: u16, message: String },

    /// Failed to parse the API response body.
    Parse(String),

    /// Invalid input (missing args, bad file path, etc.)
    Input(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Transport(msg) => write!(f, "Network error: {msg}"),
            ApiError::Http { status, message } => write!(f, "API error (HTTP {status}): {message}"),
            ApiError::Parse(msg) => write!(f, "Response parse error: {msg}"),
            ApiError::Input(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Sanitize and truncate an upstream HTTP response body before including it in
/// error messages. Prevents raw upstream content (which may contain secrets,
/// multiline HTML, or excessively long payloads) from leaking.
fn sanitize_and_truncate_body(body: &str) -> String {
    let sanitized = body.replace('\n', "\\n").replace('\r', "\\r");

    let redacted = regex_lite::Regex::new(r"[A-Za-z0-9_-]{20,}")
        .map(|re| re.replace_all(&sanitized, "[REDACTED]").into_owned())
        .unwrap_or(sanitized);

    if redacted.len() > 200 {
        format!("{}...", &redacted[..200])
    } else {
        redacted
    }
}

/// Map an HTTP status code from the remote API to an appropriate `ApiError`.
///
/// Error messages are intentionally clear and discourage retry loops
/// for non-transient errors (402, 429).
pub fn http_status_to_error(status: u16, body: &str) -> ApiError {
    let safe_body = sanitize_and_truncate_body(body);
    let message = match status {
        402 => format!(
            "IronProse API requires a paid subscription (HTTP 402). \
             Visit https://ironprose.com to activate your API key. Details: {safe_body}"
        ),
        429 => format!(
            "IronProse API rate limit exceeded (HTTP 429). \
             Please wait before sending more requests. Details: {safe_body}"
        ),
        500..=599 => format!(
            "IronProse API server error (HTTP {status}). \
             This is a transient error — retry after a brief delay. Details: {safe_body}"
        ),
        401 | 403 => format!(
            "IronProse API authentication failed (HTTP {status}). \
             Check your IRONPROSE_API_KEY. Details: {safe_body}"
        ),
        _ => format!("IronProse API returned unexpected status {status}: {safe_body}"),
    };
    ApiError::Http { status, message }
}
