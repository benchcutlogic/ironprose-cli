#[allow(dead_code)]
mod client;
mod error;
mod input;
mod schema;
#[allow(dead_code)]
mod types;

use clap::{Parser, Subcommand};
use client::ApiClient;

#[derive(Parser)]
#[command(
    name = "ironprose",
    version,
    about = "IronProse CLI — prose analysis tools for writers"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// IronProse API base URL
    #[arg(
        long,
        global = true,
        env = "IRONPROSE_API_URL",
        default_value = "https://prose-mcp.fly.dev"
    )]
    api_url: String,

    /// API key for authenticated access (optional, free tier available)
    #[arg(long, global = true, env = "IRONPROSE_API_KEY")]
    api_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze prose text for style, grammar, and craft issues
    Analyze {
        /// Text to analyze (reads from stdin if not provided)
        text: Option<String>,

        /// Read input from a file
        #[arg(short, long)]
        file: Option<String>,

        /// Raw JSON payload (sent directly to the API, bypasses other flags)
        #[arg(long, conflicts_with_all = ["text", "file", "score_only", "rules", "severity_min", "genre", "locale"])]
        json: Option<String>,

        /// Only output scores (no diagnostics)
        #[arg(long)]
        score_only: bool,

        /// Only run specific rules (comma-separated)
        #[arg(long, value_delimiter = ',')]
        rules: Option<Vec<String>>,

        /// Minimum severity: error, warning, information, hint
        #[arg(long)]
        severity_min: Option<String>,

        /// Genre context for analysis (e.g. fiction, nonfiction, academic)
        #[arg(long)]
        genre: Option<String>,

        /// Locale for language-specific rules (e.g. en-US, en-GB)
        #[arg(long)]
        locale: Option<String>,

        /// Output format: json (default), or text
        #[arg(short, long, default_value = "json")]
        output: String,
    },

    /// Compare original and revised text
    Compare {
        /// Original text (or use --original-file)
        #[arg(long)]
        original: Option<String>,

        /// Revised text (or use --revised-file)
        #[arg(long)]
        revised: Option<String>,

        /// Read original from file
        #[arg(long)]
        original_file: Option<String>,

        /// Read revised from file
        #[arg(long)]
        revised_file: Option<String>,

        /// Raw JSON payload (sent directly to the API, bypasses other flags)
        #[arg(long, conflicts_with_all = ["original", "revised", "original_file", "revised_file", "genre", "locale"])]
        json: Option<String>,

        /// Genre context for comparison (e.g. fiction, nonfiction)
        #[arg(long, help = "Genre context for comparison (e.g. fiction, nonfiction)")]
        genre: Option<String>,

        /// Locale for language-specific rules (e.g. en-US, en-GB)
        #[arg(long, help = "Locale for language-specific rules (e.g. en-US, en-GB)")]
        locale: Option<String>,

        /// Output format: json (default), or text
        #[arg(short, long, default_value = "json")]
        output: String,
    },

    /// List all available analysis rules
    ListRules {
        /// Output format: json (default), or text
        #[arg(short, long, default_value = "json")]
        output: String,
    },

    /// Dump the API schema for an endpoint (agent introspection)
    ///
    /// Examples:
    ///   ironprose schema analyze
    ///   ironprose schema compare
    ///   ironprose schema rate
    ///   ironprose schema list-rules
    ///   ironprose schema insights
    ///   ironprose schema          # dumps full OpenAPI spec
    Schema {
        /// Endpoint name: analyze, compare, rate, list-rules, entitlement, insights
        endpoint: Option<String>,
    },

    /// Get aggregate feedback insights per analyzer rule
    ///
    /// Returns per-rule counts of helpful, not_helpful, and false_positive
    /// ratings, plus a precision proxy. Requires an API key.
    Insights {
        /// Only include ratings on or after this date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Only include ratings before this date (YYYY-MM-DD)
        #[arg(long)]
        until: Option<String>,

        /// Filter by genre (prefix match: 'fiction' matches 'fiction:literary')
        #[arg(long)]
        genre: Option<String>,

        /// Filter by work identifier
        #[arg(long)]
        work_id: Option<String>,

        /// Output format: json (default), or text
        #[arg(short, long, default_value = "json")]
        output: String,
    },

    /// Rate a diagnostic as helpful, not_helpful, or false_positive
    ///
    /// Agents: prefer --json for full API control.
    /// Humans: use --rule and --rating convenience flags.
    Rate {
        /// Rule that produced the diagnostic
        #[arg(long)]
        rule: Option<String>,

        /// Rating: helpful, not_helpful, or false_positive
        #[arg(long)]
        rating: Option<String>,

        /// Raw JSON payload (sent directly to the API, bypasses other flags)
        #[arg(long, conflicts_with_all = ["rule", "rating", "context", "diagnostic_id"])]
        json: Option<String>,

        /// Why this rating was given (free-text context)
        #[arg(long)]
        context: Option<String>,

        /// Diagnostic ID from the analyze response
        #[arg(long)]
        diagnostic_id: Option<String>,
    },
}

/// Normalize a locale string to its canonical API form.
/// Accepts case-insensitive and underscore/hyphen variants.
/// Returns `Some("en-us")`, `Some("en-gb")`, or `Some("en-any")`, or `None` for unknown values.
fn normalize_locale(locale: &str) -> Option<&'static str> {
    let lower = locale.to_lowercase().replace('_', "-");
    match lower.as_str() {
        "en-us" => Some("en-us"),
        "en-gb" => Some("en-gb"),
        "en-any" => Some("en-any"),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let api_url = cli.api_url.clone();
    let client = ApiClient::new(cli.api_url, cli.api_key);

    match cli.command {
        Commands::Analyze {
            text,
            file,
            json,
            score_only,
            rules,
            severity_min,
            genre,
            locale,
            output,
        } => {
            let args = if let Some(raw) = json {
                // Raw JSON passthrough — send directly to API
                input::validate_json_input(&raw)?
            } else {
                let input_text = resolve_input(text, file.as_deref()).await?;
                input::validate_text_input(&input_text)?;

                let mut args = serde_json::json!({ "text": input_text });
                if score_only {
                    args["score_only"] = serde_json::json!(true);
                }
                if let Some(rules) = rules {
                    args["rules"] = serde_json::json!(rules);
                }
                if let Some(sev) = severity_min {
                    args["severity_min"] = serde_json::json!(sev);
                }
                if let Some(ref l) = locale {
                    input::validate_locale(l)?;
                }
                if let Some(g) = genre {
                    args["genre"] = serde_json::json!(g);
                }
                if let Some(ref l) = locale {
                    let canonical = normalize_locale(l);
                    match canonical {
                        Some(c) => args["locale"] = serde_json::json!(c),
                        None => {
                            eprintln!(
                                "error: unknown locale {:?}.\naccepted locales: en-us, en-gb, en-any (and case/separator variants)",
                                l
                            );
                            std::process::exit(1);
                        }
                    }
                }
                args
            };

            let result = client.call_remote("analyze", args).await?;
            print_output(&result, &output);
        }

        Commands::Compare {
            original,
            revised,
            original_file,
            revised_file,
            json,
            genre,
            locale,
            output,
        } => {
            let args = if let Some(raw) = json {
                input::validate_json_input(&raw)?
            } else {
                let orig = resolve_input(original, original_file.as_deref()).await?;
                let rev = resolve_input(revised, revised_file.as_deref()).await?;
                input::validate_text_input(&orig)?;
                input::validate_text_input(&rev)?;

                if let Some(ref l) = locale {
                    input::validate_locale(l)?;
                }

                let mut args = serde_json::json!({
                    "original": orig,
                    "revised": rev,
                });
                if let Some(g) = genre {
                    args["genre"] = serde_json::json!(g);
                }
                if let Some(l) = locale {
                    args["locale"] = serde_json::json!(l);
                }
                args
            };

            let mut result = client.call_remote("compare", args).await?;
            dedup_compare_result(&mut result);
            print_output(&result, &output);
        }

        Commands::ListRules { output } => {
            let result = client
                .call_remote("list_rules", serde_json::json!({}))
                .await?;
            print_output(&result, &output);
        }

        Commands::Schema { endpoint } => {
            let spec = schema::full_spec(&api_url).await;
            let output = match endpoint {
                Some(name) => schema::endpoint_schema(&spec, &name)?,
                None => spec,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&output).unwrap_or_default()
            );
        }

        Commands::Insights {
            since,
            until,
            genre,
            work_id,
            output,
        } => {
            let result = client
                .call_insights(
                    since.as_deref(),
                    until.as_deref(),
                    genre.as_deref(),
                    work_id.as_deref(),
                )
                .await?;
            print_output(&result, &output);
        }

        Commands::Rate {
            rule,
            rating,
            json,
            context,
            diagnostic_id,
        } => {
            let args = if let Some(raw) = json {
                input::validate_json_input(&raw)?
            } else {
                let rule =
                    rule.ok_or("--rule is required (or use --json for raw payload passthrough)")?;
                let rating = rating
                    .ok_or("--rating is required: helpful, not_helpful, or false_positive")?;
                let mut args = serde_json::json!({
                    "rule": rule,
                    "rating": rating,
                });
                if let Some(ctx) = context {
                    args["context"] = serde_json::json!(ctx);
                }
                if let Some(did) = diagnostic_id {
                    args["diagnostic_id"] = serde_json::json!(did);
                }
                args
            };

            let result = client.call_remote("rate", args).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&result).unwrap_or_default()
            );
        }
    }

    Ok(())
}

/// Deduplicate diagnostics in a compare API response.
///
/// The `introduced` and `fixed` arrays may contain the same diagnostic
/// multiple times (identical `id` + char offsets). This function removes
/// duplicates, keeping the first occurrence of each unique diagnostic
/// identified by `(id, start_line, start_char, end_line, end_char, rule)`.
///
/// When `id` is `None` (the API may omit it), two different diagnostics
/// that target the same span but have different `rule` values would
/// otherwise be incorrectly collapsed. Including `rule` in the key ensures
/// distinct diagnostics on the same span are preserved regardless of
/// whether `id` is present.
fn dedup_compare_result(result: &mut serde_json::Value) {
    for key in ["introduced", "fixed"] {
        if let Some(arr) = result.get_mut(key).and_then(|v| v.as_array_mut()) {
            let mut seen = std::collections::HashSet::new();
            arr.retain(|diag| {
                let id = diag.get("id").and_then(|v| v.as_str()).map(str::to_owned);
                // When `id` is absent the span alone is not sufficient to
                // identify a unique diagnostic — include `rule` so that two
                // findings at the same location but with different rules are
                // not incorrectly deduplicated.
                let rule_fallback = if id.is_none() {
                    diag.get("rule").and_then(|v| v.as_str()).map(str::to_owned)
                } else {
                    None
                };
                let dedup_key = (
                    id,
                    diag.get("start_line").and_then(|v| v.as_i64()),
                    diag.get("start_char").and_then(|v| v.as_i64()),
                    diag.get("end_line").and_then(|v| v.as_i64()),
                    diag.get("end_char").and_then(|v| v.as_i64()),
                    rule_fallback,
                );
                seen.insert(dedup_key)
            });
        }
    }
}

/// Resolve text input from argument, file, or stdin.
async fn resolve_input(
    text: Option<String>,
    file: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(t) = text {
        return Ok(t);
    }
    if let Some(path) = file {
        input::validate_file_path(path)?;
        return Ok(tokio::fs::read_to_string(path).await?);
    }
    // Read from stdin
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    if buf.is_empty() {
        return Err("No input provided. Pass text as argument, --file, or pipe to stdin.".into());
    }
    Ok(buf)
}

/// Print output in the requested format.
fn print_output(value: &serde_json::Value, format: &str) {
    match format {
        "text" => {
            if let Some(rules) = value.get("rules").and_then(|r| r.as_array()) {
                // Insights response: items contain "total_ratings". List-rules items use "name"/"category".
                if rules
                    .first()
                    .map(|r| r.get("total_ratings").is_some())
                    .unwrap_or(false)
                {
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
                // list-rules response: items use "name" and "category"
                for r in rules {
                    let name = r.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                    let category = r.get("category").and_then(|v| v.as_str()).unwrap_or("?");
                    println!("{name}  [{category}]");
                }
                let total = rules.len();
                eprintln!("\n{total} rule(s)");
                return;
            }
            if let Some(diagnostics) = value.get("diagnostics").and_then(|d| d.as_array()) {
                for d in diagnostics {
                    let rule = d.get("rule").and_then(|v| v.as_str()).unwrap_or("?");
                    let severity = d.get("severity").and_then(|v| v.as_str()).unwrap_or("?");
                    let message = d.get("message").and_then(|v| v.as_str()).unwrap_or("?");
                    // API returns 0-indexed line numbers; add 1 for human-readable display
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
            if let Some(score) = value.get("score") {
                println!(
                    "{}",
                    serde_json::to_string_pretty(score).unwrap_or_default()
                );
            }
            // Compare response: fixed / introduced / persistent + original_score / revised_score
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

                if let (Some(orig), Some(rev)) =
                    (value.get("original_score"), value.get("revised_score"))
                {
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
        _ => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_default()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_diag(id: &str, start_char: i64) -> serde_json::Value {
        serde_json::json!({
            "rule": "repetition",
            "severity": "warning",
            "message": "test",
            "start_line": 0,
            "start_char": start_char,
            "end_line": 0,
            "end_char": start_char + 4,
            "id": id
        })
    }

    #[test]
    fn test_dedup_compare_result_removes_duplicates() {
        let diag = make_diag("d-001", 4);
        let mut result = serde_json::json!({
            "introduced": [diag.clone(), diag.clone(), diag.clone()],
            "fixed": [diag.clone(), diag.clone()],
            "persistent": []
        });

        dedup_compare_result(&mut result);

        let introduced = result["introduced"].as_array().unwrap();
        let fixed = result["fixed"].as_array().unwrap();
        assert_eq!(
            introduced.len(),
            1,
            "introduced should have 1 entry after dedup"
        );
        assert_eq!(fixed.len(), 1, "fixed should have 1 entry after dedup");
    }

    #[test]
    fn test_dedup_compare_result_keeps_distinct_diagnostics() {
        let diag_a = make_diag("d-001", 4);
        let diag_b = make_diag("d-002", 10);
        let mut result = serde_json::json!({
            "introduced": [diag_a.clone(), diag_b.clone(), diag_a.clone()],
            "fixed": [],
            "persistent": []
        });

        dedup_compare_result(&mut result);

        let introduced = result["introduced"].as_array().unwrap();
        assert_eq!(
            introduced.len(),
            2,
            "two distinct diagnostics should remain"
        );
    }

    #[test]
    fn test_dedup_compare_result_no_op_when_no_duplicates() {
        let diag_a = make_diag("d-001", 4);
        let diag_b = make_diag("d-002", 8);
        let mut result = serde_json::json!({
            "introduced": [diag_a],
            "fixed": [diag_b],
            "persistent": []
        });

        dedup_compare_result(&mut result);

        assert_eq!(result["introduced"].as_array().unwrap().len(), 1);
        assert_eq!(result["fixed"].as_array().unwrap().len(), 1);
    }
}
