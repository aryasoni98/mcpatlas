# Plugin development

Extend the CNCF MCP server with plugins (WASM/Extism).

## Overview

Plugins run in a sandboxed runtime and can register tools and resources. The host exposes a stable ABI and capability-based permissions.

## Getting started

1. **Target**  Build your plugin for `wasm32-wasip1` (or the target supported by the host).
2. **Manifest**  Provide a `plugin.toml` (or equivalent) with name, version, and declared capabilities.
3. **Host**  Use the CLI or server config to load the plugin (e.g. `plugin install`, or a config list of plugin paths).

## SDK and ABI

- Use the plugin SDK (when published) for the recommended interface and host functions.
- Document any custom host functions and version guarantees in the repo (e.g. in `crates/cncf-mcp-plugins/`).

## Security

- Plugins are sandboxed; the host limits filesystem, network, and other capabilities per manifest.
- Do not expose secrets to plugins unless required; follow the principle of least privilege.

## Related

- [Architecture](/docs/architecture)  Where the plugin runtime fits.
- [Contributing](/docs/contributing)  How to submit a plugin or SDK improvements.
