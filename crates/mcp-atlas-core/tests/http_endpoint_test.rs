//! Integration tests for the HTTP/SSE transport endpoints.
//!
//! Uses `tower::ServiceExt::oneshot` to test the Axum router without binding a port.

use std::sync::Arc;

use http_body_util::BodyExt;
use hyper::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use mcp_atlas_core::server::{AppState, build_router};
use mcp_atlas_data::models::{GitHubMetrics, Maturity, Project};
use mcp_atlas_search::SearchIndex;

/// Build a test AppState with a handful of fake projects.
fn test_state() -> Arc<AppState> {
    let projects = vec![
        Project {
            name: "Kubernetes".into(),
            description: Some("Production-Grade Container Orchestration".into()),
            homepage_url: Some("https://kubernetes.io".into()),
            repo_url: Some("https://github.com/kubernetes/kubernetes".into()),
            logo: None,
            crunchbase: None,
            category: "Orchestration & Management".into(),
            subcategory: "Scheduling & Orchestration".into(),
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: Some(GitHubMetrics {
                stars: 100000,
                forks: 35000,
                open_issues: 2000,
                contributors: 3000,
                last_commit: None,
                license: Some("Apache-2.0".into()),
                language: Some("Go".into()),
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
                stars: 24000,
                forks: 4500,
                open_issues: 1500,
                contributors: 900,
                last_commit: None,
                license: Some("Apache-2.0".into()),
                language: Some("C++".into()),
            }),
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
    ];

    let search_index = SearchIndex::build(&projects).unwrap();
    let graph = std::sync::Arc::new(mcp_atlas_graph::engine::ProjectGraph::build(&projects));
    Arc::new(AppState {
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
        github_client: None,
    })
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = build_router(test_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(http_body_util::Empty::<hyper::body::Bytes>::new())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
}

#[tokio::test]
async fn test_info_endpoint() {
    let app = build_router(test_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/info")
                .body(http_body_util::Empty::<hyper::body::Bytes>::new())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let info: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(info["server"], "mcp-atlas");
    assert_eq!(info["projects"], 2);
    assert_eq!(info["indexed"], 2);
}

#[tokio::test]
async fn test_mcp_post_initialize() {
    let app = build_router(test_state());

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("content-type", "application/json")
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["jsonrpc"], "2.0");
    assert_eq!(result["id"], 1);
    assert!(result["result"]["serverInfo"]["name"].as_str().unwrap() == "mcp-atlas");
}

#[tokio::test]
async fn test_mcp_post_tools_list() {
    let app = build_router(test_state());

    let body = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("content-type", "application/json")
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: Value = serde_json::from_slice(&body).unwrap();

    let tools = result["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 15);
}

#[tokio::test]
async fn test_mcp_post_search() {
    let app = build_router(test_state());

    let body = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "search_projects",
            "arguments": { "query": "kubernetes" }
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("content-type", "application/json")
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: Value = serde_json::from_slice(&body).unwrap();

    assert!(
        result["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Kubernetes")
    );
}

#[tokio::test]
async fn test_mcp_post_unknown_method() {
    let app = build_router(test_state());

    let body = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "nonexistent/method",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("content-type", "application/json")
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["error"]["code"], -32601);
}

// --- Streamable HTTP transport tests ---

#[tokio::test]
async fn test_streamable_initialize_creates_session() {
    let app = build_router(test_state());

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/stream")
                .header("content-type", "application/json")
                .header("accept", "application/json")
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Should have a session ID header
    let session_id = response
        .headers()
        .get("mcp-session-id")
        .expect("should have Mcp-Session-Id header")
        .to_str()
        .unwrap();
    assert!(!session_id.is_empty());

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["result"]["serverInfo"]["name"], "mcp-atlas");
}

#[tokio::test]
async fn test_streamable_requires_session_for_non_init() {
    let app = build_router(test_state());

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/stream")
                .header("content-type", "application/json")
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_streamable_session_lifecycle() {
    let state = test_state();

    // Step 1: Initialize and get session
    let app = build_router(state.clone());
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/stream")
                .header("content-type", "application/json")
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    let session_id = response
        .headers()
        .get("mcp-session-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Step 2: Use session for a tools/list call
    let app = build_router(state.clone());
    let body = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/stream")
                .header("content-type", "application/json")
                .header("mcp-session-id", &session_id)
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["result"]["tools"].as_array().unwrap().len(), 15);

    // Step 3: Delete session
    let app = build_router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/mcp/stream")
                .header("mcp-session-id", &session_id)
                .body(http_body_util::Empty::<hyper::body::Bytes>::new())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Step 4: Verify session is invalidated
    let app = build_router(state);
    let body = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/list",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/stream")
                .header("content-type", "application/json")
                .header("mcp-session-id", &session_id)
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_streamable_sse_response() {
    let state = test_state();

    // Initialize first
    let app = build_router(state.clone());
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/stream")
                .header("content-type", "application/json")
                .header("accept", "text/event-stream")
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("mcp-session-id").is_some());

    // Response should be SSE content type
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/event-stream"));
}

#[tokio::test]
async fn test_404_for_unknown_route() {
    let app = build_router(test_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(http_body_util::Empty::<hyper::body::Bytes>::new())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_mcp_post_batch_request() {
    let app = build_router(test_state());

    let body = json!([
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

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("content-type", "application/json")
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: Value = serde_json::from_slice(&body).unwrap();

    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["id"], 1);
    assert_eq!(arr[1]["id"], 2);
    assert!(!arr[1]["result"]["tools"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let app = build_router(test_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(http_body_util::Empty::<hyper::body::Bytes>::new())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/plain"));

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&body);

    assert!(text.contains("mcp_atlas_projects_total 2"));
    assert!(text.contains("mcp_atlas_requests_total 0"));
    assert!(text.contains("mcp_atlas_uptime_seconds"));
    assert!(text.contains("mcp_atlas_graph_nodes"));
}

#[tokio::test]
async fn test_mcp_post_rejects_wrong_content_type() {
    let app = build_router(test_state());

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("content-type", "text/plain")
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    serde_json::to_vec(&body).unwrap(),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
