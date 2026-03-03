//! Per-method JSON-RPC handlers. Each MCP method has a dedicated handler.

use std::sync::Arc;

use serde_json::{Value, json};
use tracing::debug;

use crate::error::codes;
use crate::server::AppState;

use super::args;
use super::completion;
use super::graph;
use super::issue;
use super::prompts;
use super::recommend;
use super::resources;
use super::schemas;
use super::search;

const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2024-11-05", "2025-03-26"];

fn ok(id: Value, result: Value) -> Value {
    super::make_success_response(id, result)
}

fn err(id: Value, code: i32, msg: &str) -> Value {
    super::make_error_response(id, code, msg)
}

fn err_data(id: Value, code: i32, msg: &str, data: Value) -> Value {
    super::make_error_response_with_data(id, code, msg, data)
}

pub fn handle_initialize(id: Value, params: &Value) -> Value {
    let client_version = params.get("protocolVersion").and_then(|v| v.as_str());
    if let Some(ver) = client_version
        && !SUPPORTED_PROTOCOL_VERSIONS.contains(&ver)
    {
        return err(
            id,
            codes::SERVER_ERROR,
            &format!(
                "Unsupported protocol version: {ver}. Supported: {}",
                SUPPORTED_PROTOCOL_VERSIONS.join(", ")
            ),
        );
    }
    let protocol_version = client_version.unwrap_or("2024-11-05");
    ok(id, initialize_result(Some(protocol_version)))
}

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
            "name": "mcp-atlas",
            "version": env!("CARGO_PKG_VERSION"),
        }
    })
}

pub fn handle_initialized(id: Value) -> Value {
    ok(id, json!({}))
}

pub fn handle_ping(id: Value) -> Value {
    ok(id, json!({}))
}

pub fn handle_tools_list(state: &Arc<AppState>, id: Value, params: &Value) -> Value {
    let cursor = params.get("cursor").and_then(|c| c.as_str());
    ok(id, tools_list_paginated(state, cursor))
}

fn tools_list_paginated(state: &Arc<AppState>, cursor: Option<&str>) -> Value {
    if cursor.is_some() {
        return json!({ "tools": [] });
    }
    let annot = schemas::read_only_annotations();
    let core_tools = schemas::core_tools_list(&annot);
    let plugin_tools: Vec<Value> = state
        .plugin_tools
        .read()
        .map(|guard| guard.iter().map(|t| t.to_list_entry(&annot)).collect())
        .unwrap_or_default();
    let tools: Vec<Value> = core_tools.into_iter().chain(plugin_tools).collect();
    json!({ "tools": tools })
}

pub async fn handle_tools_call(state: &Arc<AppState>, id: Value, params: &Value) -> Value {
    let tool_name = args::parse_string_arg(params, "name", "");
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    let request_id_str = id.to_string().trim_matches('"').to_string();
    let cancel_token = tokio_util::sync::CancellationToken::new();
    {
        let mut in_flight = state.in_flight.write().expect("in_flight lock poisoned");
        in_flight.insert(request_id_str.clone(), cancel_token.clone());
    }

    let start = std::time::Instant::now();
    let result = tokio::select! {
        res = dispatch_tool(state, &tool_name, &arguments) => res,
        () = cancel_token.cancelled() => Err(anyhow::anyhow!("Request cancelled by client")),
    };
    let latency_ms = start.elapsed().as_millis() as u64;

    {
        let mut in_flight = state.in_flight.write().expect("in_flight lock poisoned");
        in_flight.remove(&request_id_str);
    }

    if let Some(logger) = &state.audit_logger {
        let params_hash = crate::audit::params_hash(&arguments);
        let status = if result.is_ok() { "ok" } else { "error" };
        logger.log_tool_call(&tool_name, &params_hash, status, latency_ms);
    }

    match result {
        Ok(r) => ok(id, r),
        Err(e) => err_data(
            id,
            codes::SERVER_ERROR,
            &e.to_string(),
            json!({ "tool": tool_name }),
        ),
    }
}

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
        "get_issue_context" => issue::handle_get_issue_context(state, arguments).await,
        _ => {
            let handler = {
                let guard = state
                    .plugin_tools
                    .read()
                    .map_err(|e| anyhow::anyhow!("plugin_tools lock: {}", e))?;
                guard
                    .iter()
                    .find(|t| t.name == tool_name)
                    .map(|t| t.handler_ref())
            };
            match handler {
                Some(h) => (h.as_ref())(arguments.clone()).await,
                None => anyhow::bail!("Unknown tool: {tool_name}"),
            }
        }
    }
}

pub fn handle_resources_list(id: Value) -> Value {
    ok(id, resources::resources_list())
}

pub fn handle_resources_templates_list(id: Value) -> Value {
    ok(id, resources::resource_templates_list())
}

pub fn handle_resources_read(state: &Arc<AppState>, id: Value, params: &Value) -> Value {
    let uri = args::parse_string_arg(params, "uri", "");
    match resources::read_resource(state, &uri) {
        Ok(r) => ok(id, r),
        Err(e) => err(id, codes::SERVER_ERROR, &e.to_string()),
    }
}

pub fn handle_resources_subscribe(state: &Arc<AppState>, id: Value, params: &Value) -> Value {
    let uri = args::parse_string_arg(params, "uri", "");
    if uri.is_empty() {
        return err(id, codes::SERVER_ERROR, "Missing required parameter: uri");
    }
    let mut subs = state
        .resource_subscriptions
        .write()
        .expect("resource_subscriptions lock poisoned");
    subs.insert(uri.clone());
    debug!("Subscribed to resource: {uri}");
    ok(id, json!({}))
}

pub fn handle_resources_unsubscribe(state: &Arc<AppState>, id: Value, params: &Value) -> Value {
    let uri = args::parse_string_arg(params, "uri", "");
    let mut subs = state
        .resource_subscriptions
        .write()
        .expect("resource_subscriptions lock poisoned");
    subs.remove(&uri);
    debug!("Unsubscribed from resource: {uri}");
    ok(id, json!({}))
}

pub fn handle_prompts_list(id: Value) -> Value {
    ok(id, prompts::prompts_list())
}

pub fn handle_prompts_get(_state: &Arc<AppState>, id: Value, params: &Value) -> Value {
    let prompt_name = args::parse_string_arg(params, "name", "");
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
    match prompts::get_prompt(&prompt_name, &arguments) {
        Ok(r) => ok(id, r),
        Err(e) => err(id, codes::SERVER_ERROR, &e.to_string()),
    }
}

pub fn handle_notifications_cancelled(state: &Arc<AppState>, id: Value, params: &Value) -> Value {
    let request_id = params
        .get("requestId")
        .map(|v| v.to_string().trim_matches('"').to_string())
        .unwrap_or_default();
    let reason = params
        .get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("client requested cancellation");
    debug!("Cancellation requested for request {request_id}: {reason}");

    let token = {
        let in_flight = state.in_flight.read().expect("in_flight lock poisoned");
        in_flight.get(&request_id).cloned()
    };
    if let Some(token) = token {
        token.cancel();
        debug!("Cancelled in-flight request {request_id}");
    }
    ok(id, json!({}))
}

pub fn handle_logging_set_level(state: &Arc<AppState>, id: Value, params: &Value) -> Value {
    let level = args::parse_string_arg(params, "level", "info");
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
    if !valid_levels.contains(&level.as_str()) {
        return err(
            id,
            codes::SERVER_ERROR,
            &format!("Invalid log level: {level}"),
        );
    }
    let mut log_level = state.log_level.write().expect("log_level lock poisoned");
    *log_level = level.clone();
    let tracing_level = match level.as_str() {
        "notice" => "info",
        "warning" => "warn",
        "critical" | "alert" | "emergency" => "error",
        other => other,
    };
    if let Some(ref reload_fn) = state.log_level_reload {
        reload_fn(tracing_level);
    }
    debug!("Log level set to: {level}");
    ok(id, json!({}))
}

pub fn handle_completion_complete(state: &Arc<AppState>, id: Value, params: &Value) -> Value {
    let ref_obj = params.get("ref").cloned().unwrap_or(json!({}));
    let argument = params.get("argument").cloned().unwrap_or(json!({}));
    ok(
        id,
        completion::handle_completion(state, &ref_obj, &argument),
    )
}

pub fn handle_roots_list(id: Value) -> Value {
    ok(id, json!({ "roots": [] }))
}

pub fn handle_notifications_roots_list_changed(id: Value) -> Value {
    ok(id, json!({}))
}
