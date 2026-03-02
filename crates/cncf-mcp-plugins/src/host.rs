//! Trait for host callbacks used by WASM plugins (Phase 3).

use anyhow::Result;

/// Host API that plugins can call. Implemented by the MCP server core.
pub trait PluginHost: Send + Sync {
    /// Run search_projects and return JSON result.
    fn search_projects(&self, query: &str) -> Result<String>;
    /// Run get_project and return JSON result.
    fn get_project(&self, name: &str) -> Result<String>;
    /// Log a message (level: debug, info, warn, error).
    fn log(&self, level: &str, message: &str);
    /// HTTP GET if the URL is allowlisted (e.g. artifacthub.io). Returns body as string or error.
    fn http_get(&self, url: &str) -> Result<String>;
}
