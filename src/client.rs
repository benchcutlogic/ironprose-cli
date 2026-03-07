//! HTTP client for the IronProse REST API.
//!
//! Requests are sent using the appropriate HTTP method:
//! - `analyze`, `compare`, `rate` → `POST /api/<tool_name>` with JSON body
//! - `list_rules` → `GET /api/rules`
//! - `rule_doc`   → `GET /api/rules/<name>` (returns markdown)
//! - `entitlement` → `GET /api/entitlement`

use crate::error::{http_status_to_error, ApiError};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// HTTP client for the IronProse REST API.
pub struct ApiClient {
    client: Client,
    api_base: String,
    api_key: Option<String>,
}

impl ApiClient {
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

    /// Forward a tool call to the remote API, returning raw JSON.
    pub async fn call_remote(&self, tool_name: &str, args: Value) -> Result<Value, ApiError> {
        let (method, url) = match tool_name {
            "list_rules" => ("GET", format!("{}/api/rules", self.api_base)),
            "entitlement" => ("GET", format!("{}/api/entitlement", self.api_base)),
            "analyze" => ("POST", format!("{}/api/analyze", self.api_base)),
            "compare" => ("POST", format!("{}/api/compare", self.api_base)),
            "rate" => ("POST", format!("{}/api/rate", self.api_base)),
            other => {
                return Err(ApiError::Input(format!("Unknown remote tool: {other}")));
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
            ApiError::Transport(format!(
                "Failed to reach IronProse API: {e}. Check your network connection."
            ))
        })?;

        let status = response.status().as_u16();

        if status == 200 {
            let body: Value = response
                .json()
                .await
                .map_err(|e| ApiError::Parse(format!("Failed to parse API response: {e}")))?;
            Ok(body)
        } else {
            let body_text = response.text().await.unwrap_or_default();
            Err(http_status_to_error(status, &body_text))
        }
    }

    /// Forward a tool call and deserialize the response into a typed struct.
    pub async fn call_typed<T: DeserializeOwned>(
        &self,
        tool_name: &str,
        args: Value,
    ) -> Result<T, ApiError> {
        let value = self.call_remote(tool_name, args).await?;
        serde_json::from_value(value)
            .map_err(|e| ApiError::Parse(format!("Failed to deserialize response: {e}")))
    }

    /// Fetch aggregate feedback insights per analyzer rule.
    ///
    /// All parameters are optional query filters:
    /// - `since` / `until` → date range (YYYY-MM-DD)
    /// - `genre` → genre prefix filter
    /// - `work_id` → work identifier filter
    pub async fn call_insights(
        &self,
        since: Option<&str>,
        until: Option<&str>,
        genre: Option<&str>,
        work_id: Option<&str>,
    ) -> Result<Value, ApiError> {
        let url = format!("{}/api/insights", self.api_base);

        let mut request = self.client.get(&url);

        if let Some(ref key) = self.api_key {
            request = request.bearer_auth(key);
        }

        // Append optional query parameters
        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(s) = since {
            query.push(("since", s));
        }
        if let Some(u) = until {
            query.push(("until", u));
        }
        if let Some(g) = genre {
            query.push(("genre", g));
        }
        if let Some(w) = work_id {
            query.push(("work_id", w));
        }
        if !query.is_empty() {
            request = request.query(&query);
        }

        let response = request.send().await.map_err(|e| {
            ApiError::Transport(format!(
                "Failed to reach IronProse API: {e}. Check your network connection."
            ))
        })?;

        let status = response.status().as_u16();

        if status == 200 {
            let body: Value = response
                .json()
                .await
                .map_err(|e| ApiError::Parse(format!("Failed to parse API response: {e}")))?;
            Ok(body)
        } else {
            let body_text = response.text().await.unwrap_or_default();
            Err(http_status_to_error(status, &body_text))
        }
    }

    /// Fetch documentation for a specific rule (returns markdown text).
    pub async fn rule_doc(&self, rule_name: &str) -> Result<String, ApiError> {
        let url = format!("{}/api/rules/{}", self.api_base, rule_name);

        let mut request = self.client.get(&url);
        if let Some(ref key) = self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::Transport(format!("Failed to reach IronProse API: {e}")))?;

        let status = response.status().as_u16();

        if status == 200 {
            response
                .text()
                .await
                .map_err(|e| ApiError::Parse(format!("Failed to read rule doc: {e}")))
        } else {
            let body_text = response.text().await.unwrap_or_default();
            Err(http_status_to_error(status, &body_text))
        }
    }
}
