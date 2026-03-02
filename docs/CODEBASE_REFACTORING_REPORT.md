# MCPAtlas Codebase Refactoring & Quality Report

**Generated:** March 3, 2025  
**Scope:** Whole codebase (Rust crates, site, deploy)

---

## Executive Summary

| Category | Count | Priority |
|----------|-------|----------|
| Unused code | 5 items | Medium |
| Duplication | 4 areas | Medium |
| Modularization | 5 areas | Medium |
| Optimization | 5 areas | Low–Medium |
| Code quality | 7 areas | Medium |

---

## 1. Code Structure & Architecture

### 1.1 Rust Workspace

| Crate | Purpose | Status |
|-------|---------|--------|
| `mcp-atlas-core` | MCP server, tool handlers, JSON-RPC, transports | Core |
| `mcp-atlas-data` | Models, pipeline, ingestion, enrichment | Core |
| `mcp-atlas-search` | Tantivy full-text search | Core |
| `mcp-atlas-graph` | In-memory knowledge graph | Core |
| `mcp-atlas-vector` | Qdrant vector backend (optional) | Optional |
| `mcp-atlas-graph-surrealdb` | SurrealDB graph backend (optional) | Optional, not in `members` |
| `mcp-atlas-plugins` | WASM plugin host, manifest, registry | Core |
| `mcp-atlas-cli` | CLI sync/validate | Core |

**Note:** `mcp-atlas-graph-surrealdb` is an optional dep of `mcp-atlas-core` but not in workspace `members`. Adding it would make it explicit for development.

### 1.2 Site / Frontend

- **Stack:** Vite 7, React 18, TypeScript, Tailwind, Framer Motion
- **Routes:** `/` (LandingPage), `/docs` (DocsLayout + DocPage)
- **Layout:** `MainLayout` → `Navbar`, `Footer`, `Section`
- **Landing:** `FloatingIconsHero`, `Problem`, `Solution`, `FeaturesGrid`, `ArchitectureSection`, `BentoDemo`, `UseCases`, `RoadmapSection`, `CTA`

### 1.3 Deploy

- **Helm:** Two charts — `deploy/helm/cncf-mcp/` and `deploy/helm/mcp-atlas/`
- **Docker:** `deploy/docker/Dockerfile`, `docker-compose.yml`
- **CI:** `.github/workflows/` (ci, release, data-sync, pages, tag-on-main)

---

## 2. Unused Code & Dead Code

### 2.1 Rust

| Item | Location | Notes |
|------|----------|-------|
| `McpError` | `crates/mcp-atlas-core/src/error.rs` | Enum defined but never used; only `error::codes` is imported |
| `PluginRegistry` | `crates/mcp-atlas-plugins/src/registry.rs` | Exported, never imported elsewhere |
| `PluginRuntime` | `crates/mcp-atlas-plugins/src/runtime.rs` | Stub; only used when `runtime` feature is enabled (feature is empty) |
| `PluginHost` | `crates/mcp-atlas-plugins/src/host.rs` | Trait defined but no implementations |

### 2.2 `#[allow(dead_code)]` Usage

| File | Notes |
|------|-------|
| `mcp-atlas-vector/src/backend.rs` | "used when creating collection; kept for validation" |
| `mcp-atlas-data/src/summary.rs` | "used when building client; kept for potential future use" |
| `mcp-atlas-data/src/embeddings.rs` | "used when building client; kept for potential future use" |

### 2.3 Site / Frontend

| Item | Location | Notes |
|------|----------|-------|
| `GlowingEffect` | `site/src/components/ui/glowing-effect.tsx` | Not exported from `ui/index.ts`; not imported anywhere |
| `motion` package | `site/package.json` | Only used in `glowing-effect.tsx` via `animate` from `motion/react` — component is unused |

**Recommendation:** Remove `GlowingEffect` and the `motion` dependency; use `framer-motion` only (it supports `animate`). This reduces bundle size and dependency count.

---

## 3. Duplication & Reuse Opportunities

### 3.1 Helm Charts

- **`deploy/helm/cncf-mcp/`** vs **`deploy/helm/mcp-atlas/`** are nearly identical
- **Differences:** image repo (`ghcr.io/aryasoni98/mcp-atlas` vs `ghcr.io/mcp-atlas/mcp-atlas`), Chart metadata (home, sources, icon)
- **Recommendation:** Consolidate into one chart with configurable values, or clearly document which is canonical.

### 3.2 Duplicate Motion Libraries (Site)

- **`framer-motion`** and **`motion`** used together
- `framer-motion` is used in most components; `motion` only in unused `glowing-effect.tsx`
- **Recommendation:** Remove `motion`; standardize on `framer-motion`.

### 3.3 Tool Argument Extraction

Repeated pattern across tool handlers:

```rust
args.get("query").and_then(|q| q.as_str()).unwrap_or("");
args.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;
args.get("offset").and_then(|o| o.as_u64()).unwrap_or(0) as usize;
```

**Recommendation:** Add helpers in `tools/args.rs`:

```rust
pub fn parse_string_arg(args: &Value, key: &str, default: &str) -> &str { ... }
pub fn parse_u64_arg(args: &Value, key: &str, default: u64) -> u64 { ... }
pub fn parse_optional_str(args: &Value, key: &str) -> Option<&str> { ... }
```

### 3.4 Tool Schema Definitions

- Tool schemas in `core_tools_list()` are ~250 lines inline in `mod.rs`
- **Recommendation:** Move to `tools/schemas.rs` or JSON/YAML for reuse and maintenance.

---

## 4. Modularization & Refactoring

### 4.1 `tools/mod.rs` (High Priority)

- **~810 lines** in a single module
- Mixes: JSON-RPC dispatch, tool list, resources, prompts, completion

**Recommendation:** Split into:

| New Module | Responsibility |
|------------|----------------|
| `tools/dispatch.rs` | `handle_jsonrpc`, tool call routing |
| `tools/resources.rs` | Resource list, read, templates |
| `tools/completion.rs` | Completion handling |
| `tools/schemas.rs` | Tool definitions, `core_tools_list()` |

### 4.2 `search.rs` (~680 lines)

- Contains RRF, hybrid search, and many handlers
- **Recommendation:** Extract `reciprocal_rank_fusion` and hybrid search into `search::hybrid` or shared module.

### 4.3 Plugin System

- `PluginHost`, `PluginRegistry`, `PluginRuntime` are stubs
- `runtime` feature is empty (`runtime = []`)
- **Recommendation:** Either remove or clearly mark as "Phase 3 / experimental" in ROADMAP; wire plugin flow when ready.

### 4.4 Site Component Structure

- `components/landing/` and `components/ui/` are well-organized
- `components/motion/` and `components/diagram/` are separate
- **Recommendation:** Add barrel exports for `components/landing/` if not already used consistently.

### 4.5 `handle_jsonrpc` Complexity

- ~290 lines, many branches
- **Recommendation:** Split by method: `handle_initialize`, `handle_tools_list`, `handle_tools_call`, etc., and dispatch from a smaller match.

---

## 5. Optimization Opportunities

### 5.1 Site Bundle Size

- `manualChunks` in `vite.config.ts` includes `vendor: ["react", "react-dom", "framer-motion"]`; `motion` is not chunked
- **Recommendation:** Remove `motion` entirely (see §2.3).

### 5.2 Release Build

- `[profile.release]`: `lto = true`, `codegen-units = 1`, `strip = true`, `panic = "abort"`
- **Recommendation:** Ensure `opt-level = 3` (default) is sufficient; consider `opt-level = "z"` for smaller binaries if size matters.

### 5.3 Search Index

- `SearchIndex::build` runs on every startup
- **Recommendation:** Persist index to disk when possible and reload instead of rebuilding.

### 5.4 Pipeline Concurrency

- GitHub: 5 concurrent requests; Artifact Hub: 2
- **Recommendation:** Make configurable via env or config.

### 5.5 Lock Usage in Hot Paths

- `state.sessions.read().unwrap()`, `state.in_flight.write().unwrap()` in server and tools
- **Recommendation:** Prefer `expect("...")` with clear messages for lock failures; consider `DashMap` or similar for high-contention paths.

---

## 6. Code Quality Issues

### 6.1 `unwrap()` in Production Code

| Location | Context |
|----------|---------|
| `server.rs` | `sessions.read().unwrap()`, `write().unwrap()` — can panic on poisoned lock |
| `tools/mod.rs` | `in_flight.write().unwrap()`, `resource_subscriptions.write().unwrap()` |
| `search.rs` L29 | `partial_cmp(...).unwrap_or(...)` — acceptable (fallback to Equal) |

**Recommendation:** Replace lock `unwrap()` with `expect("lock poisoned: ...")` or handle `PoisonError`; tests may keep `unwrap()`.

### 6.2 Naming Consistency

- `cncf-mcp` vs `mcp-atlas` used inconsistently (Helm charts, docs)
- **Recommendation:** Standardize on `mcp-atlas` as canonical; document `cncf-mcp` as legacy alias if needed.

### 6.3 Coupling

- `AppState` depends on many concrete types
- Tool handlers use `Arc<AppState>` directly
- **Recommendation:** Introduce traits for search, graph, embeddings to reduce coupling and improve testability.

### 6.4 Rust Edition

- `edition = "2024"` in workspace; `rust-version = "1.85"`
- **Recommendation:** Confirm Rust 2024 is stable in your CI and contributor environments.

---

## 7. Suggested Action Plan

### Phase 1 — Quick Wins (1–2 days)

1. Remove `GlowingEffect` and `motion` dependency from site
2. Add `mcp-atlas-graph-surrealdb` to workspace `members` (optional)
3. Use `McpError` in tool/resource error paths or remove it
4. Replace lock `unwrap()` with `expect()` in server and tools

### Phase 2 — Consolidation (3–5 days)

1. Merge or differentiate Helm charts (`cncf-mcp` vs `mcp-atlas`)
2. Split `tools/mod.rs` into dispatch, resources, completion, schemas
3. Add argument extraction helpers (`parse_string_arg`, etc.)
4. Extract tool schemas to `tools/schemas.rs`

### Phase 3 — Deeper Refactor (1–2 weeks)

1. Split `handle_jsonrpc` by method
2. Extract hybrid search / RRF into `search::hybrid`
3. Introduce traits for search, graph, embeddings
4. Document or wire plugin stubs (`PluginRegistry`, `PluginRuntime`, `PluginHost`)

---

## 8. Appendix: File Counts

| Area | Files |
|------|-------|
| Rust crates | ~40 `.rs` files |
| Site | ~35 `.tsx`/`.ts` files |
| Helm | 2 charts, 23 template files |
| CI | 5 workflow files |

---

*Report generated from static analysis and exploration of the CNCF_MCP codebase.*
