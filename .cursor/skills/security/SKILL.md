---
name: security
description: Security and compliance for MCPAtlas: threat model, RBAC, audit, supply chain, input validation, plugin sandbox. Use when implementing auth, hardening, or reviewing security-sensitive code.
---

# Security — MCPAtlas

Apply BluePrint §8 (Security & Compliance) and DEEP_PLAN §9.

## Threat model (BluePrint §8)

| Threat | Mitigation |
|--------|------------|
| Malicious MCP client / crafted queries | Input validation, query length limits (e.g. 1KB), regex-safe search |
| Plugin exfiltration via network | WASM sandbox, network allowlist per plugin manifest |
| GitHub token leak | Never log; never in responses; sealed secrets / external store in K8s |
| DoS via expensive queries | Per-client rate limit (token bucket), request timeout (e.g. 30s), semaphore |
| Supply chain (deps, image) | cargo-deny, cargo-audit, SBOM, cosign, pinned base images |
| WASM escape | Wasmtime sandbox, no WASI FS, CPU epoch limits |

## Authentication and RBAC

- **STDIO**: No auth (local trust).
- **HTTP**: Auth middleware. Roles: Anonymous (read), Reader (read), PluginAdmin (+ plugin install), Admin (all + /admin). API key in `Authorization: Bearer <key>`; validate against config or Redis. No auth = reject or Anonymous.
- Store key hashes (e.g. bcrypt), not raw keys. Optional scopes for tool-level access.

## Input validation

- Validate and bound all MCP params: string length, allowed enums, numeric ranges. Reject with clear JSON-RPC error.
- Sanitize search/graph inputs; use parameterized queries. No user-controlled format strings in SQL or index queries.
- Request body size limit (e.g. 1MB). Return 413 or equivalent when exceeded.

## Audit logging

- Log tool calls: timestamp, session_id, client_ip, method, tool_name, params_hash (SHA-256, not raw params), response_code, latency_ms. Structured JSON to stderr or OTLP.
- Never log: API keys, tokens, full request params, PII.

## Supply chain

- **Build**: Hermetic where possible; pinned base image. CI: no self-hosted runners for release.
- **Signing**: Cosign for binaries and container image. Attest SBOM (SPDX).
- **Dependencies**: cargo-deny (licenses, duplicates, tree); cargo-audit for advisories. Fix or document exceptions.
- **SLSA**: Use slsa-github-generator (or equivalent) for provenance in release workflow.

## Plugin sandbox

- Network: only hosts declared in plugin manifest; host validates URL before proxy.
- Memory: max_memory_mb via Wasmtime StoreLimits.
- CPU: max_cpu_ms per call via epoch interruption.
- No filesystem. Plugin signing (e.g. .wasm.sig) before load. Log every plugin tool call (name, tool, latency, outcome).

When adding auth, new endpoints, or plugin hooks: check threat model and audit surface.
