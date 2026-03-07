//! Typed API request and response types for the IronProse REST API.
//!
//! Derived from the OpenAPI 3.1 spec at `/api/openapi.json`.
//! These types provide compile-time safety for API interactions and
//! serve as the source of truth for mock server test fixtures.

use serde::{Deserialize, Serialize};

// ── Request Types ──────────────────────────────────────────────

/// Parameters for `POST /api/analyze`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeParams {
    /// The text to analyze.
    pub text: String,

    /// Language locale: "en-us", "en-gb", or "en-any" (default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// Only run these specific rules. If omitted, all rules run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,

    /// Minimum severity: "error", "warning", "information", or "hint".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity_min: Option<String>,

    /// If true, only return scores (no diagnostics or profile).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_only: Option<bool>,

    /// Per-request analyzer configuration overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

/// Parameters for `POST /api/compare`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareParams {
    /// The original text (before edits).
    pub original: String,

    /// The revised text (after edits).
    pub revised: String,

    /// Language locale for spell checking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// Per-request analyzer configuration overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

/// Parameters for `POST /api/rate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateParams {
    /// The rule (analyzer) that produced the diagnostic.
    pub rule: String,

    /// Rating: "helpful", "not_helpful", or "false_positive".
    pub rating: String,

    /// Optional context about why the rating was given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Optional diagnostic ID from the analyze response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_id: Option<String>,

    /// Optional: the input text that was analyzed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_text: Option<String>,

    /// Optional: the diagnostic message text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Optional: severity of the diagnostic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,

    /// Optional: start line of the diagnostic span.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<i32>,

    /// Optional: start character within the start line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_char: Option<i32>,

    /// Optional: end line of the diagnostic span.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i32>,

    /// Optional: end character within the end line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_char: Option<i32>,
}

// ── Response Types ─────────────────────────────────────────────

/// Response from `POST /api/analyze`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeResult {
    /// The 6-axis quality score.
    pub score: ScoreResult,

    /// Total word count.
    pub word_count: u64,

    /// Diagnostics (omitted when `score_only` is true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<DiagnosticItem>>,

    /// Statistical style profile (omitted when `score_only` is true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<ProfileResult>,

    /// True if the input was truncated due to free-tier limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

/// Response from `POST /api/compare`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    /// Diagnostics present in the original but absent in the revised (fixed).
    pub fixed: Vec<DiagnosticItem>,

    /// Diagnostics present in the revised but absent in the original (introduced).
    pub introduced: Vec<DiagnosticItem>,

    /// Diagnostics present in both versions.
    pub persistent: Vec<DiagnosticItem>,

    /// Score for the original text.
    pub original_score: ScoreResult,

    /// Score for the revised text.
    pub revised_score: ScoreResult,
}

/// Response from `POST /api/rate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateResult {
    pub status: String,
    pub message: String,
}

/// Response from `GET /api/rules`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRulesResult {
    pub rules: Vec<RuleInfo>,
    pub total: u64,
}

/// Response from `GET /api/entitlement`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementResult {
    /// The user's tier: "pro" or "free".
    pub tier: String,

    /// Whether the API key is valid.
    pub valid: bool,
}

/// Response from `GET /api/insights`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsResult {
    /// Per-rule feedback statistics, sorted by total ratings descending.
    pub rules: Vec<RuleInsightItem>,

    /// Total number of rules with ratings.
    pub total: u64,
}

/// Per-rule feedback statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInsightItem {
    /// The analyzer rule name.
    pub rule: String,

    /// Total number of ratings for this rule.
    pub total_ratings: i64,

    /// Number of "helpful" ratings.
    pub helpful: i64,

    /// Number of "not_helpful" ratings.
    pub not_helpful: i64,

    /// Number of "false_positive" ratings.
    pub false_positive: i64,

    /// Precision proxy: helpful / (helpful + false_positive). 0 if no data.
    pub precision_proxy: f64,
}

/// Error response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}

// ── Shared Types ───────────────────────────────────────────────

/// A single diagnostic finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticItem {
    /// The rule (analyzer) that produced this diagnostic.
    pub rule: String,

    /// Severity level.
    pub severity: String,

    /// Human-readable message.
    pub message: String,

    /// Start line (0-indexed).
    pub start_line: i32,

    /// Start character within the start line.
    pub start_char: i32,

    /// End line (0-indexed).
    pub end_line: i32,

    /// End character within the end line.
    pub end_char: i32,

    /// Unique identifier (use with the `rate` tool).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Whether this diagnostic was produced by a Heuristic, Model, or Hybrid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,

    /// Confidence score (0.0–1.0). Heuristics default to 1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

/// 6-axis quality score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResult {
    pub concreteness: f64,
    pub imagery_density: f64,
    pub vocabulary_richness: f64,
    pub sentence_variety: f64,
    pub dialogue_balance: f64,
    pub pacing_score: f64,
}

/// Statistical style profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResult {
    pub avg_sentence_length: f64,
    pub sentence_length_variance: f64,
    pub min_sentence_length: u64,
    pub max_sentence_length: u64,
    pub total_sentences: u64,
    pub total_paragraphs: u64,
    pub dialogue_line_ratio: f64,
    pub avg_paragraph_length_words: f64,
}

/// Info about a single analysis rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInfo {
    pub name: String,
    pub category: String,
}

// ── Test Fixtures ──────────────────────────────────────────────

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    pub fn analyze_result() -> AnalyzeResult {
        AnalyzeResult {
            score: score_result(),
            word_count: 42,
            diagnostics: Some(vec![diagnostic_item()]),
            profile: Some(profile_result()),
            truncated: None,
        }
    }

    pub fn compare_result() -> CompareResult {
        CompareResult {
            fixed: vec![diagnostic_item()],
            introduced: vec![],
            persistent: vec![],
            original_score: score_result(),
            revised_score: ScoreResult {
                concreteness: 0.85,
                imagery_density: 0.72,
                vocabulary_richness: 0.68,
                sentence_variety: 0.74,
                dialogue_balance: 0.55,
                pacing_score: 0.80,
            },
        }
    }

    pub fn list_rules_result() -> ListRulesResult {
        ListRulesResult {
            rules: vec![
                RuleInfo {
                    name: "repetition".into(),
                    category: "style".into(),
                },
                RuleInfo {
                    name: "passive_voice".into(),
                    category: "grammar".into(),
                },
                RuleInfo {
                    name: "weak_verb".into(),
                    category: "style".into(),
                },
            ],
            total: 3,
        }
    }

    pub fn rate_result() -> RateResult {
        RateResult {
            status: "ok".into(),
            message: "Feedback recorded. Thank you!".into(),
        }
    }

    pub fn entitlement_result() -> EntitlementResult {
        EntitlementResult {
            tier: "free".into(),
            valid: true,
        }
    }

    pub fn insights_result() -> InsightsResult {
        InsightsResult {
            rules: vec![
                RuleInsightItem {
                    rule: "repetition".into(),
                    total_ratings: 10,
                    helpful: 7,
                    not_helpful: 2,
                    false_positive: 1,
                    precision_proxy: 0.875,
                },
                RuleInsightItem {
                    rule: "passive_voice".into(),
                    total_ratings: 5,
                    helpful: 3,
                    not_helpful: 1,
                    false_positive: 1,
                    precision_proxy: 0.75,
                },
            ],
            total: 2,
        }
    }

    pub fn score_result() -> ScoreResult {
        ScoreResult {
            concreteness: 0.75,
            imagery_density: 0.62,
            vocabulary_richness: 0.58,
            sentence_variety: 0.70,
            dialogue_balance: 0.45,
            pacing_score: 0.68,
        }
    }

    pub fn profile_result() -> ProfileResult {
        ProfileResult {
            avg_sentence_length: 14.5,
            sentence_length_variance: 8.2,
            min_sentence_length: 3,
            max_sentence_length: 32,
            total_sentences: 12,
            total_paragraphs: 4,
            dialogue_line_ratio: 0.25,
            avg_paragraph_length_words: 52.0,
        }
    }

    pub fn diagnostic_item() -> DiagnosticItem {
        DiagnosticItem {
            rule: "repetition".into(),
            severity: "warning".into(),
            message: "Word 'dark' repeated 2 times in close proximity.".into(),
            start_line: 0,
            start_char: 4,
            end_line: 0,
            end_char: 8,
            id: Some("d-001".into()),
            source_type: Some("Heuristic".into()),
            confidence: Some(1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_result_roundtrip() {
        let result = fixtures::analyze_result();
        let json = serde_json::to_string(&result).unwrap();
        let parsed: AnalyzeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.word_count, 42);
        assert_eq!(parsed.diagnostics.unwrap().len(), 1);
    }

    #[test]
    fn test_compare_result_roundtrip() {
        let result = fixtures::compare_result();
        let json = serde_json::to_string(&result).unwrap();
        let parsed: CompareResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.fixed.len(), 1);
        assert!(parsed.introduced.is_empty());
    }

    #[test]
    fn test_list_rules_result_roundtrip() {
        let result = fixtures::list_rules_result();
        let json = serde_json::to_string(&result).unwrap();
        let parsed: ListRulesResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total, 3);
        assert_eq!(parsed.rules[0].name, "repetition");
    }

    #[test]
    fn test_score_only_omits_diagnostics() {
        let result = AnalyzeResult {
            score: fixtures::score_result(),
            word_count: 10,
            diagnostics: None,
            profile: None,
            truncated: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("diagnostics"));
        assert!(!json.contains("profile"));
    }
}
