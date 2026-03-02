use clap::Parser;

/// Transport protocol for the MCP server.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Transport {
    /// STDIO transport — communicates over stdin/stdout (local dev).
    Stdio,
    /// Server-Sent Events over HTTP (remote clients).
    Sse,
}

/// Graph storage backend (Blueprint §2b).
#[derive(Debug, Clone, Default, clap::ValueEnum)]
pub enum GraphBackendKind {
    /// In-memory graph (default).
    #[default]
    Mem,
    /// SurrealDB (requires --surreal-url).
    Surreal,
}

/// CNCF Landscape MCP Server configuration.
#[derive(Debug, Parser)]
#[command(name = "cncf-mcp", about = "MCP Server for the CNCF Landscape")]
pub struct AppConfig {
    /// Transport protocol to use.
    #[arg(long, default_value = "stdio")]
    pub transport: Transport,

    /// Port for SSE transport.
    #[arg(long, default_value = "3000")]
    pub port: u16,

    /// GitHub personal access token for API enrichment.
    #[arg(long, env = "GITHUB_TOKEN")]
    pub github_token: Option<String>,

    /// Path to a local landscape.yml file (skips fetching from GitHub).
    #[arg(long, env = "CNCF_LANDSCAPE_FILE")]
    pub landscape_file: Option<String>,

    /// Skip GitHub enrichment during startup (faster, less data).
    #[arg(long, default_value = "false")]
    pub skip_github: bool,

    /// Directory for caching landscape data (default: ~/.cache/cncf-mcp).
    #[arg(long, env = "CNCF_MCP_CACHE_DIR")]
    pub cache_dir: Option<String>,

    /// Maximum cache age in seconds before refreshing (default: 86400 = 24h).
    #[arg(long, default_value = "86400")]
    pub max_cache_age: u64,

    /// Rate limit: max requests per second for HTTP transport (0 = unlimited).
    #[arg(long, default_value = "50")]
    pub rate_limit: u64,

    /// Graph backend: mem (in-memory) or surreal (SurrealDB).
    #[arg(long, default_value = "mem", env = "CNCF_MCP_GRAPH_BACKEND")]
    pub graph_backend: GraphBackendKind,

    /// SurrealDB URL when graph-backend is surreal (e.g. ws://localhost:8000).
    #[arg(long, env = "CNCF_MCP_SURREAL_URL")]
    pub surreal_url: Option<String>,

    /// Enable Artifact Hub enrichment (Helm packages per project). Slower startup.
    #[arg(long, default_value = "false", env = "CNCF_MCP_ARTIFACT_HUB")]
    pub artifact_hub: bool,

    /// Qdrant URL for vector search (e.g. http://localhost:6334). Enables hybrid search when set with --embedding-api-base.
    #[arg(long, env = "CNCF_MCP_QDRANT_URL")]
    pub qdrant_url: Option<String>,

    /// Embedding API base URL (Ollama: http://localhost:11434, OpenAI-compatible: https://api.openai.com). Enables hybrid search when set.
    #[arg(long, env = "CNCF_MCP_EMBEDDING_API_BASE")]
    pub embedding_api_base: Option<String>,

    /// Embedding API key (for OpenAI-compatible APIs). Optional for Ollama.
    #[arg(long, env = "CNCF_MCP_EMBEDDING_API_KEY")]
    pub embedding_api_key: Option<String>,

    /// Embedding model name (e.g. nomic-embed-text, text-embedding-3-small).
    #[arg(long, env = "CNCF_MCP_EMBEDDING_MODEL", default_value = "nomic-embed-text")]
    pub embedding_model: String,

    /// Enable LLM summary enrichment during pipeline (slower; uses fallback when API unavailable).
    #[arg(long, default_value = "false", env = "CNCF_MCP_SUMMARY_ENABLED")]
    pub summary_enabled: bool,

    /// Summary API base URL (e.g. https://api.openai.com or http://localhost:11434 for Ollama).
    #[arg(long, env = "CNCF_MCP_SUMMARY_API_BASE")]
    pub summary_api_base: Option<String>,

    /// Summary API key (for OpenAI-compatible). Optional for Ollama.
    #[arg(long, env = "CNCF_MCP_SUMMARY_API_KEY")]
    pub summary_api_key: Option<String>,

    /// Summary model (e.g. gpt-4o-mini, llama3.2).
    #[arg(long, env = "CNCF_MCP_SUMMARY_MODEL")]
    pub summary_model: Option<String>,

    /// Max projects to summarize per pipeline run (0 = no limit).
    #[arg(long, default_value = "0", env = "CNCF_MCP_SUMMARY_MAX_PER_RUN")]
    pub summary_max_per_run: usize,
}
