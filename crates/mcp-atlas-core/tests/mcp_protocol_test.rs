/// Integration tests for the MCP JSON-RPC protocol layer.
///
/// These tests create an AppState with sample data and exercise
/// the full JSON-RPC routing + tool dispatch pipeline.
use std::sync::Arc;

use serde_json::json;

// We test the tools module directly since it's the core of the MCP server.
// In a real deployment, these JSON-RPC messages would come over STDIO or HTTP.

fn sample_state() -> Arc<mcp_atlas_core::server::AppState> {
    use mcp_atlas_data::models::{GitHubMetrics, Maturity, Project};
    use mcp_atlas_search::SearchIndex;

    let projects = vec![
        Project {
            name: "Prometheus".into(),
            description: Some("Monitoring system and time series database".into()),
            homepage_url: Some("https://prometheus.io".into()),
            repo_url: Some("https://github.com/prometheus/prometheus".into()),
            logo: None,
            crunchbase: None,
            category: "Observability and Analysis".into(),
            subcategory: "Monitoring".into(),
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: Some(GitHubMetrics {
                stars: 55000,
                forks: 9000,
                open_issues: 800,
                contributors: 1200,
                last_commit: Some("2025-03-01T10:00:00Z".into()),
                license: Some("Apache-2.0".into()),
                language: Some("Go".into()),
            }),
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
        Project {
            name: "Grafana".into(),
            description: Some(
                "Open and composable observability and data visualization platform".into(),
            ),
            homepage_url: Some("https://grafana.com".into()),
            repo_url: Some("https://github.com/grafana/grafana".into()),
            logo: None,
            crunchbase: None,
            category: "Observability and Analysis".into(),
            subcategory: "Monitoring".into(),
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: Some(GitHubMetrics {
                stars: 65000,
                forks: 12000,
                open_issues: 3000,
                contributors: 2500,
                last_commit: Some("2025-03-01T12:00:00Z".into()),
                license: Some("AGPL-3.0".into()),
                language: Some("TypeScript".into()),
            }),
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
        Project {
            name: "Envoy".into(),
            description: Some("Cloud-native high-performance edge/middle/service proxy".into()),
            homepage_url: Some("https://www.envoyproxy.io".into()),
            repo_url: Some("https://github.com/envoyproxy/envoy".into()),
            logo: None,
            crunchbase: None,
            category: "Orchestration & Management".into(),
            subcategory: "Service Proxy".into(),
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: Some(GitHubMetrics {
                stars: 25000,
                forks: 5000,
                open_issues: 1500,
                contributors: 800,
                last_commit: Some("2025-02-28T08:00:00Z".into()),
                license: Some("Apache-2.0".into()),
                language: Some("C++".into()),
            }),
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
        Project {
            name: "Istio".into(),
            description: Some("Connect, secure, control, and observe services".into()),
            homepage_url: Some("https://istio.io".into()),
            repo_url: Some("https://github.com/istio/istio".into()),
            logo: None,
            crunchbase: None,
            category: "Orchestration & Management".into(),
            subcategory: "Service Mesh".into(),
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: None,
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
        Project {
            name: "WasmEdge".into(),
            description: Some("A lightweight and high-performance WebAssembly runtime".into()),
            homepage_url: Some("https://wasmedge.org".into()),
            repo_url: Some("https://github.com/WasmEdge/WasmEdge".into()),
            logo: None,
            crunchbase: None,
            category: "Runtime".into(),
            subcategory: "Container Runtime".into(),
            maturity: Some(Maturity::Sandbox),
            extra: Default::default(),
            github: None,
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
    ];

    let search_index = SearchIndex::build(&projects).unwrap();
    let graph = std::sync::Arc::new(mcp_atlas_graph::engine::ProjectGraph::build(&projects));
    Arc::new(mcp_atlas_core::server::AppState {
        projects,
        search_index,
        graph,
        sessions: std::sync::RwLock::new(std::collections::HashSet::new()),
        log_level: std::sync::RwLock::new("info".into()),
        log_level_reload: None,
        request_count: std::sync::atomic::AtomicU64::new(0),
        start_time: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        resource_subscriptions: std::sync::RwLock::new(std::collections::HashSet::new()),
        in_flight: std::sync::RwLock::new(std::collections::HashMap::new()),
        embedding_provider: None,
        vector_backend: None,
        plugin_tools: std::sync::RwLock::new(Vec::new()),
        audit_logger: None,
    })
}

#[tokio::test]
async fn test_initialize() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["serverInfo"]["name"].as_str().unwrap() == "mcp-atlas");
    assert!(response["result"]["capabilities"]["tools"].is_object());
}

#[tokio::test]
async fn test_tools_list() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let tools = response["result"]["tools"].as_array().unwrap();
    assert!(
        tools.len() >= 5,
        "Expected at least 5 tools, got {}",
        tools.len()
    );

    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"search_projects"));
    assert!(tool_names.contains(&"get_project"));
    assert!(tool_names.contains(&"compare_projects"));
    assert!(tool_names.contains(&"find_alternatives"));
    assert!(tool_names.contains(&"get_health_score"));
}

#[tokio::test]
async fn test_plugin_tool_registration_and_dispatch() {
    use mcp_atlas_core::tools::{DynamicTool, register_plugin_tool};
    use std::future::Future;
    use std::pin::Pin;

    let state = sample_state();
    let handler = Box::new(move |args: serde_json::Value| {
        let out = serde_json::json!({
            "content": [{ "type": "text", "text": args.get("message").and_then(|v| v.as_str()).unwrap_or("ok") }]
        });
        Box::pin(async move { Ok(out) })
            as Pin<Box<dyn Future<Output = anyhow::Result<serde_json::Value>> + Send>>
    });
    let tool = DynamicTool::new(
        "plugin_echo",
        "Echo a message (plugin tool)",
        serde_json::json!({
            "type": "object",
            "properties": { "message": { "type": "string" } }
        }),
        handler,
    );
    register_plugin_tool(&state, tool);

    let list_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });
    let list_resp = mcp_atlas_core::tools::handle_jsonrpc(&state, &list_req).await;
    let tools = list_resp["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(
        names.contains(&"plugin_echo"),
        "plugin_echo should be in tools list"
    );

    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "plugin_echo",
            "arguments": { "message": "hello from plugin" }
        }
    });
    let call_resp = mcp_atlas_core::tools::handle_jsonrpc(&state, &call_req).await;
    assert!(
        call_resp["error"].is_null(),
        "plugin_echo call should succeed: {:?}",
        call_resp["error"]
    );
    let text = call_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(text, "hello from plugin");
}

#[tokio::test]
async fn test_search_projects() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "search_projects",
            "arguments": { "query": "monitoring", "limit": 5 }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("Prometheus"),
        "Should find Prometheus for 'monitoring'"
    );
}

#[tokio::test]
async fn test_get_project_found() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "get_project",
            "arguments": { "name": "Prometheus" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("prometheus.io"));
    assert!(text.contains("55000")); // stars
}

#[tokio::test]
async fn test_get_project_not_found() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "get_project",
            "arguments": { "name": "NonExistentProject" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("not found"));
}

#[tokio::test]
async fn test_compare_projects() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "compare_projects",
            "arguments": { "projects": ["Prometheus", "Grafana"] }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Prometheus"));
    assert!(text.contains("Grafana"));
    assert!(text.contains("Graduated")); // Both are graduated
    assert!(text.contains("Go"));
    assert!(text.contains("TypeScript"));
}

#[tokio::test]
async fn test_find_alternatives() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "find_alternatives",
            "arguments": { "project": "Prometheus" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    // Grafana is in the same subcategory (Monitoring)
    assert!(
        text.contains("Grafana"),
        "Should find Grafana as alternative to Prometheus"
    );
    assert!(text.contains("Alternatives to Prometheus"));
}

#[tokio::test]
async fn test_get_health_score() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": "tools/call",
        "params": {
            "name": "get_health_score",
            "arguments": { "project": "Prometheus" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Health Report: Prometheus"));
    assert!(text.contains("**Stars:** 55000"));
    assert!(text.contains("Health Score:"));
}

#[tokio::test]
async fn test_list_categories() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 9,
        "method": "tools/call",
        "params": {
            "name": "list_categories",
            "arguments": {}
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Observability and Analysis"));
    assert!(text.contains("Monitoring"));
    assert!(text.contains("Service Proxy"));
}

#[tokio::test]
async fn test_get_stats() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "get_stats",
            "arguments": {}
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("**Total projects/products:** 5"));
    assert!(text.contains("**Graduated:** 4"));
    assert!(text.contains("**Sandbox:** 1"));
}

#[tokio::test]
async fn test_resources_list() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 11,
        "method": "resources/list",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let resources = response["result"]["resources"].as_array().unwrap();
    assert!(!resources.is_empty());
    let uris: Vec<&str> = resources
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();
    assert!(uris.contains(&"cncf://landscape/overview"));
}

#[tokio::test]
async fn test_resources_read() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 12,
        "method": "resources/read",
        "params": { "uri": "cncf://landscape/overview" }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let contents = response["result"]["contents"].as_array().unwrap();
    assert_eq!(contents[0]["uri"], "cncf://landscape/overview");
    let text = contents[0]["text"].as_str().unwrap();
    assert!(text.contains("Total projects"));
}

#[tokio::test]
async fn test_unknown_method() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 99,
        "method": "nonexistent/method",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert_eq!(response["error"]["code"], -32601);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Method not found")
    );
}

#[tokio::test]
async fn test_get_good_first_issues() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 91,
        "method": "tools/call",
        "params": {
            "name": "get_good_first_issues",
            "arguments": { "limit": 5 }
        }
    });
    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Good First Issue"));
}

#[tokio::test]
async fn test_get_migration_path() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 92,
        "method": "tools/call",
        "params": {
            "name": "get_migration_path",
            "arguments": { "from_project": "Prometheus", "to_project": "Grafana" }
        }
    });
    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Migration Guide"));
    assert!(text.contains("Prometheus"));
    assert!(text.contains("Grafana"));
}

#[tokio::test]
async fn test_unknown_tool() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 100,
        "method": "tools/call",
        "params": {
            "name": "nonexistent_tool",
            "arguments": {}
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert_eq!(response["error"]["code"], -32000);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Unknown tool")
    );
}

#[tokio::test]
async fn test_suggest_stack() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 101,
        "method": "tools/call",
        "params": {
            "name": "suggest_stack",
            "arguments": {
                "use_case": "microservices with monitoring and service proxy"
            }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Suggested Cloud-Native Stack"));
    assert!(text.contains("monitoring"));
}

#[tokio::test]
async fn test_suggest_stack_with_constraints() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 102,
        "method": "tools/call",
        "params": {
            "name": "suggest_stack",
            "arguments": {
                "use_case": "observability stack",
                "constraints": ["no C++"]
            }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Suggested Cloud-Native Stack"));
    assert!(text.contains("Constraints"));
}

#[tokio::test]
async fn test_analyze_trends() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 103,
        "method": "tools/call",
        "params": {
            "name": "analyze_trends",
            "arguments": { "category": "Observability" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Trend Analysis"));
    assert!(text.contains("Monitoring"));
    // Should find Prometheus and Grafana
    assert!(text.contains("Prometheus") || text.contains("Grafana"));
}

#[tokio::test]
async fn test_analyze_trends_no_match() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 104,
        "method": "tools/call",
        "params": {
            "name": "analyze_trends",
            "arguments": { "category": "NonExistentCategory" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("No projects found"));
}

#[tokio::test]
async fn test_prompts_list() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 105,
        "method": "prompts/list",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let prompts = response["result"]["prompts"].as_array().unwrap();
    assert!(prompts.len() >= 3);

    let names: Vec<&str> = prompts
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"evaluate_tool"));
    assert!(names.contains(&"plan_migration"));
    assert!(names.contains(&"review_stack"));
    assert!(names.contains(&"onboard_contributor"));
}

#[tokio::test]
async fn test_prompts_get_evaluate_tool() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 106,
        "method": "prompts/get",
        "params": {
            "name": "evaluate_tool",
            "arguments": {
                "tool_name": "Prometheus",
                "use_case": "monitoring Kubernetes clusters"
            }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["error"].is_null());
    let messages = response["result"]["messages"].as_array().unwrap();
    assert!(!messages.is_empty());
    let text = messages[0]["content"]["text"].as_str().unwrap();
    assert!(text.contains("Prometheus"));
    assert!(text.contains("monitoring Kubernetes clusters"));
}

#[tokio::test]
async fn test_prompts_get_plan_migration() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 107,
        "method": "prompts/get",
        "params": {
            "name": "plan_migration",
            "arguments": {
                "from": "Envoy",
                "to": "Istio"
            }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["error"].is_null());
    let messages = response["result"]["messages"].as_array().unwrap();
    let text = messages[0]["content"]["text"].as_str().unwrap();
    assert!(text.contains("Envoy"));
    assert!(text.contains("Istio"));
    assert!(text.contains("migration"));
}

#[tokio::test]
async fn test_tools_list_count() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 108,
        "method": "tools/list",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let tools = response["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 14, "Expected 14 tools, got {}", tools.len());
}

#[tokio::test]
async fn test_initialize_has_prompts_capability() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 109,
        "method": "initialize",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"]["capabilities"]["prompts"].is_object());
    assert!(response["result"]["capabilities"]["tools"].is_object());
    assert!(response["result"]["capabilities"]["resources"].is_object());
}

#[tokio::test]
async fn test_get_relationships() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 201,
        "method": "tools/call",
        "params": {
            "name": "get_relationships",
            "arguments": { "project": "Prometheus" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    // Prometheus and Grafana are both in "Monitoring" so should be AlternativeTo
    assert!(
        text.contains("AlternativeTo"),
        "Expected AlternativeTo relationships, got: {text}"
    );
    assert!(
        text.contains("Grafana"),
        "Expected Grafana as alternative, got: {text}"
    );
}

#[tokio::test]
async fn test_get_relationships_filtered() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 202,
        "method": "tools/call",
        "params": {
            "name": "get_relationships",
            "arguments": { "project": "Prometheus", "relation": "alternative" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Grafana"));
    // Should only show AlternativeTo, not IntegratesWith
    assert!(!text.contains("IntegratesWith"));
}

#[tokio::test]
async fn test_get_relationships_not_found() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 203,
        "method": "tools/call",
        "params": {
            "name": "get_relationships",
            "arguments": { "project": "NonExistentProject" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("No relationships found"));
}

#[tokio::test]
async fn test_find_path() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 204,
        "method": "tools/call",
        "params": {
            "name": "find_path",
            "arguments": { "from": "Prometheus", "to": "Grafana" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    // Direct AlternativeTo edge exists between them
    assert!(
        text.contains("Path:") || text.contains("hop"),
        "Expected path result, got: {text}"
    );
}

#[tokio::test]
async fn test_find_path_no_connection() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 205,
        "method": "tools/call",
        "params": {
            "name": "find_path",
            "arguments": { "from": "Prometheus", "to": "NonExistent" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("No path found"));
}

#[tokio::test]
async fn test_get_graph_stats() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 206,
        "method": "tools/call",
        "params": {
            "name": "get_graph_stats",
            "arguments": {}
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Knowledge Graph Statistics"));
    assert!(text.contains("Nodes:"));
    assert!(text.contains("Edges:"));
}

#[tokio::test]
async fn test_resource_templates_list() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 301,
        "method": "resources/templates/list",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let templates = response["result"]["resourceTemplates"].as_array().unwrap();
    assert!(templates.len() >= 2);
    assert!(
        templates
            .iter()
            .any(|t| t["uriTemplate"] == "cncf://projects/{name}")
    );
}

#[tokio::test]
async fn test_completion_project_name() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 302,
        "method": "completion/complete",
        "params": {
            "ref": { "type": "ref/tool", "name": "get_project" },
            "argument": { "name": "name", "value": "prom" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let values = response["result"]["completion"]["values"]
        .as_array()
        .unwrap();
    assert!(
        values.iter().any(|v| v.as_str().unwrap() == "Prometheus"),
        "Expected Prometheus in completions, got: {values:?}"
    );
}

#[tokio::test]
async fn test_completion_category() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 303,
        "method": "completion/complete",
        "params": {
            "ref": { "type": "ref/tool", "name": "analyze_trends" },
            "argument": { "name": "category", "value": "Observ" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let values = response["result"]["completion"]["values"]
        .as_array()
        .unwrap();
    assert!(
        values
            .iter()
            .any(|v| v.as_str().unwrap().contains("Observability")),
        "Expected Observability category in completions, got: {values:?}"
    );
}

#[tokio::test]
async fn test_completion_maturity() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 304,
        "method": "completion/complete",
        "params": {
            "ref": { "type": "ref/tool", "name": "search_projects" },
            "argument": { "name": "maturity", "value": "" }
        }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let values = response["result"]["completion"]["values"]
        .as_array()
        .unwrap();
    assert_eq!(values.len(), 3);
    assert!(values.iter().any(|v| v == "graduated"));
}

#[tokio::test]
async fn test_initialize_has_completions_capability() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 305,
        "method": "initialize",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"]["capabilities"]["completions"].is_object());
}

// --- JSON-RPC batch request tests ---

#[tokio::test]
async fn test_batch_request() {
    let state = sample_state();
    let batch = json!([
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        },
        {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }
    ]);

    let response = mcp_atlas_core::tools::handle_jsonrpc_message(&state, &batch).await;
    let responses = response.unwrap();
    let arr = responses.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["id"], 1);
    assert_eq!(arr[0]["result"]["serverInfo"]["name"], "mcp-atlas");
    assert_eq!(arr[1]["id"], 2);
    assert!(!arr[1]["result"]["tools"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_batch_all_notifications() {
    let state = sample_state();
    // A batch of all notifications (no "id") should return None
    let batch = json!([
        {
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        }
    ]);

    let response = mcp_atlas_core::tools::handle_jsonrpc_message(&state, &batch).await;
    assert!(response.is_none());
}

#[tokio::test]
async fn test_batch_empty_array() {
    let state = sample_state();
    let batch = json!([]);

    let response = mcp_atlas_core::tools::handle_jsonrpc_message(&state, &batch).await;
    let resp = response.unwrap();
    assert!(resp["error"]["code"].as_i64().is_some());
}

#[tokio::test]
async fn test_batch_mixed_notifications_and_requests() {
    let state = sample_state();
    let batch = json!([
        {
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        },
        {
            "jsonrpc": "2.0",
            "id": 10,
            "method": "ping",
            "params": {}
        }
    ]);

    let response = mcp_atlas_core::tools::handle_jsonrpc_message(&state, &batch).await;
    let responses = response.unwrap();
    let arr = responses.as_array().unwrap();
    // Only the request with an id should produce a response
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], 10);
}

// --- MCP logging tests ---

#[tokio::test]
async fn test_logging_set_level() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 400,
        "method": "logging/setLevel",
        "params": { "level": "debug" }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"].is_object());
    assert!(response["error"].is_null());

    // Verify the level was actually stored
    let level = state.log_level.read().unwrap();
    assert_eq!(*level, "debug");
}

#[tokio::test]
async fn test_logging_invalid_level() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 401,
        "method": "logging/setLevel",
        "params": { "level": "bogus" }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Invalid log level")
    );
}

#[tokio::test]
async fn test_initialize_has_logging_capability() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 402,
        "method": "initialize",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"]["capabilities"]["logging"].is_object());
}

// --- Tool annotations tests ---

#[tokio::test]
async fn test_tools_have_annotations() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 500,
        "method": "tools/list",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let tools = response["result"]["tools"].as_array().unwrap();

    // Every tool should have annotations
    for tool in tools {
        let annot = &tool["annotations"];
        assert_eq!(
            annot["readOnlyHint"], true,
            "tool {} missing readOnlyHint",
            tool["name"]
        );
        assert_eq!(annot["destructiveHint"], false);
        assert_eq!(annot["idempotentHint"], true);
        assert_eq!(annot["openWorldHint"], false);
    }
}

#[tokio::test]
async fn test_tools_list_with_cursor_returns_empty() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 501,
        "method": "tools/list",
        "params": { "cursor": "some-cursor" }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    let tools = response["result"]["tools"].as_array().unwrap();
    assert!(
        tools.is_empty(),
        "cursor pagination should return empty second page"
    );
}

// --- Protocol version validation ---

#[tokio::test]
async fn test_initialize_with_valid_protocol_version() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 600,
        "method": "initialize",
        "params": { "protocolVersion": "2024-11-05" }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"]["serverInfo"].is_object());
    assert!(response["error"].is_null());
}

#[tokio::test]
async fn test_initialize_with_unsupported_protocol_version() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 601,
        "method": "initialize",
        "params": { "protocolVersion": "1999-01-01" }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Unsupported protocol version")
    );
}

#[tokio::test]
async fn test_initialize_with_2025_protocol_version() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 602,
        "method": "initialize",
        "params": { "protocolVersion": "2025-03-26" }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"]["serverInfo"].is_object());
    assert_eq!(
        response["result"]["protocolVersion"].as_str().unwrap(),
        "2025-03-26"
    );
}

// --- Resource subscription tests ---

#[tokio::test]
async fn test_resource_subscribe() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 700,
        "method": "resources/subscribe",
        "params": { "uri": "cncf://landscape/overview" }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"].is_object());
    assert!(response["error"].is_null());

    // Verify subscription was stored
    let subs = state.resource_subscriptions.read().unwrap();
    assert!(subs.contains("cncf://landscape/overview"));
}

#[tokio::test]
async fn test_resource_unsubscribe() {
    let state = sample_state();

    // Subscribe first
    let request = json!({
        "jsonrpc": "2.0",
        "id": 701,
        "method": "resources/subscribe",
        "params": { "uri": "cncf://landscape/overview" }
    });
    mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;

    // Unsubscribe
    let request = json!({
        "jsonrpc": "2.0",
        "id": 702,
        "method": "resources/unsubscribe",
        "params": { "uri": "cncf://landscape/overview" }
    });
    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"].is_object());

    // Verify removed
    let subs = state.resource_subscriptions.read().unwrap();
    assert!(!subs.contains("cncf://landscape/overview"));
}

#[tokio::test]
async fn test_initialize_advertises_subscribe() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 703,
        "method": "initialize",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert_eq!(
        response["result"]["capabilities"]["resources"]["subscribe"],
        true
    );
}

// --- Cancellation tests ---

#[tokio::test]
async fn test_cancellation_notification() {
    let state = sample_state();

    // Manually register a cancellation token as if a request were in-flight
    let token = tokio_util::sync::CancellationToken::new();
    {
        let mut in_flight = state.in_flight.write().unwrap();
        in_flight.insert("42".to_string(), token.clone());
    }

    // Send cancellation notification (no id = notification)
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/cancelled",
        "params": { "requestId": "42", "reason": "user cancelled" }
    });
    mcp_atlas_core::tools::handle_jsonrpc(&state, &notification).await;

    // The token should now be cancelled
    assert!(token.is_cancelled());
}

#[tokio::test]
async fn test_cancellation_unknown_request() {
    let state = sample_state();

    // Send cancellation for a non-existent request — should not panic
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/cancelled",
        "params": { "requestId": "999" }
    });
    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &notification).await;
    // Should still return a response (suppressed as notification by the caller)
    assert!(response["result"].is_object());
}

#[tokio::test]
async fn test_tool_call_tracks_in_flight() {
    let state = sample_state();

    // Execute a tool call — after it completes, in_flight should be empty
    let request = json!({
        "jsonrpc": "2.0",
        "id": 800,
        "method": "tools/call",
        "params": { "name": "get_stats", "arguments": {} }
    });
    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"].is_object());

    // In-flight map should be empty after completion
    let in_flight = state.in_flight.read().unwrap();
    assert!(in_flight.is_empty());
}

// --- Roots list tests ---

#[tokio::test]
async fn test_roots_list() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 900,
        "method": "roots/list",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"]["roots"].is_array());
    assert!(response["error"].is_null());
}

#[tokio::test]
async fn test_roots_list_changed_notification() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "method": "notifications/roots/list_changed",
        "params": {}
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(response["result"].is_object());
}

// --- Structured error data tests ---

#[tokio::test]
async fn test_tool_error_includes_data() {
    let state = sample_state();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 950,
        "method": "tools/call",
        "params": { "name": "nonexistent_tool", "arguments": {} }
    });

    let response = mcp_atlas_core::tools::handle_jsonrpc(&state, &request).await;
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Unknown tool")
    );
    assert_eq!(response["error"]["data"]["tool"], "nonexistent_tool");
}
