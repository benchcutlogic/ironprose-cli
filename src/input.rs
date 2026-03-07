//! Input validation and hardening against agent hallucinations.
//!
//! Agents hallucinate. Build like it. This module validates all external
//! inputs before they reach the API client or filesystem.

use std::path::Path;

/// Validate a file path is safe to read.
///
/// Rejects:
/// - Paths containing `..` (directory traversal)
/// - Absolute paths (agents should only read relative to CWD)
/// - Paths with control characters
/// - Paths containing URL-encoded sequences (`%`)
pub fn validate_file_path(path: &str) -> Result<(), String> {
    // Reject control characters
    if path.bytes().any(|b| b < 0x20 && b != b'\t') {
        return Err(format!("File path contains control characters: {:?}", path));
    }

    // Reject URL-encoded sequences (agent hallucination: %2e%2e for ..)
    if path.contains('%') {
        return Err(format!(
            "File path contains percent-encoding (possible double-encoding): {path}"
        ));
    }

    // Reject path traversal
    let p = Path::new(path);
    for component in p.components() {
        if let std::path::Component::ParentDir = component {
            return Err(format!(
                "File path contains directory traversal (..): {path}"
            ));
        }
    }

    // Reject absolute paths — agents should work relative to CWD
    if p.is_absolute() {
        return Err(format!(
            "Absolute file paths are not allowed: {path}. Use a relative path."
        ));
    }

    // Verify the file exists
    if !p.exists() {
        return Err(format!("File not found: {path}"));
    }

    Ok(())
}

/// Validate text input doesn't contain control characters.
///
/// Allows: printable ASCII, UTF-8, newlines, carriage returns, tabs.
/// Rejects: NUL bytes, bell, backspace, and other control chars below 0x20.
pub fn validate_text_input(text: &str) -> Result<(), String> {
    for (i, byte) in text.bytes().enumerate() {
        if byte < 0x20 && byte != b'\n' && byte != b'\r' && byte != b'\t' {
            return Err(format!(
                "Text input contains control character (0x{byte:02x}) at byte offset {i}. \
                 Remove non-printable characters before submitting."
            ));
        }
    }
    Ok(())
}

/// Validate a raw JSON string is syntactically valid.
pub fn validate_json_input(json: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(json).map_err(|e| format!("Invalid JSON input: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_traversal() {
        assert!(validate_file_path("../../etc/passwd").is_err());
        assert!(validate_file_path("foo/../bar").is_err());
    }

    #[test]
    fn test_reject_absolute_path() {
        assert!(validate_file_path("/etc/passwd").is_err());
    }

    #[test]
    fn test_reject_percent_encoding() {
        assert!(validate_file_path("%2e%2e/etc/passwd").is_err());
    }

    #[test]
    fn test_reject_control_chars_in_path() {
        assert!(validate_file_path("foo\x00bar").is_err());
        assert!(validate_file_path("foo\x07bar").is_err());
    }

    #[test]
    fn test_accept_valid_relative_path() {
        // This will fail with "File not found" but NOT with a security error
        let err = validate_file_path("chapter-07.md").unwrap_err();
        assert!(
            err.contains("not found"),
            "Expected 'not found', got: {err}"
        );
    }

    #[test]
    fn test_text_rejects_control_chars() {
        assert!(validate_text_input("hello\x00world").is_err());
        assert!(validate_text_input("hello\x07world").is_err());
    }

    #[test]
    fn test_text_allows_normal_content() {
        assert!(validate_text_input("Hello, world!\nNew paragraph.\tTabbed.").is_ok());
        assert!(validate_text_input("Héllo wörld — em dash").is_ok());
    }

    #[test]
    fn test_json_validation() {
        assert!(validate_json_input(r#"{"text": "hello"}"#).is_ok());
        assert!(validate_json_input("not json").is_err());
    }
}
