use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::info;

use cncf_mcp_data::embeddings::{EmbeddingProvider, OllamaEmbeddingProvider, OpenAICompatibleEmbeddingProvider};
use cncf_mcp_data::models::Project;
use cncf_mcp_data::pipeline::{self, PipelineConfig};
use cncf_mcp_data::storage::{GraphBackend, VectorBackend};
use cncf_mcp_graph::engine::ProjectGraph;
use cncf_mcp_search::SearchIndex;

use crate::config::{AppConfig, GraphBackendKind, Transport};
use crate::tools;

/// Callback to apply a new tracing filter at runtime (used by `logging/setLevel`).
pub type LogLevelReloadFn = Box<dyn Fn(&str) + Send + Sync>;

/// Shared application state available to all tool handlers.
pub struct AppState {
    pub projects: Vec<Project>,
    pub search_index: SearchIndex,
    pub graph: std::sync::Arc<dyn GraphBackend>,
    /// Active session IDs for Streamable HTTP transport.
    pub sessions: std::sync::RwLock<std::collections::HashSet<String>>,
    /// Current MCP log level set by client via `logging/setLevel`.
    pub log_level: std::sync::RwLock<String>,
    /// When set, applying a new level updates the tracing-subscriber filter at runtime.
    pub log_level_reload: Option<LogLevelReloadFn>,
    /// Request counter for metrics.
    pub request_count: std::sync::atomic::AtomicU64,
    /// Timestamp when the server started (seconds since UNIX epoch).
    pub start_time: u64,
    /// Resource URIs that clients have subscribed to.
    pub resource_subscriptions: std::sync::RwLock<std::collections::HashSet<String>>,
    /// In-flight request IDs mapped to cancellation tokens.
    pub in_flight:
        std::sync::RwLock<std::collections::HashMap<String, tokio_util::sync::CancellationToken>>,
    /// Optional embedding provider for hybrid (BM25 + vector) search.
    pub embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
    /// Optional vector backend (e.g. Qdrant) for hybrid search.
    pub vector_backend: Option<Arc<dyn VectorBackend>>,
    /// Dynamically registered plugin tools (name, description, schema, handler).
    pub plugin_tools: std::sync::RwLock<Vec<tools::DynamicTool>>,
    /// Optional audit logger for tool calls (params_hash, status, latency_ms; no PII).
    pub audit_logger: Option<std::sync::Arc<dyn crate::audit::AuditLogger>>,
}

/// Main entry point — loads data, builds index, starts MCP transport.
/// If `log_level_reload` is provided, MCP `logging/setLevel` will update the tracing filter at runtime.
pub async fn run(config: AppConfig, log_level_reload: Option<LogLevelReloadFn>) -> Result<()> {
    // Load and index data
    let state = Arc::new(init_state(&config, log_level_reload).await?);

    info!(
        "Server ready — {} projects indexed, {} documents in search",
        state.projects.len(),
        state.search_index.doc_count()
    );

    match config.transport {
        Transport::Stdio => run_stdio(state).await,
        Transport::Sse => run_sse(state, config.port, config.rate_limit).await,
    }
}

async fn init_state(
    config: &AppConfig,
    log_level_reload: Option<LogLevelReloadFn>,
) -> Result<AppState> {
    let cache_dir = config
        .cache_dir
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| dirs_cache_dir().join("cncf-mcp"));
    let max_age = std::time::Duration::from_secs(config.max_cache_age);

    // Try loading from cache first
    let projects = if let Some(cached) = cncf_mcp_data::cache::load(&cache_dir, max_age) {
        cached
    } else {
        // Cache miss — run the pipeline
        let pipeline_config = PipelineConfig {
            github_token: config.github_token.clone(),
            landscape_file: config.landscape_file.as_ref().map(Into::into),
            github_concurrency: if config.skip_github { 0 } else { 5 },
            artifact_hub_enabled: config.artifact_hub,
            summary_enabled: config.summary_enabled,
            summary_api_base: config.summary_api_base.clone(),
            summary_api_key: config.summary_api_key.clone(),
            summary_model: config.summary_model.clone(),
            summary_max_per_run: config.summary_max_per_run,
            ..Default::default()
        };

        let projects = if config.skip_github {
            let (_landscape, projects) = match &pipeline_config.landscape_file {
                Some(path) => cncf_mcp_data::landscape::load_landscape_from_file(path)?,
                None => cncf_mcp_data::landscape::load_landscape().await?,
            };
            projects
        } else {
            pipeline::run_pipeline(&pipeline_config).await?
        };

        // Save to cache for next startup
        if let Err(e) = cncf_mcp_data::cache::save(&cache_dir, &projects) {
            tracing::warn!("Failed to write cache: {e}");
        }

        projects
    };

    let search_index = SearchIndex::build(&projects)?;
    let graph: std::sync::Arc<dyn GraphBackend> = match config.graph_backend {
        GraphBackendKind::Mem => std::sync::Arc::new(ProjectGraph::build(&projects)),
        GraphBackendKind::Surreal => {
            #[cfg(feature = "graph-surrealdb")]
            {
                let edges = cncf_mcp_graph::engine::compute_edges(&projects);
                std::sync::Arc::new(
                    cncf_mcp_graph_surrealdb::SurrealGraphBackend::new(&edges)
                        .await
                        .context("SurrealDB graph backend init")?,
                )
            }
            #[cfg(not(feature = "graph-surrealdb"))]
            {
                anyhow::bail!(
                    "Graph backend 'surreal' requires building with --features graph-surrealdb"
                );
            }
        }
    };

    let (embedding_provider, vector_backend) = init_vector_layer(config).await?;

    Ok(AppState {
        projects,
        search_index,
        graph,
        sessions: std::sync::RwLock::new(std::collections::HashSet::new()),
        log_level: std::sync::RwLock::new("info".into()),
        log_level_reload,
        request_count: std::sync::atomic::AtomicU64::new(0),
        embedding_provider,
        vector_backend,
        start_time: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        resource_subscriptions: std::sync::RwLock::new(std::collections::HashSet::new()),
        in_flight: std::sync::RwLock::new(std::collections::HashMap::new()),
        plugin_tools: std::sync::RwLock::new(Vec::new()),
        audit_logger: Some(std::sync::Arc::new(crate::audit::StderrAuditLogger)),
    })
}

/// Initialize optional embedding provider and vector backend for hybrid search.
async fn init_vector_layer(
    config: &AppConfig,
) -> Result<(
    Option<Arc<dyn EmbeddingProvider>>,
    Option<Arc<dyn VectorBackend>>,
)> {
    let (provider, backend) = match (
        config.embedding_api_base.as_deref(),
        config.qdrant_url.as_deref(),
    ) {
        (Some(api_base), Some(qdrant_url)) => {
            let provider: Arc<dyn EmbeddingProvider> =
                if api_base.contains("openai") || api_base.contains("api.") {
                    Arc::new(
                        OpenAICompatibleEmbeddingProvider::new(
                            api_base.to_string(),
                            Some(config.embedding_model.clone()),
                            config.embedding_api_key.clone(),
                        )
                        .context("OpenAI-compatible embedding provider")?,
                    )
                } else {
                    Arc::new(
                        OllamaEmbeddingProvider::new(
                            Some(api_base.to_string()),
                            Some(config.embedding_model.clone()),
                        )
                        .context("Ollama embedding provider")?,
                    )
                };
            let dims = provider.dimensions() as u32;
            #[cfg(feature = "vector-qdrant")]
            let backend: Option<Arc<dyn VectorBackend>> = {
                let b = cncf_mcp_vector::QdrantVectorBackend::new(
                    qdrant_url.as_str(),
                    None,
                    dims,
                )
                .await
                .context("Qdrant vector backend")?;
                Some(Arc::new(b))
            };
            #[cfg(not(feature = "vector-qdrant"))]
            let backend: Option<Arc<dyn VectorBackend>> = {
                let _ = (qdrant_url, dims);
                tracing::warn!(
                    "Qdrant URL set but vector-qdrant feature disabled; build with --features vector-qdrant for hybrid search"
                );
                None
            };
            (Some(provider), backend)
        }
        _ => (None, None),
    };
    Ok((provider, backend))
}

/// Platform-appropriate cache directory (XDG on Linux, ~/Library/Caches on macOS).
fn dirs_cache_dir() -> std::path::PathBuf {
    // Simple cross-platform fallback without adding a dependency
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return std::path::PathBuf::from(xdg);
    }
    if let Ok(home) = std::env::var("HOME") {
        let home = std::path::PathBuf::from(home);
        if cfg!(target_os = "macos") {
            return home.join("Library/Caches");
        }
        return home.join(".cache");
    }
    std::path::PathBuf::from("/tmp")
}

/// Run the MCP server over STDIO using Content-Length framing (per MCP spec).
///
/// Wire format (same as LSP):
/// ```text
/// Content-Length: <N>\r\n
/// \r\n
/// <N bytes of JSON-RPC 2.0>
/// ```
async fn run_stdio(state: Arc<AppState>) -> Result<()> {
    info!("MCP STDIO transport active — reading from stdin");

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);

    loop {
        // Read headers until empty line
        let content_length = match read_content_length(&mut reader).await {
            Ok(Some(len)) => len,
            Ok(None) => {
                info!("STDIO: EOF received, shutting down");
                break;
            }
            Err(e) => {
                tracing::warn!("STDIO: Failed to read header: {e}");
                continue;
            }
        };

        // Read exactly content_length bytes
        let mut body = vec![0u8; content_length];
        tokio::io::AsyncReadExt::read_exact(&mut reader, &mut body).await?;

        let body_str = String::from_utf8_lossy(&body);
        let response = match serde_json::from_str::<serde_json::Value>(&body_str) {
            Ok(message) => tools::handle_jsonrpc_message(&state, &message).await,
            Err(e) => Some(tools::make_error_response(
                serde_json::Value::Null,
                crate::error::codes::PARSE_ERROR,
                &format!("Parse error: {e}"),
            )),
        };

        // Notifications (and all-notification batches) don't get responses
        let Some(response) = response else {
            continue;
        };

        // Write response with Content-Length framing
        let response_bytes = serde_json::to_vec(&response)?;
        let header = format!("Content-Length: {}\r\n\r\n", response_bytes.len());
        stdout.write_all(header.as_bytes()).await?;
        stdout.write_all(&response_bytes).await?;
        stdout.flush().await?;
    }

    Ok(())
}

/// Read Content-Length header from STDIO stream.
/// Returns None on EOF, Some(length) on success.
async fn read_content_length(reader: &mut BufReader<tokio::io::Stdin>) -> Result<Option<usize>> {
    let mut content_length: Option<usize> = None;
    let mut header_line = String::new();

    loop {
        header_line.clear();
        let bytes_read = reader.read_line(&mut header_line).await?;
        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        let trimmed = header_line.trim();
        if trimmed.is_empty() {
            // Empty line = end of headers
            break;
        }

        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            content_length = Some(
                value
                    .trim()
                    .parse()
                    .context("Invalid Content-Length value")?,
            );
        }
        // Ignore other headers (Content-Type, etc.)
    }

    content_length
        .map(Some)
        .ok_or_else(|| anyhow::anyhow!("Missing Content-Length header"))
}

/// Build the Axum router for the HTTP/SSE transport.
///
/// Exposed publicly so integration tests can use `tower::ServiceExt` against it.
pub fn build_router(state: Arc<AppState>) -> axum::Router {
    use axum::{
        Json, Router,
        extract::State,
        http::{HeaderMap, StatusCode},
        response::{
            IntoResponse, Response,
            sse::{Event, Sse},
        },
        routing::{get, post},
    };
    use std::convert::Infallible;
    use tokio_stream::wrappers::ReceiverStream;
    use tower_http::cors::{Any, CorsLayer};
    use tower_http::trace::TraceLayer;

    type SharedState = Arc<AppState>;

    async fn handle_mcp_post(
        State(state): State<SharedState>,
        headers: HeaderMap,
        Json(body): Json<serde_json::Value>,
    ) -> Response {
        // MCP spec requires Content-Type: application/json
        if let Err(resp) = validate_content_type(&headers) {
            return resp;
        }
        match tools::handle_jsonrpc_message(&state, &body).await {
            Some(response) => Json(response).into_response(),
            None => StatusCode::NO_CONTENT.into_response(),
        }
    }

    async fn handle_health() -> &'static str {
        "ok"
    }

    /// Prometheus-compatible metrics endpoint.
    async fn handle_metrics(State(state): State<SharedState>) -> Response {
        let requests = state
            .request_count
            .load(std::sync::atomic::Ordering::Relaxed);
        let sessions = state.sessions.read().unwrap().len();
        let uptime = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(state.start_time);
        let graph_stats = state
            .graph
            .stats()
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("graph stats failed: {e}");
                cncf_mcp_data::storage::GraphStats {
                    total_nodes: 0,
                    total_edges: 0,
                    relation_counts: std::collections::HashMap::new(),
                }
            });

        let body = format!(
            "# HELP cncf_mcp_projects_total Total number of CNCF projects loaded.\n\
             # TYPE cncf_mcp_projects_total gauge\n\
             cncf_mcp_projects_total {}\n\
             # HELP cncf_mcp_search_docs_total Total documents in the search index.\n\
             # TYPE cncf_mcp_search_docs_total gauge\n\
             cncf_mcp_search_docs_total {}\n\
             # HELP cncf_mcp_requests_total Total JSON-RPC requests processed.\n\
             # TYPE cncf_mcp_requests_total counter\n\
             cncf_mcp_requests_total {}\n\
             # HELP cncf_mcp_active_sessions Number of active Streamable HTTP sessions.\n\
             # TYPE cncf_mcp_active_sessions gauge\n\
             cncf_mcp_active_sessions {}\n\
             # HELP cncf_mcp_graph_nodes Total nodes in the knowledge graph.\n\
             # TYPE cncf_mcp_graph_nodes gauge\n\
             cncf_mcp_graph_nodes {}\n\
             # HELP cncf_mcp_graph_edges Total edges in the knowledge graph.\n\
             # TYPE cncf_mcp_graph_edges gauge\n\
             cncf_mcp_graph_edges {}\n\
             # HELP cncf_mcp_uptime_seconds Server uptime in seconds.\n\
             # TYPE cncf_mcp_uptime_seconds gauge\n\
             cncf_mcp_uptime_seconds {}\n",
            state.projects.len(),
            state.search_index.doc_count(),
            requests,
            sessions,
            graph_stats.total_nodes,
            graph_stats.total_edges,
            uptime,
        );

        (
            [(
                axum::http::header::CONTENT_TYPE,
                "text/plain; version=0.0.4; charset=utf-8",
            )],
            body,
        )
            .into_response()
    }

    async fn handle_info(State(state): State<SharedState>) -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "server": "cncf-mcp",
            "version": env!("CARGO_PKG_VERSION"),
            "projects": state.projects.len(),
            "indexed": state.search_index.doc_count(),
        }))
    }

    async fn handle_sse(
        State(state): State<SharedState>,
    ) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
        let (tx, rx) = tokio::sync::mpsc::channel(16);

        // Send initial connection event
        let endpoint = "mcp";
        let _ = tx
            .send(Ok(Event::default()
                .event("endpoint")
                .data(format!("/{endpoint}"))))
            .await;

        // Send server info as first data event
        let info = serde_json::json!({
            "server": "cncf-mcp",
            "version": env!("CARGO_PKG_VERSION"),
            "projects": state.projects.len(),
        });
        let _ = tx
            .send(Ok(Event::default().event("message").data(info.to_string())))
            .await;

        // Keep connection alive with periodic pings
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                if tx.send(Ok(Event::default().comment("ping"))).await.is_err() {
                    break;
                }
            }
        });

        Sse::new(ReceiverStream::new(rx))
    }

    /// Validate that the Content-Type header is application/json (MCP spec requirement).
    #[allow(clippy::result_large_err)]
    fn validate_content_type(headers: &HeaderMap) -> Result<(), Response> {
        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !content_type.contains("application/json") {
            return Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Content-Type must be application/json",
            )
                .into_response());
        }
        Ok(())
    }

    /// Streamable HTTP transport (MCP 2025-03-26 spec).
    ///
    /// POST: accepts JSON-RPC, responds with SSE stream or JSON based on Accept header.
    /// Each new session gets an `Mcp-Session-Id` header; subsequent requests must include it.
    async fn handle_streamable_post(
        State(state): State<SharedState>,
        headers: HeaderMap,
        Json(body): Json<serde_json::Value>,
    ) -> Response {
        // MCP spec requires Content-Type: application/json
        if let Err(resp) = validate_content_type(&headers) {
            return resp;
        }
        let accept = headers
            .get("accept")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/json");

        // Session management: check or create session
        let session_id = headers
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let active_session = if let Some(ref sid) = session_id {
            // Validate existing session
            let sessions = state.sessions.read().unwrap();
            if sessions.contains(sid) {
                sid.clone()
            } else {
                return (StatusCode::BAD_REQUEST, "Invalid session ID").into_response();
            }
        } else {
            // New session — only for `initialize` requests
            let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
            if method != "initialize" {
                return (StatusCode::BAD_REQUEST, "Missing Mcp-Session-Id header").into_response();
            }
            let new_sid = uuid::Uuid::new_v4().to_string();
            {
                let mut sessions = state.sessions.write().unwrap();
                sessions.insert(new_sid.clone());
            }
            new_sid
        };

        // Process the JSON-RPC request (supports single and batch)
        let response = tools::handle_jsonrpc_message(&state, &body).await;

        // Notifications don't get responses
        let Some(response) = response else {
            let mut resp = StatusCode::NO_CONTENT.into_response();
            resp.headers_mut()
                .insert("mcp-session-id", active_session.parse().unwrap());
            return resp;
        };

        if accept.contains("text/event-stream") {
            // Respond with SSE stream — send the result as a single SSE event
            let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(4);

            let _ = tx
                .send(Ok(Event::default()
                    .event("message")
                    .data(response.to_string())))
                .await;

            // Close the stream after sending the response
            drop(tx);

            let sse = Sse::new(ReceiverStream::new(rx));
            let mut resp = sse.into_response();
            resp.headers_mut()
                .insert("mcp-session-id", active_session.parse().unwrap());
            resp
        } else {
            // Standard JSON response
            let mut resp = Json(response).into_response();
            resp.headers_mut()
                .insert("mcp-session-id", active_session.parse().unwrap());
            resp
        }
    }

    /// GET on /mcp/stream opens an SSE stream for server-initiated notifications.
    async fn handle_streamable_get(
        State(state): State<SharedState>,
        headers: HeaderMap,
    ) -> Response {
        let session_id = headers.get("mcp-session-id").and_then(|v| v.to_str().ok());

        let Some(sid) = session_id else {
            return (StatusCode::BAD_REQUEST, "Missing Mcp-Session-Id header").into_response();
        };

        // Validate session
        {
            let sessions = state.sessions.read().unwrap();
            if !sessions.contains(sid) {
                return (StatusCode::BAD_REQUEST, "Invalid session ID").into_response();
            }
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(16);

        // Keep-alive pings
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                if tx.send(Ok(Event::default().comment("ping"))).await.is_err() {
                    break;
                }
            }
        });

        Sse::new(ReceiverStream::new(rx)).into_response()
    }

    /// DELETE on /mcp/stream terminates a session.
    async fn handle_streamable_delete(
        State(state): State<SharedState>,
        headers: HeaderMap,
    ) -> Response {
        let session_id = headers.get("mcp-session-id").and_then(|v| v.to_str().ok());

        let Some(sid) = session_id else {
            return (StatusCode::BAD_REQUEST, "Missing Mcp-Session-Id header").into_response();
        };

        let removed = {
            let mut sessions = state.sessions.write().unwrap();
            sessions.remove(sid)
        };

        if removed {
            StatusCode::NO_CONTENT.into_response()
        } else {
            (StatusCode::NOT_FOUND, "Session not found").into_response()
        }
    }

    // CORS: Allow any origin for MCP clients (browsers, remote tools)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(Any)
        .expose_headers(["mcp-session-id".parse().unwrap()]);

    Router::new()
        .route("/health", get(handle_health))
        .route("/metrics", get(handle_metrics))
        .route("/mcp", post(handle_mcp_post))
        .route("/info", get(handle_info))
        .route("/sse", get(handle_sse))
        // Streamable HTTP transport (MCP 2025-03-26)
        .route(
            "/mcp/stream",
            post(handle_streamable_post)
                .get(handle_streamable_get)
                .delete(handle_streamable_delete),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Run the MCP server over HTTP with SSE support via Axum.
async fn run_sse(state: Arc<AppState>, port: u16, rate_limit: u64) -> Result<()> {
    let mut app = build_router(state);

    // Apply concurrency limiting if configured (> 0)
    if rate_limit > 0 {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(rate_limit as usize));
        app = app.layer(axum::middleware::from_fn(
            move |req, next: axum::middleware::Next| {
                let sem = semaphore.clone();
                async move {
                    match sem.try_acquire() {
                        Ok(_permit) => next.run(req).await,
                        Err(_) => axum::http::Response::builder()
                            .status(429)
                            .body(axum::body::Body::from("Too many requests"))
                            .unwrap(),
                    }
                }
            },
        ));
    }

    let addr = format!("0.0.0.0:{port}");
    info!("MCP HTTP/SSE transport listening on http://{addr}");
    info!("  POST /mcp        — JSON-RPC endpoint");
    info!("  POST /mcp/stream — Streamable HTTP (MCP 2025-03-26)");
    info!("  GET  /sse        — SSE event stream");
    info!("  GET  /health     — Health check");
    info!("  GET  /metrics    — Prometheus metrics");
    if rate_limit > 0 {
        info!("  Concurrency limit: {rate_limit} concurrent requests");
    }

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Graceful shutdown: listen for SIGTERM/SIGINT
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shut down gracefully");
    Ok(())
}

/// Wait for a shutdown signal (Ctrl+C or SIGTERM).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl+c");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => info!("Received Ctrl+C, initiating graceful shutdown"),
        () = terminate => info!("Received SIGTERM, initiating graceful shutdown"),
    }
}
