# Plugin Development

The CNCF MCP Server is designed to support **WASM plugins** (e.g. via [Extism](https://extism.org/)) so that contributors can add tools without changing the core. This is planned; the plugin host interface and manifest format are described in [MCP_BLUEPRINT.md](../../MCP_BLUEPRINT.md) §6.

## Planned capabilities

- **Plugin manifest** (TOML) — name, version, WASM path, permissions (network allowlist, memory/CPU limits).
- **Host functions** — e.g. `host_search`, `host_get_project`, `host_http_get`, `host_log`, `host_cache_get`/`host_cache_set`.
- **Sandbox** — Network only to allowed hosts; no filesystem; CPU/memory limits; optional plugin signing.

When the plugin system is released, SDK and examples will be documented here and in the main repository. See the repo root **MCP_BLUEPRINT.md** §6 for the full design.
