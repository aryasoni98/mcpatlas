---
name: mcp-expertise
description: MCP protocol depth for MCPAtlas: tools, resources, prompts, JSON-RPC 2.0, transports, completion, cancellation. Use when adding or changing MCP surface, debugging client compatibility, or implementing spec behavior.
---

# MCP Expertise — MCPAtlas

Apply Model Context Protocol knowledge for spec compliance and client compatibility (BluePrint §3, DEEP_PLAN §5–6).

## Protocol basics

- **Wire format**: JSON-RPC 2.0. Request: `jsonrpc`, `id`, `method`, `params`. Response: `jsonrpc`, `id`, `result` or `error`. Notifications: `id` is null.
- **STDIO**: Content-Length framing (`Content-Length: N\r\n\r\n{json}`). Logs to stderr; only JSON-RPC to stdout.
- **SSE**: Server-sent events; client sends JSON-RPC in request body or query. Connection lifecycle and reconnection per spec.
- **Streamable HTTP (MCP 2025-03-26)**: Session via `Mcp-Session-Id`; streaming request/response. Support version negotiation.

## Version negotiation

- Client sends `initialize` with `protocolVersion` (e.g. `"2024-11-05"` or `"2025-03-26"`). Server **must** return the same `protocolVersion` in `initialize` result — do not hardcode a single version. Non-compliance is listed tech debt in BluePrint.

## Tools

- **Schema**: Each tool has `name`, `description`, `inputSchema` (JSON Schema). Handler receives `params`; returns JSON-serializable result or JSON-RPC error.
- **Placement**: One module per domain under `crates/mcp-atlas-core/src/tools/`. Register all tools at server startup.
- **Errors**: Use structured `data` in JSON-RPC error for tool-specific codes and hints. Validate inputs (length, enums, types) before calling backend.
- **Pagination**: When returning lists, support `limit`/`offset` and include `_meta` (e.g. total, limit, offset) in result.

## Resources

- **URI scheme**: `cncf://` (e.g. `cncf://landscape/overview`, `cncf://projects/{name}`, `cncf://categories/{category}`). Template params validated and documented.
- **Handlers**: Return content (e.g. JSON) and MIME type. Support `resources/subscribe`; actual push requires notification bus (BluePrint: transport-agnostic emitter not yet implemented — subscriptions tracked but not pushed).

## Prompts

- **Templates**: Name, description, arguments. Server returns template with placeholders; client fills and uses. Document arguments and example usage.
- **Existing**: evaluate_tool, plan_migration, review_stack, onboard_contributor. Add new prompts in prompts module and register.

## Completion

- `completion/complete` for partial input: project names, categories, maturity levels, relation types. Use to improve UX in clients that support completion.

## Batching and cancellation

- **Batch**: JSON-RPC allows multiple requests in one payload. Process in order; return array of responses. Preserve `id` for each.
- **Cancellation**: Handle `notifications/cancelled` with request `id`; cancel in-flight work when supported by transport. Track in-flight by id; timeouts (e.g. 30s) as fallback.

## CORS and HTTP transport

- CORS: Allow origin, GET/POST/DELETE/OPTIONS per BluePrint. Reject non-JSON POST with 415. Rate limit (semaphore and optionally per-client). Trace requests (e.g. TraceLayer). Expose `/metrics` (Prometheus). Graceful shutdown on SIGTERM/SIGINT.

When adding a tool/resource/prompt: document schema, list in CLAUDE.md and docs, and ensure protocol version and error shape stay spec-compliant.
