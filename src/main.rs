#[allow(dead_code)]
mod client;
mod error;
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

        /// Output format: json (default), or text
        #[arg(short, long, default_value = "json")]
        output: String,
    },

    /// List all available analysis rules
    ListRules,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = ApiClient::new(cli.api_url, cli.api_key);

    match cli.command {
        Commands::Analyze {
            text,
            file,
            score_only,
            rules,
            severity_min,
            output,
        } => {
            let input = resolve_input(text, file.as_deref()).await?;

            let mut args = serde_json::json!({ "text": input });
            if score_only {
                args["score_only"] = serde_json::json!(true);
            }
            if let Some(rules) = rules {
                args["rules"] = serde_json::json!(rules);
            }
            if let Some(sev) = severity_min {
                args["severity_min"] = serde_json::json!(sev);
            }

            let result = client.call_remote("analyze", args).await?;
            print_output(&result, &output);
        }

        Commands::Compare {
            original,
            revised,
            original_file,
            revised_file,
            output,
        } => {
            let orig = resolve_input(original, original_file.as_deref()).await?;
            let rev = resolve_input(revised, revised_file.as_deref()).await?;

            let args = serde_json::json!({
                "original": orig,
                "revised": rev,
            });

            let result = client.call_remote("compare", args).await?;
            print_output(&result, &output);
        }

        Commands::ListRules => {
            let result = client
                .call_remote("list_rules", serde_json::json!({}))
                .await?;
            print_output(&result, "json");
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
