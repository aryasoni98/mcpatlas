---
name: solution-architect
description: System design and tradeoffs for CNCF MCP: storage backends, deployment tiers, scalability, security layers. Use when designing features, choosing technologies, or reviewing architecture.
---

# Solution Architect — CNCF MCP

Apply system-design thinking per BluePrint §2 and DEEP_PLAN §2, §8, §9.

## Target architecture (north star)

- **Clients** → Transport (STDIO | SSE | Streamable HTTP) → Auth middleware → Core (Tool Router, Resources, Prompts, Services) → Plugin Host, Notifier, Completion.
- **Core** depends on **storage traits** (Search, Graph, Vector, Cache), not concrete DBs. Implementations: local (embedded) vs cloud (SurrealDB, Redis, Qdrant).
- **Data pipeline**: Landscape YAML → GitHub (and optional CNCF API, Artifact Hub) → optional LLM/embeddings; async, scheduled; cache TTL-based (file or Redis).
- **Stateless HTTP**: Session and shared state in Redis so multiple pods can serve any request.

## Deployment tiers

| Tier   | Use case        | Components                                      | Memory / startup   |
|--------|------------------|--------------------------------------------------|--------------------|
| **Tier 1** | Local dev        | Single binary, embedded Tantivy + graph, in-memory cache | ~100MB, &lt;500ms |
| **Tier 2** | Team server      | Docker Compose: cncf-mcp replicas, Redis, optional Meilisearch/Qdrant, reverse proxy | — |
| **Tier 3** | Cloud / K8s      | Deployment + HPA, Redis/SurrealDB/Qdrant, CronJob pipeline, Ingress, Operator | Scale 3–50 pods |

Choose defaults and flags (e.g. `--graph-backend mem|surreal`) so Tier 1 works out of the box; Tier 2/3 opt in to external backends.

## Key tradeoffs

- **Graph**: In-memory (simple, no deps) vs SurrealDB (persistence, graph queries, Rust-native). Use trait so both are pluggable.
- **Search**: Tantivy (embedded, Rust) vs Meilisearch (external). Default Tantivy; mmap persistent index for large scale to avoid full rebuild on restart.
- **Vector**: Optional Qdrant for semantic search. BM25-only mode must work without it.
- **Plugin**: WASM (Extism/Wasmtime) for sandbox and multi-language; narrow host ABI, no filesystem, network allowlist, CPU/memory limits.
- **Auth**: STDIO = no auth (local trust). HTTP = API key / OAuth; RBAC (Anonymous, Reader, PluginAdmin, Admin) per BluePrint §4c.

## Risks to mitigate (BluePrint §1)

- In-memory-only state → add persistence (SurrealDB, Redis) and/or mmap index.
- O(n²) graph build → lazy construction or incremental edges; cap or batch by subcategory.
- Single-process ceiling → move sessions and coordination to Redis.
- GitHub sync blocking startup → cache aggressively; optional `--skip-github`; background refresh.
- No auth on HTTP → add auth middleware and rate limiting per client.

When proposing a change, state which tier it serves and how it affects scalability, security, and operability.
