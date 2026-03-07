use super::audit::AuditLog;
use super::local_tools;
use super::proxy::RemoteProxy;
use rmcp::model::*;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

/// Standalone stdio MCP server for IronProse.
///
/// Routes tool calls to either:
/// - **Remote proxy**: `analyze`, `compare`, `rate`, `list_rules` → HTTP POST to remote API
/// - **Local tools**: `read_file`, `write_file`, `list_files` → sandboxed workspace operations
#[derive(Clone)]
pub struct StdioMcpServer {
    proxy: Arc<RemoteProxy>,
    workspace_dir: Option<PathBuf>,
    audit_log: Option<Arc<AuditLog>>,
}

impl StdioMcpServer {
    pub fn new(api_base: String, api_key: Option<String>, workspace_dir: Option<String>) -> Self {
        let ws = workspace_dir.map(PathBuf::from);
        let audit_log = ws.as_ref().map(|p| Arc::new(AuditLog::new(p)));

        Self {
            proxy: Arc::new(RemoteProxy::new(api_base, api_key)),
            workspace_dir: ws,
            audit_log,
        }
    }
}

/// Convert a serde_json::Value (expected object) to the rmcp JsonObject (Map<String,Value>).
fn value_to_json_object(v: Value) -> serde_json::Map<String, Value> {
    match v {
        Value::Object(map) => map,
        _ => serde_json::Map::new(),
    }
}

/// Helper to build a successful CallToolResult with text content.
fn text_result(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(text)],
        structured_content: None,
        is_error: Some(false),
        meta: None,
    }
}

/// Helper to build an error CallToolResult.
fn error_result(msg: String) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(msg)],
        structured_content: None,
        is_error: Some(true),
        meta: None,
    }
}

impl ServerHandler for StdioMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "ironprose-stdio-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("IronProse Stdio MCP".into()),
                description: Some(
                    "Standalone stdio MCP server for prose analysis with local file tools".into(),
                ),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "IronProse prose analysis server. Use 'analyze' to check prose quality, \
                 'compare' to diff revisions, 'list_rules' to see available analyzers, \
                 and 'read_file'/'write_file'/'list_files' for workspace file operations."
                    .into(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(build_tool_list()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name: &str = &request.name;
        let args: Value = match request.arguments {
            Some(ref map) => serde_json::to_value(map).unwrap_or(Value::Object(Default::default())),
            None => Value::Object(Default::default()),
        };

        tracing::info!(tool = tool_name, "Tool call received");

        match tool_name {
            // ── Remote proxy tools ────────────────────────────
            "analyze" | "compare" | "rate" | "list_rules" => {
                match self.proxy.call_remote(tool_name, args).await {
                    Ok(result) => Ok(text_result(
                        serde_json::to_string_pretty(&result).unwrap_or_default(),
                    )),
                    Err(e) => Ok(error_result(format!("Error: {}", e.message))),
                }
            }

            // ── Local file tools ──────────────────────────────
            "read_file" => {
                let relative_path = args["path"].as_str().unwrap_or("");
                let offset = args["offset"].as_u64();
                let max_bytes = args["max_bytes"].as_u64();

                let ws = local_tools::resolve_workspace(
                    args["workspace_dir"].as_str(),
                    self.workspace_dir.as_ref(),
                );

                match local_tools::read_file(ws.as_deref(), relative_path, offset, max_bytes) {
                    Ok(result) => Ok(text_result(
                        serde_json::to_string_pretty(&result).unwrap_or_default(),
                    )),
                    Err(e) => Ok(error_result(format!("Error: {}", e.message))),
                }
            }

            "write_file" => {
                let relative_path = args["path"].as_str().unwrap_or("");
                let content = args["content"].as_str().unwrap_or("");

                let ws = local_tools::resolve_workspace(
                    args["workspace_dir"].as_str(),
                    self.workspace_dir.as_ref(),
                );

                // Use a per-request audit log when the request workspace differs
                // from the server-wide workspace, so writes are logged to the
                // correct workspace's audit file.
                let request_audit;
                let audit = if args["workspace_dir"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty())
                {
                    request_audit = ws.as_ref().map(|p| AuditLog::new(p));
                    request_audit.as_ref()
                } else {
                    self.audit_log.as_deref()
                };

                match local_tools::write_file(ws.as_deref(), relative_path, content, audit) {
                    Ok(result) => Ok(text_result(
                        serde_json::to_string_pretty(&result).unwrap_or_default(),
                    )),
                    Err(e) => Ok(error_result(format!("Error: {}", e.message))),
                }
            }

            "list_files" => {
                let subdirectory = args["subdirectory"].as_str();
                let recursive = args["recursive"].as_bool().unwrap_or(false);

                let ws = local_tools::resolve_workspace(
                    args["workspace_dir"].as_str(),
                    self.workspace_dir.as_ref(),
                );

                match local_tools::list_files(ws.as_deref(), subdirectory, recursive) {
                    Ok(result) => Ok(text_result(
                        serde_json::to_string_pretty(&result).unwrap_or_default(),
                    )),
                    Err(e) => Ok(error_result(format!("Error: {}", e.message))),
                }
            }

            _ => Ok(error_result(format!(
                "Unknown tool: {tool_name}. Use list_tools to see available tools."
            ))),
        }
    }
}

/// Helper to create a JSON schema object for tool input.
fn schema(v: Value) -> Arc<serde_json::Map<String, Value>> {
    Arc::new(value_to_json_object(v))
}

/// Build the static list of tools exposed by this server.
fn build_tool_list() -> Vec<Tool> {
    vec![
        // ── Remote proxy tools (matching remote API schemas) ──
        Tool {
            name: "analyze".into(),
            title: None,
            description: Some(
                "Analyze prose text for style, grammar, and craft issues. Returns diagnostics \
                 with severity levels, scores, and an optional style profile."
                    .into(),
            ),
            input_schema: schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to analyze."
                    },
                    "locale": {
                        "type": "string",
                        "description": "Language locale: 'en-us', 'en-gb', or 'en-any' (default)."
                    },
                    "rules": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Only run these specific rules. If omitted, all rules run."
                    },
                    "severity_min": {
                        "type": "string",
                        "description": "Minimum severity: 'error', 'warning', 'information', or 'hint'."
                    },
                    "score_only": {
                        "type": "boolean",
                        "description": "If true, only return scores (no diagnostics/profile)."
                    },
                    "config": {
                        "type": "object",
                        "description": "Per-request analyzer configuration overrides."
                    }
                },
                "required": ["text"]
            })),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "compare".into(),
            title: None,
            description: Some(
                "Compare original and revised text, showing fixed, introduced, and persistent \
                 diagnostics plus scores for both versions."
                    .into(),
            ),
            input_schema: schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "original": {
                        "type": "string",
                        "description": "The original text (before edits)."
                    },
                    "revised": {
                        "type": "string",
                        "description": "The revised text (after edits)."
                    },
                    "locale": {
                        "type": "string",
                        "description": "Language locale for spell checking."
                    },
                    "config": {
                        "type": "object",
                        "description": "Per-request analyzer configuration overrides."
                    }
                },
                "required": ["original", "revised"]
            })),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "rate".into(),
            title: None,
            description: Some(
                "Submit feedback on a diagnostic: 'helpful', 'not_helpful', or 'false_positive'."
                    .into(),
            ),
            input_schema: schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "rule": {
                        "type": "string",
                        "description": "The rule (analyzer) that produced the diagnostic."
                    },
                    "rating": {
                        "type": "string",
                        "description": "Rating: 'helpful', 'not_helpful', or 'false_positive'."
                    },
                    "context": {
                        "type": "string",
                        "description": "Optional context about the rating."
                    },
                    "diagnostic_id": {
                        "type": "string",
                        "description": "Optional diagnostic ID from the analyze response."
                    }
                },
                "required": ["rule", "rating"]
            })),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "list_rules".into(),
            title: None,
            description: Some("List all available analysis rules with their categories.".into()),
            input_schema: schema(serde_json::json!({
                "type": "object",
                "properties": {}
            })),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        // ── Local file tools ──────────────────────────────────
        Tool {
            name: "read_file".into(),
            title: None,
            description: Some(
                "Read a file from the workspace. Large files are paginated — use offset and \
                 max_bytes to page through content. Maximum single read is 1 MB."
                    .into(),
            ),
            input_schema: schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path within the workspace (e.g. 'chapter1.md')."
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Byte offset to start reading from (default: 0)."
                    },
                    "max_bytes": {
                        "type": "integer",
                        "description": "Maximum bytes to read (default: 65536, max: 1048576)."
                    },
                    "workspace_dir": {
                        "type": "string",
                        "description": "Override workspace directory (default: IRONPROSE_WORKSPACE env)."
                    }
                },
                "required": ["path"]
            })),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "write_file".into(),
            title: None,
            description: Some(
                "Write content to a file in the workspace. Creates parent directories as needed. \
                 Maximum content size is 1 MB. All writes are logged locally."
                    .into(),
            ),
            input_schema: schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path within the workspace."
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write."
                    },
                    "workspace_dir": {
                        "type": "string",
                        "description": "Override workspace directory (default: IRONPROSE_WORKSPACE env)."
                    }
                },
                "required": ["path", "content"]
            })),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "list_files".into(),
            title: None,
            description: Some(
                "List files in the workspace directory. Skips hidden files/directories.".into(),
            ),
            input_schema: schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "subdirectory": {
                        "type": "string",
                        "description": "Optional subdirectory to list (relative to workspace)."
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "List files recursively (default: false)."
                    },
                    "workspace_dir": {
                        "type": "string",
                        "description": "Override workspace directory."
                    }
                }
            })),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
    ]
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a workspace with files.
    pub(crate) fn create_test_workspace() -> TempDir {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("chapter1.md"),
            "# Chapter 1\n\nSome prose here.\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("chapter2.md"),
            "# Chapter 2\n\nMore prose.\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join("notes")).unwrap();
        std::fs::write(dir.path().join("notes/ideas.md"), "An idea.\n").unwrap();
        dir
    }

    #[test]
    fn test_resolve_workspace_dir_explicit_overrides_default() {
        let ws = local_tools::resolve_workspace(
            Some("/explicit/path"),
            Some(&PathBuf::from("/default/path")),
        );
        assert_eq!(ws.unwrap(), PathBuf::from("/explicit/path"));
    }

    #[test]
    fn test_resolve_workspace_dir_empty_with_default() {
        let ws = local_tools::resolve_workspace(Some(""), Some(&PathBuf::from("/default")));
        assert_eq!(ws.unwrap(), PathBuf::from("/default"));
    }

    #[test]
    fn test_resolve_workspace_dir_empty_no_default() {
        let ws = local_tools::resolve_workspace(None, None);
        assert!(ws.is_none());
    }

    #[test]
    fn test_read_file_basic() {
        let dir = create_test_workspace();
        let result = local_tools::read_file(Some(dir.path()), "chapter1.md", None, None).unwrap();
        assert!(result.content.contains("Chapter 1"));
        assert!(!result.truncated);
    }

    #[test]
    fn test_read_file_pagination() {
        let dir = create_test_workspace();
        let result =
            local_tools::read_file(Some(dir.path()), "chapter1.md", None, Some(10)).unwrap();
        assert_eq!(result.bytes_read, 10);
        assert!(result.truncated);
        assert_eq!(result.offset, 0);
    }

    #[test]
    fn test_read_file_offset() {
        let dir = create_test_workspace();
        let result =
            local_tools::read_file(Some(dir.path()), "chapter1.md", Some(2), Some(5)).unwrap();
        assert_eq!(result.bytes_read, 5);
        assert_eq!(result.offset, 2);
    }

    #[test]
    fn test_write_file_basic() {
        let dir = create_test_workspace();
        let result =
            local_tools::write_file(Some(dir.path()), "output.md", "Hello, world!", None).unwrap();
        assert!(result.success);
        assert_eq!(result.bytes_written, 13);
        let content = std::fs::read_to_string(dir.path().join("output.md")).unwrap();
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_write_file_too_large() {
        let dir = create_test_workspace();
        let big_content = "x".repeat(2_000_000);
        let result = local_tools::write_file(Some(dir.path()), "big.md", &big_content, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_files_basic() {
        let dir = create_test_workspace();
        let result = local_tools::list_files(Some(dir.path()), None, false).unwrap();
        let names: Vec<&str> = result.files.iter().map(|f| f.path.as_str()).collect();
        assert!(names.iter().any(|n| n.contains("chapter1.md")));
        assert!(names.iter().any(|n| n.contains("chapter2.md")));
    }

    #[test]
    fn test_list_files_recursive() {
        let dir = create_test_workspace();
        let result = local_tools::list_files(Some(dir.path()), None, true).unwrap();
        let names: Vec<&str> = result.files.iter().map(|f| f.path.as_str()).collect();
        assert!(names.iter().any(|n| n.contains("ideas.md")));
    }

    #[test]
    fn test_write_file_audit_log() {
        let dir = create_test_workspace();
        let audit = AuditLog::new(dir.path());
        local_tools::write_file(
            Some(dir.path()),
            "logged.md",
            "Audited content",
            Some(&audit),
        )
        .unwrap();

        let log_path = dir.path().join(".ironprose/audit.jsonl");
        assert!(log_path.exists());
        let log_content = std::fs::read_to_string(log_path).unwrap();
        assert!(log_content.contains("logged.md"));
        assert!(log_content.contains("write_file"));
    }

    #[test]
    fn test_sandbox_traversal_rejected_in_read() {
        let dir = create_test_workspace();
        let result = local_tools::read_file(Some(dir.path()), "../../../etc/passwd", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_sandbox_traversal_rejected_in_write() {
        let dir = create_test_workspace();
        let result =
            local_tools::write_file(Some(dir.path()), "../escape.txt", "evil content", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_sandbox_absolute_path_rejected() {
        let dir = create_test_workspace();
        let result = local_tools::read_file(Some(dir.path()), "/etc/passwd", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_workspace_returns_error() {
        let result = local_tools::read_file(None, "chapter1.md", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_list_contains_all_tools() {
        let tools = build_tool_list();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(names.contains(&"analyze"));
        assert!(names.contains(&"compare"));
        assert!(names.contains(&"rate"));
        assert!(names.contains(&"list_rules"));
        assert!(names.contains(&"read_file"));
        assert!(names.contains(&"write_file"));
        assert!(names.contains(&"list_files"));
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn test_tool_schemas_have_type_object() {
        let tools = build_tool_list();
        for tool in &tools {
            let schema_type = tool.input_schema.get("type").and_then(|v| v.as_str());
            assert_eq!(
                schema_type,
                Some("object"),
                "Tool '{}' schema must have type: object",
                tool.name
            );
        }
    }

    /// Parity contract test: verify that proxy tool names and required params
    /// match the known remote API surface.
    #[test]
    fn test_proxy_tool_parity_with_remote_api() {
        let tools = build_tool_list();

        let get_required = |name: &str| -> Vec<String> {
            let tool = tools.iter().find(|t| t.name == name).unwrap();
            tool.input_schema
                .get("required")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        };

        // analyze: requires "text"
        let required = get_required("analyze");
        assert!(required.contains(&"text".to_string()));

        // compare: requires "original" and "revised"
        let required = get_required("compare");
        assert!(required.contains(&"original".to_string()));
        assert!(required.contains(&"revised".to_string()));

        // rate: requires "rule" and "rating"
        let required = get_required("rate");
        assert!(required.contains(&"rule".to_string()));
        assert!(required.contains(&"rating".to_string()));

        // list_rules: no required params
        let required = get_required("list_rules");
        assert!(required.is_empty());
    }
}
