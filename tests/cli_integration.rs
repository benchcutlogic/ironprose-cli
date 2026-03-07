//! Integration tests for the IronProse CLI using a mock API server.
//!
//! Each test starts a `wiremock::MockServer`, registers canned responses,
//! and invokes the CLI binary via `assert_cmd` with `--api-url` pointed at
//! the mock server.

use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Canned JSON responses matching the OpenAPI spec types.
mod fixtures {
    pub fn analyze_response() -> serde_json::Value {
        serde_json::json!({
            "score": {
                "concreteness": 0.75,
                "imagery_density": 0.62,
                "vocabulary_richness": 0.58,
                "sentence_variety": 0.70,
                "dialogue_balance": 0.45,
                "pacing_score": 0.68
            },
            "word_count": 6,
            "diagnostics": [{
                "rule": "repetition",
                "severity": "warning",
                "message": "Word 'dark' repeated 2 times in close proximity.",
                "start_line": 0,
                "start_char": 4,
                "end_line": 0,
                "end_char": 8,
                "id": "d-001",
                "source_type": "Heuristic",
                "confidence": 1.0
            }],
            "profile": {
                "avg_sentence_length": 6.0,
                "sentence_length_variance": 0.0,
                "min_sentence_length": 6,
                "max_sentence_length": 6,
                "total_sentences": 1,
                "total_paragraphs": 1,
                "dialogue_line_ratio": 0.0,
                "avg_paragraph_length_words": 6.0
            }
        })
    }

    pub fn compare_response() -> serde_json::Value {
        serde_json::json!({
            "fixed": [{
                "rule": "repetition",
                "severity": "warning",
                "message": "Word 'dark' repeated 2 times.",
                "start_line": 0, "start_char": 4,
                "end_line": 0, "end_char": 8
            }],
            "introduced": [],
            "persistent": [],
            "original_score": {
                "concreteness": 0.75,
                "imagery_density": 0.62,
                "vocabulary_richness": 0.58,
                "sentence_variety": 0.70,
                "dialogue_balance": 0.45,
                "pacing_score": 0.68
            },
            "revised_score": {
                "concreteness": 0.85,
                "imagery_density": 0.72,
                "vocabulary_richness": 0.68,
                "sentence_variety": 0.74,
                "dialogue_balance": 0.55,
                "pacing_score": 0.80
            }
        })
    }

    pub fn list_rules_response() -> serde_json::Value {
        serde_json::json!({
            "rules": [
                { "name": "repetition", "category": "style" },
                { "name": "passive_voice", "category": "grammar" },
                { "name": "weak_verb", "category": "style" }
            ],
            "total": 3
        })
    }

    pub fn rate_response() -> serde_json::Value {
        serde_json::json!({
            "status": "ok",
            "message": "Feedback recorded. Thank you!"
        })
    }

    pub fn insights_response() -> serde_json::Value {
        serde_json::json!({
            "rules": [
                {
                    "rule": "repetition",
                    "total_ratings": 10,
                    "helpful": 7,
                    "not_helpful": 2,
                    "false_positive": 1,
                    "precision_proxy": 0.875
                },
                {
                    "rule": "passive_voice",
                    "total_ratings": 5,
                    "helpful": 3,
                    "not_helpful": 1,
                    "false_positive": 1,
                    "precision_proxy": 0.75
                }
            ],
            "total": 2
        })
    }
}

#[allow(deprecated)]
fn cli() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin("ironprose"))
}

// ── Analyze Tests ──────────────────────────────────────────────

#[tokio::test]
async fn test_analyze_inline_text_json() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/analyze"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::analyze_response()))
        .mount(&server)
        .await;

    cli()
        .args([
            "analyze",
            "The dark night was very dark.",
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("concreteness"))
        .stdout(predicate::str::contains("repetition"))
        .stdout(predicate::str::contains("word_count"));
}

#[tokio::test]
async fn test_analyze_text_output_format() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/analyze"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::analyze_response()))
        .mount(&server)
        .await;

    cli()
        .args([
            "analyze",
            "The dark night was very dark.",
            "--output",
            "text",
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("concreteness"))
        // text format outputs diagnostics to stderr
        .stderr(predicate::str::contains("[warning]"))
        .stderr(predicate::str::contains("repetition"))
        .stderr(predicate::str::contains("[Heuristic 1.00]"))
        .stderr(predicate::str::contains("[id:d-001]"))
        .stderr(predicate::str::contains("ironprose rate"));
}

#[tokio::test]
async fn test_analyze_from_file() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/analyze"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::analyze_response()))
        .mount(&server)
        .await;

    // Use an existing fixture file
    cli()
        .args([
            "analyze",
            "--file",
            "tests/fixtures/sample.txt",
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("word_count"));
}

#[tokio::test]
async fn test_analyze_score_only() {
    let server = MockServer::start().await;

    let score_only_response = serde_json::json!({
        "score": {
            "concreteness": 0.75,
            "imagery_density": 0.62,
            "vocabulary_richness": 0.58,
            "sentence_variety": 0.70,
            "dialogue_balance": 0.45,
            "pacing_score": 0.68
        },
        "word_count": 6
    });

    Mock::given(method("POST"))
        .and(path("/api/analyze"))
        .respond_with(ResponseTemplate::new(200).set_body_json(score_only_response))
        .mount(&server)
        .await;

    cli()
        .args([
            "analyze",
            "The dark night was very dark.",
            "--score-only",
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("concreteness"));
}

// ── Compare Tests ──────────────────────────────────────────────

#[tokio::test]
async fn test_compare_inline() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/compare"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::compare_response()))
        .mount(&server)
        .await;

    cli()
        .args([
            "compare",
            "--original",
            "The dark night was very dark.",
            "--revised",
            "The shadowy night was dim and cold.",
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("fixed"))
        .stdout(predicate::str::contains("original_score"))
        .stdout(predicate::str::contains("revised_score"));
}

// ── List Rules Tests ───────────────────────────────────────────

#[tokio::test]
async fn test_list_rules() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/rules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::list_rules_response()))
        .mount(&server)
        .await;

    cli()
        .args(["list-rules", "--api-url", &server.uri()])
        .assert()
        .success()
        .stdout(predicate::str::contains("repetition"))
        .stdout(predicate::str::contains("passive_voice"))
        .stdout(predicate::str::contains("\"total\": 3"));
}

#[tokio::test]
async fn test_list_rules_text_output() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/rules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::list_rules_response()))
        .mount(&server)
        .await;

    cli()
        .args(["list-rules", "--output", "text", "--api-url", &server.uri()])
        .assert()
        .success()
        .stdout(predicate::str::contains("repetition  [style]"))
        .stdout(predicate::str::contains("passive_voice  [grammar]"))
        .stderr(predicate::str::contains("3 rule(s)"));
}

// ── Error Handling Tests ───────────────────────────────────────

#[tokio::test]
async fn test_error_402_payment_required() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/analyze"))
        .respond_with(
            ResponseTemplate::new(402)
                .set_body_json(serde_json::json!({"error": "Payment required"})),
        )
        .mount(&server)
        .await;

    cli()
        .args(["analyze", "Some text", "--api-url", &server.uri()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("402").or(predicate::str::contains("subscription")));
}

#[tokio::test]
async fn test_error_429_rate_limited() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/analyze"))
        .respond_with(
            ResponseTemplate::new(429).set_body_json(serde_json::json!({"error": "Rate limited"})),
        )
        .mount(&server)
        .await;

    cli()
        .args(["analyze", "Some text", "--api-url", &server.uri()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("429").or(predicate::str::contains("rate limit")));
}

#[tokio::test]
async fn test_error_500_server_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/analyze"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_json(serde_json::json!({"error": "Internal error"})),
        )
        .mount(&server)
        .await;

    cli()
        .args(["analyze", "Some text", "--api-url", &server.uri()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("500").or(predicate::str::contains("server error")));
}

#[tokio::test]
async fn test_error_401_unauthorized() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/analyze"))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(serde_json::json!({"error": "Unauthorized"})),
        )
        .mount(&server)
        .await;

    cli()
        .args(["analyze", "Some text", "--api-url", &server.uri()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("401").or(predicate::str::contains("authentication")));
}

// ── JSON Passthrough Tests ─────────────────────────────────────

#[tokio::test]
async fn test_analyze_json_passthrough() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/analyze"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::analyze_response()))
        .mount(&server)
        .await;

    cli()
        .args([
            "analyze",
            "--json",
            r#"{"text": "The dark night was very dark."}"#,
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("concreteness"));
}

#[tokio::test]
async fn test_compare_json_passthrough() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/compare"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::compare_response()))
        .mount(&server)
        .await;

    cli()
        .args([
            "compare",
            "--json",
            r#"{"original": "First draft.", "revised": "Second draft."}"#,
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("fixed"));
}

// ── Schema Introspection Tests ─────────────────────────────────

#[test]
fn test_schema_full_spec() {
    cli()
        .args(["schema"])
        .assert()
        .success()
        .stdout(predicate::str::contains("openapi"))
        .stdout(predicate::str::contains("paths"));
}

#[test]
fn test_schema_endpoint() {
    cli()
        .args(["schema", "analyze"])
        .assert()
        .success()
        .stdout(predicate::str::contains("/analyze"))
        .stdout(predicate::str::contains("POST"));
}

#[test]
fn test_schema_unknown_endpoint() {
    cli()
        .args(["schema", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown endpoint"));
}

// ── Locale Validation Tests ────────────────────────────────────

#[test]
fn test_analyze_invalid_locale_exits_1() {
    cli()
        .args(["analyze", "Some text.", "--locale", "klingon"])
        .assert()
        .code(1)
        .stderr(
            predicate::str::contains("unknown locale")
                .and(predicate::str::contains("accepted locales")),
        );
}

#[test]
fn test_analyze_invalid_locale_shows_valid_values() {
    cli()
        .args(["analyze", "Some text.", "--locale", "fr-FR"])
        .assert()
        .code(1)
        .stderr(
            predicate::str::contains("accepted locales")
                .and(predicate::str::contains("en-us"))
                .and(predicate::str::contains("en-any")),
        );
}

// ── Input Hardening Tests ──────────────────────────────────────

#[test]
fn test_reject_path_traversal() {
    cli()
        .args(["analyze", "--file", "../../etc/passwd"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("traversal"));
}

#[test]
fn test_reject_absolute_path() {
    cli()
        .args(["analyze", "--file", "/etc/passwd"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Absolute"));
}

// ── Rate Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_rate_basic() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/rate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::rate_response()))
        .mount(&server)
        .await;

    cli()
        .args([
            "rate",
            "--rule",
            "repetition",
            "--rating",
            "helpful",
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"))
        .stdout(predicate::str::contains("Feedback recorded"));
}

#[tokio::test]
async fn test_rate_json_passthrough() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/rate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::rate_response()))
        .mount(&server)
        .await;

    cli()
        .args([
            "rate",
            "--json",
            r#"{"rule":"repetition","rating":"false_positive","diagnostic_id":"d-001"}"#,
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));
}

#[tokio::test]
async fn test_rate_with_diagnostic_id() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/rate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::rate_response()))
        .mount(&server)
        .await;

    cli()
        .args([
            "rate",
            "--rule",
            "repetition",
            "--rating",
            "false_positive",
            "--diagnostic-id",
            "d-001",
            "--context",
            "Intentional repetition for emphasis",
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));
}

#[test]
fn test_rate_missing_required_args() {
    // Missing both --rule and --json should fail
    cli()
        .args(["rate", "--rating", "helpful"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--rule"));
}

// ── Insights Tests ─────────────────────────────────────────────

#[tokio::test]
async fn test_insights_basic() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/insights"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::insights_response()))
        .mount(&server)
        .await;

    cli()
        .args(["insights", "--api-url", &server.uri()])
        .assert()
        .success()
        .stdout(predicate::str::contains("repetition"))
        .stdout(predicate::str::contains("precision_proxy"))
        .stdout(predicate::str::contains("\"total\": 2"));
}

#[tokio::test]
async fn test_insights_with_filters() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/insights"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::insights_response()))
        .mount(&server)
        .await;

    cli()
        .args([
            "insights",
            "--since",
            "2024-01-01",
            "--until",
            "2024-12-31",
            "--genre",
            "fiction",
            "--work-id",
            "book-a",
            "--api-url",
            &server.uri(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("repetition"));
}

#[tokio::test]
async fn test_insights_401_unauthorized() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/insights"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(serde_json::json!({"error": "Authentication required"})),
        )
        .mount(&server)
        .await;

    cli()
        .args(["insights", "--api-url", &server.uri()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("401").or(predicate::str::contains("authentication")));
}
