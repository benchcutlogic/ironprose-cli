use super::error::http_status_to_mcp_error;
use reqwest::Client;
use rmcp::ErrorData as McpError;
use serde_json::Value;

/// HTTP proxy to the remote IronProse MCP API.
///
/// Remote tool calls are forwarded using the appropriate HTTP method:
/// - `analyze`, `compare`, `rate` → `POST /api/<tool_name>` with tool arguments as JSON body
/// - `list_rules` → `GET /api/rules` (read-only, no body)
///
/// The response is returned as-is to the MCP client.
pub struct RemoteProxy {
    client: Client,
    api_base: String,
    api_key: Option<String>,
}

impl RemoteProxy {
    pub fn new(api_base: String, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            client,
            api_base,
            api_key,
        }
    }

    /// Forward a tool call to the remote API.
    ///
    /// Maps the MCP tool name to the corresponding REST endpoint:
    /// - `analyze`    -> POST /api/analyze
    /// - `compare`    -> POST /api/compare
    /// - `rate`       -> POST /api/rate
    /// - `list_rules` -> GET  /api/rules
    pub async fn call_remote(&self, tool_name: &str, args: Value) -> Result<Value, McpError> {
        let (method, url) = match tool_name {
            "list_rules" => ("GET", format!("{}/api/rules", self.api_base)),
            "analyze" => ("POST", format!("{}/api/analyze", self.api_base)),
            "compare" => ("POST", format!("{}/api/compare", self.api_base)),
            "rate" => ("POST", format!("{}/api/rate", self.api_base)),
            other => {
                return Err(McpError::invalid_params(
                    format!("Unknown remote tool: {other}"),
                    None,
                ));
            }
        };

        let mut request = match method {
            "GET" => self.client.get(&url),
            _ => self.client.post(&url).json(&args),
        };

        if let Some(ref key) = self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await.map_err(|e| {
            McpError::internal_error(
                format!("Failed to reach IronProse API: {e}. Check your network connection."),
                None,
            )
        })?;

        let status = response.status().as_u16();

        if status == 200 {
            let body: Value = response.json().await.map_err(|e| {
                McpError::internal_error(format!("Failed to parse API response: {e}"), None)
            })?;
            Ok(body)
        } else {
            let body_text = response.text().await.unwrap_or_default();
            Err(http_status_to_mcp_error(status, &body_text))
        }
    }
}
