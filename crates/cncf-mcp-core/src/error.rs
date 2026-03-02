use thiserror::Error;

/// Structured error types for the CNCF MCP server.
#[derive(Debug, Error)]
pub enum McpError {
    /// A required parameter was missing from the tool call.
    #[error("Missing required parameter: {0}")]
    MissingParam(String),

    /// The requested project was not found in the landscape.
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    /// The requested resource URI is unknown.
    #[error("Unknown resource URI: {0}")]
    UnknownResource(String),

    /// The requested tool name is unknown.
    #[error("Unknown tool: {0}")]
    UnknownTool(String),

    /// The requested prompt name is unknown.
    #[error("Unknown prompt: {0}")]
    UnknownPrompt(String),

    /// The JSON-RPC method is not recognized.
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Search index error.
    #[error("Search error: {0}")]
    SearchError(String),

    /// Data pipeline error.
    #[error("Data pipeline error: {0}")]
    PipelineError(#[from] anyhow::Error),
}

/// JSON-RPC 2.0 error codes used by the server.
pub mod codes {
    /// Parse error — invalid JSON was received.
    pub const PARSE_ERROR: i32 = -32700;
    /// Method not found.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Server error — tool execution failed.
    pub const SERVER_ERROR: i32 = -32000;
}
