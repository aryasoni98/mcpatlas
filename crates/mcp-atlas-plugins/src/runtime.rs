//! Extism-based WASM plugin runtime (Phase 3). Requires feature `runtime`.
//!
//! When `runtime` is enabled, this module will load WASM via Extism, register
//! host functions (host_search_projects, host_get_project, host_log, host_http_get),
//! and dispatch MCP tool calls to the plugin's `run` export. Until the full
//! Extism host-function wiring is completed, PluginRuntime is a stub.

use anyhow::Result;

use crate::manifest::PluginManifest;

/// Default memory limit for plugins (64 MiB).
pub const DEFAULT_PLUGIN_MEMORY: u64 = 64 * 1024 * 1024;

/// Default call timeout (30s).
pub const DEFAULT_PLUGIN_TIMEOUT_SECS: u64 = 30;

/// WASM plugin runtime. With full Extism wiring this will hold an Extism `Plugin`
/// and dispatch tool calls to the plugin's `run` export.
#[derive(Debug)]
pub struct PluginRuntime {
    _wasm: Vec<u8>,
    manifest: PluginManifest,
}

impl PluginRuntime {
    /// Build a runtime from WASM bytes and manifest. Registers host functions that call into `host`.
    /// Stub: validates manifest and stores WASM; full Extism Plugin creation is TODO.
    pub fn new(
        wasm: &[u8],
        manifest: PluginManifest,
        _host: std::sync::Arc<dyn crate::host::PluginHost>,
    ) -> Result<Self> {
        if manifest.tools.is_empty() {
            anyhow::bail!("plugin manifest must list at least one tool");
        }
        Ok(Self {
            _wasm: wasm.to_vec(),
            manifest,
        })
    }

    /// Call a tool by name with JSON arguments. Returns JSON result or a structured error.
    /// Stub: returns error until Extism Plugin is wired.
    pub fn call(&self, _tool_name: &str, _args_json: &str) -> Result<String> {
        anyhow::bail!(
            "plugin runtime not yet wired (Extism host functions TODO); plugin '{}' has tools: {:?}",
            self.manifest.name,
            self.manifest.tools
        )
    }

    /// Manifest for this plugin.
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}
