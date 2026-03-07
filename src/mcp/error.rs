use rmcp::ErrorData as McpError;

/// Sanitize and truncate an upstream HTTP response body before including it in
/// MCP error messages. Prevents raw upstream content (which may contain secrets,
/// multiline HTML, or excessively long payloads) from leaking into client-visible errors.
fn sanitize_and_truncate_body(body: &str) -> String {
    // Replace newlines with escaped representation
    let sanitized = body.replace('\n', "\\n").replace('\r', "\\r");

    // Redact strings that look like API keys / tokens (20+ alphanumeric/dash/underscore chars)
    let redacted = regex_lite::Regex::new(r"[A-Za-z0-9_-]{20,}")
        .map(|re| re.replace_all(&sanitized, "[REDACTED]").into_owned())
        .unwrap_or(sanitized);

    // Truncate to 200 chars max
    if redacted.len() > 200 {
        format!("{}...", &redacted[..200])
    } else {
        redacted
    }
}

/// Map an HTTP status code from the remote API to an appropriate MCP error.
///
/// The error messages are intentionally clear and discourage retry loops
/// for non-transient errors (402, 429).
pub fn http_status_to_mcp_error(status: u16, body: &str) -> McpError {
    let safe_body = sanitize_and_truncate_body(body);
    match status {
        402 => McpError::internal_error(
            format!(
                "IronProse API requires a paid subscription (HTTP 402). \
                 Visit https://ironprose.com to activate your API key. \
                 Do NOT retry this request automatically. Details: {safe_body}"
            ),
            None,
        ),
        429 => McpError::internal_error(
            format!(
                "IronProse API rate limit exceeded (HTTP 429). \
                 Please wait before sending more requests. \
                 Do NOT retry in a tight loop. Details: {safe_body}"
            ),
            None,
        ),
        500..=599 => McpError::internal_error(
            format!(
                "IronProse API server error (HTTP {status}). \
                 This is a transient error — a single retry after a brief delay is acceptable. \
                 Details: {safe_body}"
            ),
            None,
        ),
        401 | 403 => McpError::internal_error(
            format!(
                "IronProse API authentication failed (HTTP {status}). \
                 Check your IRONPROSE_API_KEY. Do NOT retry. Details: {safe_body}"
            ),
            None,
        ),
        _ => McpError::internal_error(
            format!("IronProse API returned unexpected status {status}: {safe_body}"),
            None,
        ),
    }
}

/// Create an MCP error for sandbox violations.
pub fn sandbox_error(msg: impl Into<String>) -> McpError {
    McpError::invalid_params(format!("Workspace sandbox violation: {}", msg.into()), None)
}

/// Create an MCP error for missing workspace configuration.
pub fn no_workspace_error() -> McpError {
    McpError::invalid_params(
        "No workspace directory configured. Set IRONPROSE_WORKSPACE environment variable \
         or pass workspace_dir parameter."
            .to_string(),
        None,
    )
}
