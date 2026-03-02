mod graph;
mod prompts;
mod recommend;
mod search;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::{Value, json};
use tracing::debug;

use crate::error::codes;
use crate::server::AppState;

/// Async handler for a dynamically registered (e.g. plugin) tool.
pub type PluginToolHandler = Box<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = anyhow::Result<Value>> + Send>> + Send + Sync,
>;

/// A tool registered at runtime (e.g. from a WASM plugin). Listed in tools/list and dispatched in tools/call.
pub struct DynamicTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    handler: Arc<PluginToolHandler>,
}

impl std::fmt::Debug for DynamicTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .finish_non_exhaustive()
    }
}

impl DynamicTool {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        handler: PluginToolHandler,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
            handler: Arc::new(handler),
        }
    }

    pub async fn call(&self, arguments: Value) -> anyhow::Result<Value> {
        (self.handler.as_ref())(arguments).await
    }

    /// Clone of the handler for dispatch without holding the plugin_tools lock across await.
    pub fn handler_ref(&self) -> Arc<PluginToolHandler> {
        Arc::clone(&self.handler)
    }

    /// Serialize to MCP tool list entry.
    pub fn to_list_entry(&self, annotations: &Value) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.input_schema,
            "annotations": annotations
        })
    }
}

/// Register a dynamically loaded plugin tool. Call this when loading a WASM plugin to expose its tools.
pub fn register_plugin_tool(state: &Arc<AppState>, tool: DynamicTool) {
    if let Ok(mut guard) = state.plugin_tools.write() {
        guard.push(tool);
    }
}

/// MCP protocol versions this server supports.
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2024-11-05", "2025-03-26"];

/// Route a JSON-RPC 2.0 request to the appropriate handler.
pub async fn handle_jsonrpc(state: &Arc<AppState>, request: &Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = request.get("params").cloned().unwrap_or(json!({}));

    debug!("JSON-RPC method={method}");
    state
        .request_count
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    match method {
        // MCP lifecycle
        "initialize" => {
            // Validate client protocol version if provided
            let client_version = params.get("protocolVersion").and_then(|v| v.as_str());
            if let Some(ver) = client_version
                && !SUPPORTED_PROTOCOL_VERSIONS.contains(&ver)
            {
                return make_error_response(
                    id,
                    codes::SERVER_ERROR,
                    &format!(
                        "Unsupported protocol version: {ver}. Supported: {}",
                        SUPPORTED_PROTOCOL_VERSIONS.join(", ")
                    ),
                );
            }
            let protocol_version = client_version.unwrap_or("2024-11-05");
            make_success_response(id, initialize_result(Some(protocol_version)))
        }
        "initialized" => make_success_response(id, json!({})),
        "ping" => make_success_response(id, json!({})),

        // MCP tool discovery (supports cursor-based pagination per spec)
        "tools/list" => {
            let cursor = params.get("cursor").and_then(|c| c.as_str());
            make_success_response(id, tools_list_paginated(state, cursor))
        }

        // MCP tool invocation
        "tools/call" => {
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            // Register cancellation token for this request
            let request_id_str = id.to_string().trim_matches('"').to_string();
            let cancel_token = tokio_util::sync::CancellationToken::new();
            {
                let mut in_flight = state.in_flight.write().unwrap();
                in_flight.insert(request_id_str.clone(), cancel_token.clone());
            }

            let start = std::time::Instant::now();
            let result = tokio::select! {
                res = dispatch_tool(state, tool_name, &arguments) => res,
                () = cancel_token.cancelled() => {
                    Err(anyhow::anyhow!("Request cancelled by client"))
                }
            };
            let latency_ms = start.elapsed().as_millis() as u64;

            // Remove from in-flight tracking
            {
                let mut in_flight = state.in_flight.write().unwrap();
                in_flight.remove(&request_id_str);
            }

            if let Some(logger) = &state.audit_logger {
                let params_hash = crate::audit::params_hash(&arguments);
                let status = if result.is_ok() { "ok" } else { "error" };
                logger.log_tool_call(tool_name, &params_hash, status, latency_ms);
            }

            match result {
                Ok(result) => make_success_response(id, result),
                Err(e) => make_error_response_with_data(
                    id,
                    codes::SERVER_ERROR,
                    &e.to_string(),
                    json!({ "tool": tool_name }),
                ),
            }
        }

        // MCP resource discovery & reading
        "resources/list" => make_success_response(id, resources_list()),
        "resources/templates/list" => make_success_response(id, resource_templates_list()),
        "resources/read" => {
            let uri = params.get("uri").and_then(|u| u.as_str()).unwrap_or("");
            match read_resource(state, uri) {
                Ok(result) => make_success_response(id, result),
                Err(e) => make_error_response(id, codes::SERVER_ERROR, &e.to_string()),
            }
        }

        // MCP resource subscriptions
        "resources/subscribe" => {
            let uri = params.get("uri").and_then(|u| u.as_str()).unwrap_or("");
            if uri.is_empty() {
                make_error_response(id, codes::SERVER_ERROR, "Missing required parameter: uri")
            } else {
                let mut subs = state.resource_subscriptions.write().unwrap();
                subs.insert(uri.to_string());
                debug!("Subscribed to resource: {uri}");
                make_success_response(id, json!({}))
            }
        }
        "resources/unsubscribe" => {
            let uri = params.get("uri").and_then(|u| u.as_str()).unwrap_or("");
            let mut subs = state.resource_subscriptions.write().unwrap();
            subs.remove(uri);
            debug!("Unsubscribed from resource: {uri}");
            make_success_response(id, json!({}))
        }

        // MCP prompt discovery & execution
        "prompts/list" => make_success_response(id, prompts::prompts_list()),
        "prompts/get" => {
            let prompt_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            match prompts::get_prompt(prompt_name, &arguments) {
                Ok(result) => make_success_response(id, result),
                Err(e) => make_error_response(id, codes::SERVER_ERROR, &e.to_string()),
            }
        }

        // MCP cancellation notification
        "notifications/cancelled" => {
            let request_id = params
                .get("requestId")
                .map(|v| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default();
            let reason = params
                .get("reason")
                .and_then(|r| r.as_str())
                .unwrap_or("client requested cancellation");
            debug!("Cancellation requested for request {request_id}: {reason}");

            // Cancel the in-flight request if it exists
            let token = {
                let in_flight = state.in_flight.read().unwrap();
                in_flight.get(&request_id).cloned()
            };
            if let Some(token) = token {
                token.cancel();
                debug!("Cancelled in-flight request {request_id}");
            }

            // Notifications don't get responses, but since we return from handle_jsonrpc,
            // the caller (handle_jsonrpc_message) will suppress it for notifications.
            make_success_response(id, json!({}))
        }

        // MCP logging level control
        "logging/setLevel" => {
            let level = params
                .get("level")
                .and_then(|l| l.as_str())
                .unwrap_or("info");

            let valid_levels = [
                "debug",
                "info",
                "notice",
                "warning",
                "error",
                "critical",
                "alert",
                "emergency",
            ];
            if valid_levels.contains(&level) {
                let mut log_level = state.log_level.write().unwrap();
                *log_level = level.to_string();
                // Map MCP level names to tracing EnvFilter directives and apply at runtime
                let tracing_level = match level {
                    "notice" => "info",
                    "warning" => "warn",
                    "critical" | "alert" | "emergency" => "error",
                    other => other,
                };
                if let Some(ref reload_fn) = state.log_level_reload {
                    reload_fn(tracing_level);
                }
                debug!("Log level set to: {level}");
                make_success_response(id, json!({}))
            } else {
                make_error_response(
                    id,
                    codes::SERVER_ERROR,
                    &format!("Invalid log level: {level}"),
                )
            }
        }

        // MCP completions (auto-complete for tool arguments)
        "completion/complete" => {
            let ref_obj = params.get("ref").cloned().unwrap_or(json!({}));
            let argument = params.get("argument").cloned().unwrap_or(json!({}));
            make_success_response(id, handle_completion(state, &ref_obj, &argument))
        }

        // Roots — server acknowledges roots/list (typically server→client, but handle gracefully)
        "roots/list" => make_success_response(id, json!({ "roots": [] })),
        "notifications/roots/list_changed" => make_success_response(id, json!({})),

        _ => make_error_response(
            id,
            codes::METHOD_NOT_FOUND,
            &format!("Method not found: {method}"),
        ),
    }
}

/// Dispatch a tool call to the appropriate handler.
async fn dispatch_tool(
    state: &Arc<AppState>,
    tool_name: &str,
    arguments: &Value,
) -> anyhow::Result<Value> {
    match tool_name {
        "search_projects" => search::handle_search_projects(state, arguments).await,
        "get_project" => search::handle_get_project(state, arguments),
        "compare_projects" => search::handle_compare_projects(state, arguments),
        "list_categories" => search::handle_list_categories(state),
        "get_stats" => search::handle_get_stats(state),
        "find_alternatives" => search::handle_find_alternatives(state, arguments),
        "get_health_score" => search::handle_get_health_score(state, arguments),
        "suggest_stack" => recommend::handle_suggest_stack(state, arguments),
        "analyze_trends" => recommend::handle_analyze_trends(state, arguments),
        "get_relationships" => graph::handle_get_relationships(state, arguments).await,
        "find_path" => graph::handle_find_path(state, arguments).await,
        "get_graph_stats" => graph::handle_get_graph_stats(state).await,
        "get_good_first_issues" => search::handle_get_good_first_issues(state, arguments),
        "get_migration_path" => recommend::handle_get_migration_path(state, arguments).await,
        _ => {
            let handler = {
                let guard = state.plugin_tools.read().map_err(|e| anyhow::anyhow!("plugin_tools lock: {}", e))?;
                guard.iter().find(|t| t.name == tool_name).map(|t| t.handler_ref())
            };
            match handler {
                Some(h) => (h.as_ref())(arguments.clone()).await,
                None => anyhow::bail!("Unknown tool: {tool_name}"),
            }
        }
    }
}

/// MCP initialize response with server capabilities.
/// Returns the client's requested protocol version for spec compliance.
fn initialize_result(protocol_version: Option<&str>) -> Value {
    let version = protocol_version.unwrap_or("2024-11-05");
    json!({
        "protocolVersion": version,
        "capabilities": {
            "tools": { "listChanged": false },
            "resources": { "subscribe": true, "listChanged": false },
            "prompts": { "listChanged": false },
            "completions": {},
            "logging": {},
        },
        "serverInfo": {
            "name": "cncf-mcp",
            "version": env!("CARGO_PKG_VERSION"),
        }
    })
}

/// List MCP tools with optional cursor-based pagination (per MCP spec).
///
/// Returns core tools plus any dynamically registered plugin tools on the first page.
fn tools_list_paginated(state: &Arc<AppState>, cursor: Option<&str>) -> Value {
    if cursor.is_some() {
        // No more pages — we return everything on the first page
        return json!({ "tools": [] });
    }
    tools_list(state)
}

/// Common annotations for all tools — all are read-only queries.
fn read_only_annotations() -> Value {
    json!({
        "title": "CNCF Landscape Query",
        "readOnlyHint": true,
        "destructiveHint": false,
        "idempotentHint": true,
        "openWorldHint": false
    })
}

/// List all available MCP tools (core + dynamically registered).
fn tools_list(state: &Arc<AppState>) -> Value {
    let annot = read_only_annotations();
    let core_tools = core_tools_list(&annot);
    let plugin_tools: Vec<Value> = state
        .plugin_tools
        .read()
        .map(|guard| {
            guard
                .iter()
                .map(|t| t.to_list_entry(&annot))
                .collect()
        })
        .unwrap_or_default();
    let tools: Vec<Value> = core_tools
        .into_iter()
        .chain(plugin_tools)
        .collect();
    json!({ "tools": tools })
}

/// Core (built-in) tools list only.
fn core_tools_list(annot: &Value) -> Vec<Value> {
    let arr = json!([
            {
                "name": "search_projects",
                "description": "Search CNCF projects by keyword, category, or maturity level",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query" },
                        "category": { "type": "string", "description": "Filter by category" },
                        "maturity": { "type": "string", "enum": ["sandbox", "incubating", "graduated"] },
                        "min_stars": { "type": "number", "description": "Minimum GitHub stars filter" },
                        "language": { "type": "string", "description": "Filter by primary programming language (e.g., 'Go', 'Rust')" },
                        "limit": { "type": "number", "description": "Max results per page (default 10)" },
                        "offset": { "type": "number", "description": "Skip N results for pagination (default 0)" }
                    },
                    "required": ["query"]
                },
                "annotations": annot
            },
            {
                "name": "get_project",
                "description": "Get full details for a specific CNCF project by name",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Project name (e.g., 'prometheus')" }
                    },
                    "required": ["name"]
                },
                "annotations": annot
            },
            {
                "name": "compare_projects",
                "description": "Compare two or more CNCF projects side-by-side",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "projects": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of project names to compare"
                        }
                    },
                    "required": ["projects"]
                },
                "annotations": annot
            },
            {
                "name": "list_categories",
                "description": "List all CNCF landscape categories and subcategories",
                "inputSchema": { "type": "object", "properties": {} },
                "annotations": annot
            },
            {
                "name": "get_stats",
                "description": "Get overall CNCF landscape statistics",
                "inputSchema": { "type": "object", "properties": {} },
                "annotations": annot
            },
            {
                "name": "find_alternatives",
                "description": "Find alternative CNCF projects in the same subcategory",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project": { "type": "string", "description": "Project name to find alternatives for" },
                        "limit": { "type": "number", "description": "Max results (default 10)" }
                    },
                    "required": ["project"]
                },
                "annotations": annot
            },
            {
                "name": "get_health_score",
                "description": "Get project health metrics and computed health score",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project": { "type": "string", "description": "Project name" }
                    },
                    "required": ["project"]
                },
                "annotations": annot
            },
            {
                "name": "suggest_stack",
                "description": "Suggest a cloud-native stack for a given use case. Recommends graduated/incubating CNCF projects for each architectural layer.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "use_case": { "type": "string", "description": "Description of the architecture needed (e.g., 'microservices with observability and service mesh')" },
                        "constraints": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Constraints (e.g., 'no Java', 'must support ARM')"
                        }
                    },
                    "required": ["use_case"]
                },
                "annotations": annot
            },
            {
                "name": "analyze_trends",
                "description": "Analyze adoption trends in a CNCF landscape category. Shows project counts, star distribution, maturity breakdown, language stats, and top projects.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "category": { "type": "string", "description": "Category name (e.g., 'Observability', 'Runtime', 'Orchestration')" }
                    },
                    "required": ["category"]
                },
                "annotations": annot
            },
            {
                "name": "get_relationships",
                "description": "Get all known relationships for a CNCF project (alternatives, integrations, components, extensions)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project": { "type": "string", "description": "Project name (e.g., 'Kubernetes')" },
                        "relation": { "type": "string", "description": "Optional filter: 'alternative', 'integrates', 'component', 'extends', 'supersedes'" }
                    },
                    "required": ["project"]
                },
                "annotations": annot
            },
            {
                "name": "find_path",
                "description": "Find the shortest relationship path between two CNCF projects in the knowledge graph",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "from": { "type": "string", "description": "Source project name" },
                        "to": { "type": "string", "description": "Target project name" }
                    },
                    "required": ["from", "to"]
                },
                "annotations": annot
            },
            {
                "name": "get_graph_stats",
                "description": "Get knowledge graph statistics — total nodes, edges, and relationship type distribution",
                "inputSchema": { "type": "object", "properties": {} },
                "annotations": annot
            },
            {
                "name": "get_good_first_issues",
                "description": "List CNCF projects that are good candidates for contributors (have repo URLs). Optionally filter by language or category. Configure GITHUB_TOKEN to fetch open issues with 'good first issue' label.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "language": { "type": "string", "description": "Filter by primary language (e.g., Go, Rust)" },
                        "category": { "type": "string", "description": "Filter by category or subcategory" },
                        "limit": { "type": "number", "description": "Max projects to return (default 20)" }
                    }
                },
                "annotations": annot
            },
            {
                "name": "get_migration_path",
                "description": "Get a structured migration guide from one CNCF project to another. Returns key differences, relationship path, and high-level migration steps.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "from_project": { "type": "string", "description": "Source project name" },
                        "to_project": { "type": "string", "description": "Target project name" },
                        "from": { "type": "string", "description": "Alias for from_project" },
                        "to": { "type": "string", "description": "Alias for to_project" }
                    },
                    "required": []
                },
                "annotations": annot
            }
    ]);
    arr.as_array().cloned().unwrap_or_default()
}

/// Read an MCP resource by URI.
fn read_resource(state: &Arc<AppState>, uri: &str) -> anyhow::Result<Value> {
    match uri {
        "cncf://landscape/overview" => {
            let stats = search::handle_get_stats(state)?;
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/markdown",
                    "text": stats["content"][0]["text"]
                }]
            }))
        }
        _ if uri.starts_with("cncf://projects/") => {
            let name = uri.strip_prefix("cncf://projects/").unwrap_or("");
            let args = json!({ "name": name });
            let result = search::handle_get_project(state, &args)?;
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": result["content"][0]["text"]
                }]
            }))
        }
        _ if uri.starts_with("cncf://categories/") => {
            let result = search::handle_list_categories(state)?;
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/markdown",
                    "text": result["content"][0]["text"]
                }]
            }))
        }
        _ => anyhow::bail!("Unknown resource URI: {uri}"),
    }
}

/// List available MCP resources.
fn resources_list() -> Value {
    json!({
        "resources": [
            {
                "uri": "cncf://landscape/overview",
                "name": "CNCF Landscape Overview",
                "description": "High-level statistics about the CNCF landscape",
                "mimeType": "text/markdown"
            },
            {
                "uri": "cncf://categories/all",
                "name": "CNCF Categories",
                "description": "All landscape categories and subcategories",
                "mimeType": "text/markdown"
            }
        ]
    })
}

/// List MCP resource templates (URI patterns with placeholders).
fn resource_templates_list() -> Value {
    json!({
        "resourceTemplates": [
            {
                "uriTemplate": "cncf://projects/{name}",
                "name": "CNCF Project Details",
                "description": "Get details for a specific CNCF project by name",
                "mimeType": "application/json"
            },
            {
                "uriTemplate": "cncf://categories/{category}",
                "name": "CNCF Category",
                "description": "Get projects in a specific landscape category",
                "mimeType": "text/markdown"
            }
        ]
    })
}

/// Handle MCP completion requests — provide auto-complete suggestions for tool arguments.
fn handle_completion(state: &Arc<AppState>, ref_obj: &Value, argument: &Value) -> Value {
    let arg_name = argument.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let arg_value = argument.get("value").and_then(|v| v.as_str()).unwrap_or("");

    let suggestions: Vec<String> = match arg_name {
        // Project name arguments — fuzzy match against all project names
        "project" | "name" | "tool_name" | "from" | "to" | "from_project" | "to_project" => {
            let lower = arg_value.to_lowercase();
            let mut names: Vec<String> = state
                .projects
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&lower))
                .map(|p| p.name.clone())
                .collect();
            names.sort();
            names.dedup();
            names.into_iter().take(20).collect()
        }
        // Language arguments — complete from known languages in the dataset
        "language" => {
            let lower = arg_value.to_lowercase();
            let mut langs: Vec<String> = state
                .projects
                .iter()
                .filter_map(|p| {
                    p.github
                        .as_ref()
                        .and_then(|g| g.language.as_ref())
                        .cloned()
                })
                .filter(|l| !l.is_empty() && l.to_lowercase().contains(&lower))
                .collect();
            langs.sort();
            langs.dedup();
            langs.into_iter().take(20).collect()
        }
        // Category arguments
        "category" => {
            let lower = arg_value.to_lowercase();
            let mut cats: Vec<String> = state
                .projects
                .iter()
                .map(|p| p.category.clone())
                .filter(|c| !c.is_empty() && c.to_lowercase().contains(&lower))
                .collect();
            cats.sort();
            cats.dedup();
            cats.into_iter().take(20).collect()
        }
        // Maturity filter
        "maturity" => {
            vec!["sandbox".into(), "incubating".into(), "graduated".into()]
        }
        // Relation type filter
        "relation" => {
            vec![
                "alternative".into(),
                "integrates".into(),
                "component".into(),
                "extends".into(),
                "supersedes".into(),
            ]
        }
        _ => {
            // Check if the ref is a resource template — complete project names for URI
            let ref_type = ref_obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if ref_type == "ref/resource" {
                let lower = arg_value.to_lowercase();
                state
                    .projects
                    .iter()
                    .filter(|p| p.name.to_lowercase().contains(&lower))
                    .take(20)
                    .map(|p| p.name.clone())
                    .collect()
            } else {
                vec![]
            }
        }
    };

    json!({
        "completion": {
            "values": suggestions,
            "hasMore": false,
            "total": suggestions.len()
        }
    })
}

/// Handle a raw JSON-RPC message that may be a single request or a batch (array).
///
/// Returns `Some(response)` for requests with an `id`, or `None` if all were notifications.
/// Batch requests return a JSON array of responses (per JSON-RPC 2.0 spec §6).
pub async fn handle_jsonrpc_message(state: &Arc<AppState>, raw: &Value) -> Option<Value> {
    if let Some(batch) = raw.as_array() {
        if batch.is_empty() {
            return Some(make_error_response(
                Value::Null,
                codes::PARSE_ERROR,
                "Invalid Request: empty batch",
            ));
        }

        let mut responses = Vec::new();
        for request in batch {
            let is_notification = request.get("id").is_none();
            let response = handle_jsonrpc(state, request).await;
            if !is_notification {
                responses.push(response);
            }
        }

        if responses.is_empty() {
            None // All were notifications
        } else {
            Some(Value::Array(responses))
        }
    } else {
        // Single request
        let is_notification = raw.get("id").is_none();
        let response = handle_jsonrpc(state, raw).await;
        if is_notification {
            None
        } else {
            Some(response)
        }
    }
}

/// Build a JSON-RPC 2.0 success response.
pub fn make_success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

/// Build a JSON-RPC 2.0 error response.
pub fn make_error_response(id: Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

/// Build a JSON-RPC 2.0 error response with additional structured data.
pub fn make_error_response_with_data(id: Value, code: i32, message: &str, data: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message,
            "data": data
        }
    })
}
