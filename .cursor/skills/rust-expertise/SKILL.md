---
name: rust-expertise
description: Deep Rust patterns for CNCF MCP: async/tokio, error handling, serde, performance, testing. Use when implementing or reviewing Rust code, optimizing hot paths, or debugging concurrency.
---

# Rust Expertise — CNCF MCP

Apply Rust best practices for a long-lived, high-concurrency MCP server (DEEP_PLAN §3, BluePrint §9).

## Error handling

- **Application code**: `anyhow::Result`, `.context("...")` on fallible calls. No `unwrap()` in non-test code.
- **Libraries / reusable crates**: `thiserror` enums, implement `Display` and `Error`. Use `#[from]` for wrapping.
- **MCP tool handlers**: Map domain errors to JSON-RPC error with structured `data`; preserve user-facing message, log full context.

## Async and Tokio

- Use existing `#[tokio::main]`. For SSE/server: `flavor = "multi_thread"`, tune `worker_threads` (e.g. `num_cpus` or `--workers`). For STDIO binary: `current_thread` is enough.
- Prefer `Arc<AppState>` (or trait object) for shared state; avoid deep clones in hot paths. Use `tokio::sync` (RwLock, broadcast) only where needed; avoid blocking the runtime.
- Spawn background tasks for pipeline/refresh; use `tokio::select!` or channels for cancellation and shutdown (SIGTERM/SIGINT).

## Serialization and zero-copy

- All public API types: `#[derive(Debug, Clone, Serialize, Deserialize)]`. Use `#[serde(alias = "...")]` for YAML/JSON field names (e.g. `project:` → maturity).
- For high-throughput responses: consider `serde_json::value::RawValue` to avoid re-serializing cached JSON. Keep payloads bounded (pagination, limit).

## Performance (BluePrint §9)

- **Tantivy**: Prefer mmap persistent index when supported; avoid full rebuild every startup. Tune writer heap; use FAST field for exact lookups; boost name over description.
- **Hot paths**: Pre-compute health scores in pipeline; cache search results (e.g. LRU 128 entries); lazy graph build on first access.
- **Profiling**: `cargo flamegraph` for CPU; `cargo instruments` (macOS) for allocations. Use `cargo bench -p cncf-mcp-search` for search changes.
- **Binary size**: LTO, strip, static linking. Target &lt;15MB.

## Testing

- Unit tests next to code: `#[cfg(test)] mod tests`. Use `#[tokio::test]` for async; match runtime config if it matters.
- Integration tests in `tests/` or crate `tests/`; use `--skip-github` and fixture landscape when possible.
- Benchmarks in `benches/` or `#[bench]`; guard against regressions in hot paths.

## Safety and dependencies

- **No `unsafe`** in core. Enforce in CI (e.g. `forbid(unsafe_code)`).
- Minimal new dependencies; justify in PR. Use `cargo-deny` and `cargo-audit`; fix or document advisories.
- Workspace: `core` (MCP server), `data` (models, pipeline), `search` (Tantivy), `graph`, `plugins`, `cli`. Keep public APIs small and trait-based where swap is needed.

When reviewing or implementing: prefer clarity and correctness first; then measure before optimizing.
