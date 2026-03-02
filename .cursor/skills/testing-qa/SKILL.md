---
name: testing-qa
description: Test strategy for CNCF MCP: unit, integration, e2e, benchmarks, fixtures. Use when adding tests or debugging failures.
---

# Testing & QA — CNCF MCP

Keep quality bar high without blocking local dev (BluePrint §9, DEEP_PLAN §10).

## Test layout

- **Unit**: Next to code in `#[cfg(test)] mod tests`. Test pure logic, parsers, and small helpers. Use `#[tokio::test]` for async; same runtime config as prod where it matters.
- **Integration**: `tests/` at crate or workspace root. Test HTTP transport, MCP request/response, and multi-crate flows. Use fixture landscape (small YAML) and `--skip-github` so tests don’t hit network.
- **E2E** (optional): Full server start + client script or harness; run in CI only; cache pre-built binary or use short timeout.

## Fixtures and isolation

- **Landscape**: Small `landscape.yml` in `tests/fixtures/` or `data/` (e.g. 5–10 projects). Commit it; don’t fetch in tests.
- **Cache**: Use temp dir or in-memory backend so tests don’t share state. Clear or recreate per test when order matters.
- **Secrets**: No real tokens. Use placeholders or env vars that tests set to dummy values. Mock HTTP for GitHub/Artifact Hub when testing enrichment.

## What to test

- **Parsing**: landscape YAML → Project list; maturity from `project:` and `extra`; error on invalid YAML.
- **Search**: Index fixture projects; run search_projects with query, filters (category, maturity, min_stars, language); assert result shape and that filters are applied.
- **Tools**: Call each tool with valid and invalid params; assert success shape or JSON-RPC error with `data`. Pagination (limit/offset, _meta) where applicable.
- **Resources**: Resolve cncf:// URIs; assert status and body shape or error.
- **Protocol**: initialize returns correct protocolVersion; batch request returns array; notifications/cancelled doesn’t panic.

## Benchmarks

- **Search**: `cargo bench -p cncf-mcp-search`. Baseline for index build and query latency; guard against regressions in hot path.
- **Load**: Optional k6/wrk scenario (e.g. 100 concurrent search, mixed tool calls). Run in CI on schedule or tag; fail on regression vs baseline.

## Commands

```bash
cargo test --workspace
cargo test -p cncf-mcp-core -- --test-threads=4
cargo bench -p cncf-mcp-search
```

- CI runs full workspace tests. Local: run tests for crates you changed; run bench after search/index changes.
- Flaky tests: fix or quarantine; don’t leave ignore without a ticket.

When adding a tool or pipeline step: add tests that cover the happy path and at least one invalid or edge input.
