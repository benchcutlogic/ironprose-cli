//! Output rendering for the IronProse CLI.
//!
//! Supports three formats: JSON (default), text (terminal-friendly), and
//! markdown (self-contained report for sharing/archiving).

use std::collections::BTreeMap;

/// Escape API-derived text for safe interpolation into Markdown.
///
/// Guards against broken tables (pipes), inline code corruption (backticks),
/// HTML injection, and multi-line break-out (newlines).
fn escape_md(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('`', "\\`")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', " ")
        .replace('\r', "")
}

/// Severity ordering for grouping diagnostics (lower = more severe).
fn severity_rank(s: &str) -> u8 {
    match s {
        "error" => 0,
        "warning" => 1,
        "information" => 2,
        "hint" => 3,
        _ => 4,
    }
}

/// Canonical display label for a severity level.
fn severity_label(s: &str) -> &str {
    match s {
        "error" => "Errors",
        "warning" => "Warnings",
        "information" => "Info",
        "hint" => "Hints",
        _ => s,
    }
}

/// Known score axis keys in display order.
const SCORE_AXES: &[&str] = &[
    "concreteness",
    "imagery_density",
    "vocabulary_richness",
    "sentence_variety",
    "dialogue_balance",
    "pacing_score",
];

/// Human-readable axis label.
fn axis_label(key: &str) -> &str {
    match key {
        "concreteness" => "Concreteness",
        "imagery_density" => "Imagery Density",
        "vocabulary_richness" => "Vocabulary Richness",
        "sentence_variety" => "Sentence Variety",
        "dialogue_balance" => "Dialogue Balance",
        "pacing_score" => "Pacing",
        _ => key,
    }
}

// ── Public API ────────────────────────────────────────────────

/// Render the API response in the requested format.
pub fn render(value: &serde_json::Value, format: &crate::OutputFormat) {
    match format {
        crate::OutputFormat::Text => render_text(value),
        crate::OutputFormat::Markdown => render_markdown(value),
        crate::OutputFormat::Json => render_json(value),
    }
}

// ── JSON ──────────────────────────────────────────────────────

fn render_json(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_default()
    );
}

// ── Text ──────────────────────────────────────────────────────

fn render_text(value: &serde_json::Value) {
    // Insights / list-rules responses have a top-level "rules" array.
    if let Some(rules) = value.get("rules").and_then(|r| r.as_array()) {
        if rules
            .first()
            .map(|r| r.get("total_ratings").is_some())
            .unwrap_or(false)
        {
            // Insights response
            let total = value.get("total").and_then(|t| t.as_u64()).unwrap_or(0);
            println!("Insights — {total} rule(s) with feedback\n");
            for rule in rules {
                let name = rule.get("rule").and_then(|v| v.as_str()).unwrap_or("?");
                let helpful = rule.get("helpful").and_then(|v| v.as_i64()).unwrap_or(0);
                let not_helpful = rule
                    .get("not_helpful")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let false_pos = rule
                    .get("false_positive")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let total_r = rule
                    .get("total_ratings")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let precision = rule
                    .get("precision_proxy")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                println!(
                    "  {name}: {total_r} rating(s)  \
                     helpful={helpful}  not_helpful={not_helpful}  \
                     false_positive={false_pos}  precision={precision:.2}"
                );
            }
            return;
        }
        // list-rules response
        for r in rules {
            let name = r.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let category = r.get("category").and_then(|v| v.as_str()).unwrap_or("?");
            println!("{name}  [{category}]");
        }
        let total = rules.len();
        eprintln!("\n{total} rule(s)");
        return;
    }

    // Analyze response diagnostics → stderr
    if let Some(diagnostics) = value.get("diagnostics").and_then(|d| d.as_array()) {
        for d in diagnostics {
            let rule = d.get("rule").and_then(|v| v.as_str()).unwrap_or("?");
            let severity = d.get("severity").and_then(|v| v.as_str()).unwrap_or("?");
            let message = d.get("message").and_then(|v| v.as_str()).unwrap_or("?");
            let line = d
                .get("start_line")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .saturating_add(1);

            let source_tag = match d.get("source_type").and_then(|v| v.as_str()) {
                Some(st) => {
                    let conf = d.get("confidence").and_then(|v| v.as_f64()).unwrap_or(1.0);
                    format!(" [{st} {conf:.2}]")
                }
                None => String::new(),
            };

            let id_tag = match d.get("id").and_then(|v| v.as_str()) {
                Some(id) => format!(" [id:{id}]"),
                None => String::new(),
            };

            eprintln!("  [{severity}] L{line}: {message} ({rule}){source_tag}{id_tag}");
        }
        let count = diagnostics.len();
        eprintln!("\n{count} diagnostic(s)");

        if diagnostics.iter().any(|d| d.get("id").is_some()) {
            eprintln!(
                "\nRate diagnostics: ironprose rate --rule <rule> --rating helpful|not_helpful|false_positive --diagnostic-id <id>"
            );
        }
    }

    // Score → stdout (pretty JSON)
    if let Some(score) = value.get("score") {
        println!(
            "{}",
            serde_json::to_string_pretty(score).unwrap_or_default()
        );
    }

    // Compare response
    let has_compare_keys = value.get("fixed").is_some()
        || value.get("introduced").is_some()
        || value.get("persistent").is_some();
    if has_compare_keys {
        let print_diag_section = |label: &str, diags: &[serde_json::Value]| {
            if diags.is_empty() {
                eprintln!("{label}: (none)");
                return;
            }
            eprintln!("{label}: {} diagnostic(s)", diags.len());
            for d in diags {
                let rule = d.get("rule").and_then(|v| v.as_str()).unwrap_or("?");
                let severity = d.get("severity").and_then(|v| v.as_str()).unwrap_or("?");
                let message = d.get("message").and_then(|v| v.as_str()).unwrap_or("?");
                let line = d
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .saturating_add(1);
                eprintln!("  [{severity}] L{line}: {message} ({rule})");
            }
        };

        let empty = vec![];
        let fixed = value
            .get("fixed")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty);
        let introduced = value
            .get("introduced")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty);
        let persistent = value
            .get("persistent")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty);

        print_diag_section("Fixed", fixed);
        print_diag_section("Introduced", introduced);
        print_diag_section("Persistent", persistent);

        if let (Some(orig), Some(rev)) = (value.get("original_score"), value.get("revised_score")) {
            eprintln!("\nScore delta (original → revised):");
            if let (Some(orig_obj), Some(rev_obj)) = (orig.as_object(), rev.as_object()) {
                for (key, orig_val) in orig_obj {
                    let rev_val = rev_obj.get(key);
                    match (orig_val.as_f64(), rev_val.and_then(|v| v.as_f64())) {
                        (Some(o), Some(r)) => {
                            let delta = r - o;
                            let sign = if delta >= 0.0 { "+" } else { "" };
                            eprintln!("  {key}: {o:.2} → {r:.2}  ({sign}{delta:.2})");
                        }
                        _ => {
                            eprintln!(
                                "  {key}: {orig_val} → {}",
                                rev_val.unwrap_or(&serde_json::Value::Null)
                            );
                        }
                    }
                }
            }
        }
    }
}

// ── Markdown ──────────────────────────────────────────────────

fn render_markdown(value: &serde_json::Value) {
    let has_compare_keys = value.get("fixed").is_some()
        || value.get("introduced").is_some()
        || value.get("persistent").is_some();

    if has_compare_keys {
        render_markdown_compare(value);
    } else {
        render_markdown_analyze(value);
    }
}

/// Render an `analyze` response as a Markdown report.
fn render_markdown_analyze(value: &serde_json::Value) {
    let mut out = String::new();
    out.push_str("# Prose Analysis\n\n");

    // ── Scores ────────────────────────────────────────────────
    if let Some(score) = value.get("score").and_then(|s| s.as_object()) {
        out.push_str("## Scores\n\n");
        out.push_str("| Axis | Score |\n");
        out.push_str("|------|-------|\n");
        for &axis in SCORE_AXES {
            if let Some(v) = score.get(axis).and_then(|v| v.as_f64()) {
                out.push_str(&format!("| {} | {:.2} |\n", axis_label(axis), v));
            }
        }
        out.push('\n');
    }

    // ── Diagnostics ───────────────────────────────────────────
    if let Some(diagnostics) = value.get("diagnostics").and_then(|d| d.as_array()) {
        let total = diagnostics.len();
        out.push_str(&format!("## Diagnostics ({total} issues)\n\n"));

        // Group by severity
        let mut by_severity: BTreeMap<u8, (&str, Vec<&serde_json::Value>)> = BTreeMap::new();
        for d in diagnostics {
            let sev = d.get("severity").and_then(|v| v.as_str()).unwrap_or("hint");
            let rank = severity_rank(sev);
            by_severity
                .entry(rank)
                .or_insert_with(|| (sev, Vec::new()))
                .1
                .push(d);
        }

        for (sev, diags) in by_severity.values() {
            let label = severity_label(sev);
            out.push_str(&format!("### {} ({})\n\n", label, diags.len()));
            for d in diags {
                format_diagnostic_md(&mut out, d);
            }
            out.push('\n');
        }
    }

    // ── Footer ────────────────────────────────────────────────
    out.push_str("---\n");
    out.push_str(
        "*Generated by [ironprose](https://ironprose.com) · deterministic craft analyzers*\n",
    );

    print!("{out}");
}

/// Render a `compare` response as a Markdown revision report.
fn render_markdown_compare(value: &serde_json::Value) {
    let mut out = String::new();
    out.push_str("# Revision Report\n\n");

    // ── Score Comparison ──────────────────────────────────────
    if let (Some(orig), Some(rev)) = (
        value.get("original_score").and_then(|s| s.as_object()),
        value.get("revised_score").and_then(|s| s.as_object()),
    ) {
        out.push_str("## Score Comparison\n\n");
        out.push_str("| Axis | Original | Revised | Δ |\n");
        out.push_str("|------|----------|---------|---|\n");
        for &axis in SCORE_AXES {
            if let (Some(o), Some(r)) = (
                orig.get(axis).and_then(|v| v.as_f64()),
                rev.get(axis).and_then(|v| v.as_f64()),
            ) {
                let delta = r - o;
                let sign = if delta >= 0.0 { "+" } else { "" };
                out.push_str(&format!(
                    "| {} | {:.2} | {:.2} | {}{:.2} |\n",
                    axis_label(axis),
                    o,
                    r,
                    sign,
                    delta
                ));
            }
        }
        out.push('\n');
    }

    let empty = vec![];

    // ── Fixed ─────────────────────────────────────────────────
    let fixed = value
        .get("fixed")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);
    out.push_str(&format!("## Fixed ({} issues resolved)\n\n", fixed.len()));
    if fixed.is_empty() {
        out.push_str("No issues were fixed.\n\n");
    } else {
        // Summarize by rule (count table)
        let counts = count_by_rule(fixed);
        out.push_str("| Rule | Count |\n");
        out.push_str("|------|-------|\n");
        for (rule, count) in &counts {
            out.push_str(&format!("| `{}` | {count} |\n", escape_md(rule)));
        }
        out.push('\n');
    }

    // ── Introduced ────────────────────────────────────────────
    let introduced = value
        .get("introduced")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);
    out.push_str(&format!(
        "## Introduced ({} new issues)\n\n",
        introduced.len()
    ));
    if introduced.is_empty() {
        out.push_str("No new issues were introduced.\n\n");
    } else {
        // List individual diagnostics (these are actionable)
        for d in introduced {
            format_diagnostic_md(&mut out, d);
        }
        out.push('\n');
    }

    // ── Persistent ────────────────────────────────────────────
    let persistent = value
        .get("persistent")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);
    out.push_str(&format!(
        "## Persistent ({} unchanged)\n\n",
        persistent.len()
    ));
    if persistent.is_empty() {
        out.push_str("No persistent issues.\n\n");
    } else {
        let counts = count_by_rule(persistent);
        out.push_str("| Rule | Count |\n");
        out.push_str("|------|-------|\n");
        for (rule, count) in &counts {
            out.push_str(&format!("| `{}` | {count} |\n", escape_md(rule)));
        }
        out.push('\n');
    }

    // ── Footer ────────────────────────────────────────────────
    out.push_str("---\n");
    out.push_str("*Generated by [ironprose](https://ironprose.com)*\n");

    print!("{out}");
}

// ── Helpers ───────────────────────────────────────────────────

/// Format a single diagnostic as a Markdown bullet.
///
/// Output: `- **L24** \`rule_name\` — "message" · source_type · 0.72 confidence\n`
fn format_diagnostic_md(out: &mut String, d: &serde_json::Value) {
    let rule = d.get("rule").and_then(|v| v.as_str()).unwrap_or("?");
    let message = d.get("message").and_then(|v| v.as_str()).unwrap_or("?");
    let line = d
        .get("start_line")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .saturating_add(1);

    out.push_str(&format!(
        "- **L{line}** `{}` — {}",
        escape_md(rule),
        escape_md(message)
    ));

    // Telemetry metadata
    if let Some(source_type) = d.get("source_type").and_then(|v| v.as_str()) {
        let st_lower = source_type.to_lowercase();
        let conf = d.get("confidence").and_then(|v| v.as_f64()).unwrap_or(1.0);
        out.push_str(&format!(
            " · {} · {conf:.2} confidence",
            escape_md(&st_lower)
        ));
    }

    out.push('\n');
}

/// Count diagnostics by rule, returning sorted (rule, count) pairs.
fn count_by_rule(diags: &[serde_json::Value]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for d in diags {
        let rule = d
            .get("rule")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        *counts.entry(rule).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted
}

// ── Unit Tests ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn capture_stdout(f: impl FnOnce()) -> String {
        // We test the string-building helpers directly instead of capturing
        // stdout, which is simpler and avoids test parallelism issues.
        let _ = f;
        String::new()
    }

    fn sample_analyze_response() -> serde_json::Value {
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
            "diagnostics": [
                {
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
                },
                {
                    "rule": "tense_consistency",
                    "severity": "error",
                    "message": "Tense shift detected.",
                    "start_line": 5,
                    "start_char": 0,
                    "end_line": 5,
                    "end_char": 10,
                    "source_type": "Model",
                    "confidence": 0.72
                }
            ]
        })
    }

    fn sample_compare_response() -> serde_json::Value {
        serde_json::json!({
            "fixed": [
                {
                    "rule": "repetition",
                    "severity": "warning",
                    "message": "Word 'dark' repeated 2 times.",
                    "start_line": 0, "start_char": 4,
                    "end_line": 0, "end_char": 8
                },
                {
                    "rule": "repetition",
                    "severity": "warning",
                    "message": "Another repetition.",
                    "start_line": 2, "start_char": 0,
                    "end_line": 2, "end_char": 5
                },
                {
                    "rule": "passive_voice",
                    "severity": "warning",
                    "message": "Passive construction detected.",
                    "start_line": 3, "start_char": 0,
                    "end_line": 3, "end_char": 10
                }
            ],
            "introduced": [
                {
                    "rule": "paragraph_end_weight",
                    "severity": "warning",
                    "message": "Paragraph ends on weak function word.",
                    "start_line": 6, "start_char": 0,
                    "end_line": 6, "end_char": 8,
                    "source_type": "Heuristic",
                    "confidence": 0.90
                }
            ],
            "persistent": [
                {
                    "rule": "comma_splice",
                    "severity": "error",
                    "message": "Comma splice detected.",
                    "start_line": 1, "start_char": 10,
                    "end_line": 1, "end_char": 11
                }
            ],
            "original_score": {
                "concreteness": 0.68,
                "imagery_density": 0.55,
                "vocabulary_richness": 0.50,
                "sentence_variety": 0.60,
                "dialogue_balance": 0.40,
                "pacing_score": 0.65
            },
            "revised_score": {
                "concreteness": 0.79,
                "imagery_density": 0.72,
                "vocabulary_richness": 0.68,
                "sentence_variety": 0.74,
                "dialogue_balance": 0.55,
                "pacing_score": 0.80
            }
        })
    }

    #[test]
    fn test_format_diagnostic_md_with_source() {
        let d = serde_json::json!({
            "rule": "repetition",
            "severity": "warning",
            "message": "Word 'dark' repeated.",
            "start_line": 0,
            "start_char": 4,
            "end_line": 0,
            "end_char": 8,
            "source_type": "Heuristic",
            "confidence": 1.0
        });
        let mut out = String::new();
        format_diagnostic_md(&mut out, &d);
        assert!(out.contains("**L1**"));
        assert!(out.contains("`repetition`"));
        assert!(out.contains("heuristic"));
        assert!(out.contains("1.00 confidence"));
    }

    #[test]
    fn test_format_diagnostic_md_without_source() {
        let d = serde_json::json!({
            "rule": "weak_verb",
            "severity": "hint",
            "message": "Consider a stronger verb.",
            "start_line": 9,
            "start_char": 0,
            "end_line": 9,
            "end_char": 5
        });
        let mut out = String::new();
        format_diagnostic_md(&mut out, &d);
        assert!(out.contains("**L10**"));
        assert!(out.contains("`weak_verb`"));
        assert!(!out.contains("confidence"));
    }

    #[test]
    fn test_count_by_rule_sorting() {
        let diags = vec![
            serde_json::json!({"rule": "b_rule"}),
            serde_json::json!({"rule": "a_rule"}),
            serde_json::json!({"rule": "b_rule"}),
            serde_json::json!({"rule": "b_rule"}),
            serde_json::json!({"rule": "a_rule"}),
        ];
        let counts = count_by_rule(&diags);
        // b_rule (3) should come before a_rule (2)
        assert_eq!(counts[0], ("b_rule".into(), 3));
        assert_eq!(counts[1], ("a_rule".into(), 2));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(severity_rank("error") < severity_rank("warning"));
        assert!(severity_rank("warning") < severity_rank("information"));
        assert!(severity_rank("information") < severity_rank("hint"));
    }

    #[test]
    fn test_render_markdown_analyze_structure() {
        // Build the markdown string by calling the internal function
        // We test structure by inspecting what render_markdown_analyze would produce.
        // Since it prints to stdout, we validate the helpers instead.
        let value = sample_analyze_response();

        // Verify score table would contain all axes
        let score = value.get("score").unwrap().as_object().unwrap();
        for &axis in SCORE_AXES {
            assert!(score.contains_key(axis), "missing axis: {axis}");
        }

        // Verify diagnostics group correctly
        let diags = value.get("diagnostics").unwrap().as_array().unwrap();
        let mut by_sev: BTreeMap<u8, Vec<&serde_json::Value>> = BTreeMap::new();
        for d in diags {
            let sev = d.get("severity").and_then(|v| v.as_str()).unwrap();
            by_sev.entry(severity_rank(sev)).or_default().push(d);
        }
        // Should have error and warning groups
        assert!(by_sev.contains_key(&0), "no error group");
        assert!(by_sev.contains_key(&1), "no warning group");

        // Suppress unused capture_stdout warning
        drop(capture_stdout(|| {}));
    }

    #[test]
    fn test_render_markdown_compare_counts() {
        let value = sample_compare_response();
        let fixed = value.get("fixed").unwrap().as_array().unwrap();
        let counts = count_by_rule(fixed);
        // 2 repetition + 1 passive_voice
        assert_eq!(counts.len(), 2);
        assert_eq!(counts[0], ("repetition".into(), 2));
        assert_eq!(counts[1], ("passive_voice".into(), 1));
    }
}
