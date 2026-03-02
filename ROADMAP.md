# Roadmap

Public roadmap for MCPAtlas. Aligned with [MCP_BLUEPRINT.md](MCP_BLUEPRINT.md) and [DEEP_PLAN.md](DEEP_PLAN.md).

## Phase 1: Foundation (complete)

**Goal:** Installable, usable MCP server with core data and search.

- [x] Core MCP server (STDIO, SSE, Streamable HTTP)
- [x] 14 tools: search, get_project, compare, alternatives, health, suggest_stack, trends, graph, get_good_first_issues, get_migration_path
- [x] Resources and prompts
- [x] Docker image and release workflow (multi-arch binaries + container + SBOM)
- [x] Documentation (in-app docs at /docs), CONTRIBUTING, SECURITY, GOVERNANCE
- [x] Homebrew formula (template; tap TBD)
- [x] CI: site (landing + in-app docs) deploy via Pages workflow (enable Pages from Actions in repo settings)
- [x] Tech debt: `min_stars` and `language` filters wired in search handler + exposed in tool schema
- [x] Tech debt: protocol version negotiation (returns client's requested version)
- [x] Tech debt: `logging/setLevel` wired to tracing-subscriber reload handle

## Phase 2: Intelligence (nearly complete)

**Goal:** Semantic search, health scoring, richer data, CNCF Sandbox application.

- [x] Storage trait abstraction (GraphBackend, CacheBackend, VectorBackend in `mcp-atlas-data`; `ProjectGraph` implements `GraphBackend`)
- [x] SurrealDB integration for graph (optional `--graph-backend surreal`, feature `graph-surrealdb`; embedded kv-mem)
- [x] Qdrant + embeddings, hybrid BM25 + vector search (`mcp-atlas-vector` crate; `--qdrant-url` + `--embedding-api-base`; RRF merge; optional feature `vector-qdrant`)
- [x] Artifact Hub integration (Helm packages per project; `--artifact-hub`; pipeline step with rate limiting)
- [x] LLM summaries (Ollama/OpenAI-compatible), pipeline integration, content-hash cache skip, fallback
- Apply for CNCF Sandbox

## Phase 3: Extensibility

**Goal:** Plugin ecosystem, new tools, operator, contributor growth.

- WASM plugin system (Extism): `PluginHost` trait, `PluginRuntime` stub, manifest; full host-function wiring TODO
- [x] Dynamic plugin tools: `DynamicTool`, `register_plugin_tool`, tools/list merge, tools/call dispatch
- [x] Tools: get_good_first_issues, get_migration_path
- [x] Contributor automation: PULL_REQUEST_TEMPLATE.md, CODEOWNERS, plugin-development docs
- Built-in plugins (e.g. Helm analyzer)
- Kubernetes operator (CRD, reconciliation)

## Phase 4: Scale & maturity

**Goal:** Production-grade, horizontal scaling, security hardening.

- Redis for shared state (sessions, cache), multi-pod scaling
- [x] Helm chart (deployment, service, ingress, HPA, PDB; Redis option TBD)
- [x] Helm hardening: values.schema.json, Pod/container securityContext (runAsNonRoot, drop all caps, seccomp)
- [x] Audit logging: structured tool_call events (tool, params_hash, status, latency_ms) to stderr; AuditLogger trait + StderrAuditLogger
- RBAC, optional auth
- SLSA Level 3 provenance, cosign signing
- Performance tuning (persistent Tantivy index, search cache)

## Success metrics (targets)

| Metric | 3 mo | 6 mo | 12 mo |
|--------|------|------|-------|
| GitHub stars | 500 | 2,000 | 10,000 |
| Monthly downloads | 1K | 10K | 100K |
| Contributors | 5 | 20 | 50+ |
| MCP tools | 14 | 14+ | 20+ |
| Plugins | 0 | 3 | 15+ |
| CNCF status | — | Sandbox | Incubating prep |

---

For detailed week-by-week execution see [MCP_BLUEPRINT.md §11](MCP_BLUEPRINT.md).
