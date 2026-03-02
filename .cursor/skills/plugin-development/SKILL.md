---
name: plugin-development
description: WASM plugin system for CNCF MCP: Extism, manifest, host functions, sandbox. Use when implementing plugin host or writing plugin docs/SDK.
---

# Plugin Development — CNCF MCP

Apply BluePrint §6 (WASM Plugin System) and DEEP_PLAN §7.

## Runtime: Extism over raw Wasmtime

- Use Extism for stable host ABI, multi-language PDKs, and memory management. Plugins register tools; host dispatches tool calls into WASM and returns results.
- Host exposes **host functions** only (e.g. host_search, host_get_project, host_http_get, host_log, host_cache_get/set). No filesystem; network only to allowlisted hosts per manifest.

## Manifest (plugin.toml)

```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
min_host_version = "0.2.0"

[wasm]
path = "my_plugin.wasm"
hash = "sha256:..."   # integrity check

[permissions]
network = ["artifacthub.io"]
max_memory_mb = 64
max_cpu_ms = 5000
allow_cache = true

[[tools]]
name = "my_tool"
description = "..."
```

- **Permissions**: Explicit allowlist for network hosts; host_http_get validates URL against it. Enforce max_memory_mb (Wasmtime StoreLimits) and max_cpu_ms (epoch interruption).
- **Signing**: Require .wasm.sig (e.g. ed25519); host verifies before load. Document key distribution and rotation.

## Plugin lifecycle

- **Install**: Verify signature → load WASM → validate manifest → register tools with MCP router.
- **Tool call**: Router dispatches to plugin; host runs in sandbox (memory/CPU limits); host functions called from guest; return result or error. Log: plugin name, tool, latency, outcome.
- **Uninstall**: Deregister tools, drop WASM, remove files. No lingering state.

## Host functions (design)

- **host_search**, **host_get_project**: Read-only access to core data; return JSON or error. Rate-limit if needed.
- **host_http_get**: URL + optional headers. Host checks URL against permissions.network; proxy request; return body. Timeout and size limit.
- **host_log**: Forward to tracing with plugin name prefix.
- **host_cache_get**, **host_cache_set**: Key-value with TTL; namespace by plugin to avoid collisions.
- No host_fs, host_exec, or arbitrary syscalls.

## Guest (plugin author)

- Plugin implements: plugin_info, register_tools, handle_tool_call. Use Extism PDK (Rust, Go, JS, etc.). Compile to wasm32-wasi or target supported by host.
- Document: how to build, how to run locally with cncf-mcp, manifest schema, available host functions and limits, security (no FS, network allowlist, CPU/memory caps).

## References

- BluePrint.md §6 (WASM Plugin System Design).
- DEEP_PLAN.md §7 (Plugin System). Example plugins: helm-analyzer, GitHub deep-dive, vuln-scanner.

When adding a host function or changing manifest schema: update SDK docs, example plugin, and security section.
