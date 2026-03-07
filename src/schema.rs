//! Schema introspection for the IronProse API.
//!
//! Fetches the OpenAPI spec from the live API server, caches it locally,
//! and falls back to the embedded spec if offline. This ensures agents
//! always get the current API schema, not a stale build-time snapshot.

use reqwest::Client;
use serde_json::Value;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Embedded fallback spec (build-time snapshot).
const EMBEDDED_SPEC: &str = include_str!("../tests/fixtures/openapi.json");

/// Cache TTL — refetch after 1 hour.
const CACHE_TTL: Duration = Duration::from_secs(3600);

/// Cache file location: ~/.ironprose/openapi.json
fn cache_path() -> Option<PathBuf> {
    dirs_free().map(|d| d.join("openapi.json"))
}

/// Get the cache directory (~/.ironprose), creating it if needed.
fn dirs_free() -> Option<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    let dir = PathBuf::from(home).join(".ironprose");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// Check if the cache is fresh (exists and within TTL).
fn cache_is_fresh() -> bool {
    if let Some(path) = cache_path() {
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                return SystemTime::now()
                    .duration_since(modified)
                    .map(|age| age < CACHE_TTL)
                    .unwrap_or(false);
            }
        }
    }
    false
}

/// Read the cached spec from disk.
fn read_cache() -> Option<Value> {
    let path = cache_path()?;
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Write the spec to the cache.
fn write_cache(spec: &Value) {
    if let Some(path) = cache_path() {
        if let Ok(json) = serde_json::to_string(spec) {
            let _ = std::fs::write(path, json);
        }
    }
}

/// Fetch the OpenAPI spec from the remote API server.
async fn fetch_remote(api_base: &str) -> Option<Value> {
    let url = format!("{api_base}/api/openapi.json");
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .ok()?;
    let response = client.get(&url).send().await.ok()?;
    if response.status().is_success() {
        response.json().await.ok()
    } else {
        None
    }
}

/// Get the embedded fallback spec.
fn embedded_spec() -> Value {
    serde_json::from_str(EMBEDDED_SPEC).expect("embedded OpenAPI spec should be valid JSON")
}

/// Get the full OpenAPI spec — remote-first, cached, embedded fallback.
///
/// 1. If cache is fresh → use cache (zero latency)
/// 2. Try fetching from live API → cache + return
/// 3. Fall back to embedded spec
pub async fn full_spec(api_base: &str) -> Value {
    // 1. Fresh cache hit
    if cache_is_fresh() {
        if let Some(cached) = read_cache() {
            return cached;
        }
    }

    // 2. Try remote
    if let Some(remote) = fetch_remote(api_base).await {
        write_cache(&remote);
        return remote;
    }

    // 3. Stale cache is better than embedded
    if let Some(cached) = read_cache() {
        return cached;
    }

    // 4. Embedded fallback
    embedded_spec()
}

/// Get the schema for a specific endpoint.
///
/// Maps user-facing names to OpenAPI paths:
/// - `analyze` → POST /analyze
/// - `compare` → POST /compare
/// - `rate` → POST /rate
/// - `list-rules` / `list_rules` → GET /rules
/// - `entitlement` → GET /entitlement
pub fn endpoint_schema(spec: &Value, name: &str) -> Result<Value, String> {
    let paths = spec.get("paths").ok_or("OpenAPI spec missing 'paths'")?;

    let (http_method, api_path) = match name {
        "analyze" => ("post", "/analyze"),
        "compare" => ("post", "/compare"),
        "rate" => ("post", "/rate"),
        "list-rules" | "list_rules" | "rules" => ("get", "/rules"),
        "entitlement" => ("get", "/entitlement"),
        other => {
            return Err(format!(
            "Unknown endpoint: {other}. Available: analyze, compare, rate, list-rules, entitlement"
        ))
        }
    };

    let path_obj = paths
        .get(api_path)
        .ok_or(format!("Path {api_path} not found in OpenAPI spec"))?;

    let method_obj = path_obj
        .get(http_method)
        .ok_or(format!("Method {http_method} not found for {api_path}"))?;

    // Build a focused schema output
    let mut result = serde_json::json!({
        "endpoint": api_path,
        "method": http_method.to_uppercase(),
    });

    if let Some(request_body) = method_obj.get("requestBody") {
        result["request_body"] = request_body.clone();
    }
    if let Some(responses) = method_obj.get("responses") {
        result["responses"] = responses.clone();
    }
    if let Some(desc) = method_obj.get("description") {
        result["description"] = desc.clone();
    }
    if let Some(summary) = method_obj.get("summary") {
        result["summary"] = summary.clone();
    }
    if let Some(params) = method_obj.get("parameters") {
        result["parameters"] = params.clone();
    }

    Ok(result)
}
