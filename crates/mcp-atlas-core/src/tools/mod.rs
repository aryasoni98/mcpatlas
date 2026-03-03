mod args;
mod completion;
mod dispatch;
mod graph;
mod hybrid;
mod issue;
mod prompts;
mod recommend;
mod resources;
mod schemas;
mod search;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::{Value, json};
use tracing::debug;

use crate::error::codes;
use crate::server::AppState;

/// Async handler for a dynamically registered (e.g. plugin) tool.
pub type PluginToolHandler =
    Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = anyhow::Result<Value>> + Send>> + Send + Sync>;

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
        "initialize" => dispatch::handle_initialize(id, &params),
        "initialized" => dispatch::handle_initialized(id),
        "ping" => dispatch::handle_ping(id),
        "tools/list" => dispatch::handle_tools_list(state, id, &params),
        "tools/call" => dispatch::handle_tools_call(state, id, &params).await,
        "resources/list" => dispatch::handle_resources_list(id),
        "resources/templates/list" => dispatch::handle_resources_templates_list(id),
        "resources/read" => dispatch::handle_resources_read(state, id, &params),
        "resources/subscribe" => dispatch::handle_resources_subscribe(state, id, &params),
        "resources/unsubscribe" => dispatch::handle_resources_unsubscribe(state, id, &params),
        "prompts/list" => dispatch::handle_prompts_list(id),
        "prompts/get" => dispatch::handle_prompts_get(state, id, &params),
        "notifications/cancelled" => dispatch::handle_notifications_cancelled(state, id, &params),
        "logging/setLevel" => dispatch::handle_logging_set_level(state, id, &params),
        "completion/complete" => dispatch::handle_completion_complete(state, id, &params),
        "roots/list" => dispatch::handle_roots_list(id),
        "notifications/roots/list_changed" => dispatch::handle_notifications_roots_list_changed(id),
        _ => make_error_response(
            id,
            codes::METHOD_NOT_FOUND,
            &format!("Method not found: {method}"),
        ),
    }
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
