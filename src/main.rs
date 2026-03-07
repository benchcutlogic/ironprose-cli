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
        #[arg(long, conflicts_with_all = ["text", "file", "score_only", "rules", "severity_min"])]
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
        #[arg(long, conflicts_with_all = ["original", "revised", "original_file", "revised_file"])]
        json: Option<String>,

        /// Output format: json (default), or text
        #[arg(short, long, default_value = "json")]
        output: String,
    },

    /// List all available analysis rules
    ListRules,

    /// Dump the API schema for an endpoint (agent introspection)
    ///
    /// Examples:
    ///   ironprose schema analyze
    ///   ironprose schema compare
    ///   ironprose schema list-rules
    ///   ironprose schema          # dumps full OpenAPI spec
    Schema {
        /// Endpoint name: analyze, compare, rate, list-rules, entitlement
        endpoint: Option<String>,
    },
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
            output,
        } => {
            let args = if let Some(raw) = json {
                input::validate_json_input(&raw)?
            } else {
                let orig = resolve_input(original, original_file.as_deref()).await?;
                let rev = resolve_input(revised, revised_file.as_deref()).await?;
                input::validate_text_input(&orig)?;
                input::validate_text_input(&rev)?;

                serde_json::json!({
                    "original": orig,
                    "revised": rev,
                })
            };

            let result = client.call_remote("compare", args).await?;
            print_output(&result, &output);
        }

        Commands::ListRules => {
            let result = client
                .call_remote("list_rules", serde_json::json!({}))
                .await?;
            print_output(&result, "json");
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
    }

    Ok(())
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
            if let Some(diagnostics) = value.get("diagnostics").and_then(|d| d.as_array()) {
                for d in diagnostics {
                    let rule = d.get("rule").and_then(|v| v.as_str()).unwrap_or("?");
                    let severity = d.get("severity").and_then(|v| v.as_str()).unwrap_or("?");
                    let message = d.get("message").and_then(|v| v.as_str()).unwrap_or("?");
                    let line = d.get("start_line").and_then(|v| v.as_u64()).unwrap_or(0);
                    eprintln!("  [{severity}] L{line}: {message} ({rule})");
                }
                let count = diagnostics.len();
                eprintln!("\n{count} diagnostic(s)");
            }
            if let Some(score) = value.get("score") {
                println!(
                    "{}",
                    serde_json::to_string_pretty(score).unwrap_or_default()
                );
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
