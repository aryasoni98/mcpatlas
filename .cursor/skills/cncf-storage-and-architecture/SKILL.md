---
name: cncf-storage-and-architecture
description: Apply trait-based storage and service patterns for MCPAtlas: GraphBackend, CacheBackend, VectorBackend. Use when adding persistence, graph, search backends, or refactoring data layer.
---

# MCPAtlas Storage and Architecture

Follow BluePrint §2 (Final Target Architecture) and §2a–2e for storage and data layer.

## Trait-based storage

Introduce or implement backend traits instead of concrete types where swap is needed:

- **GraphBackend**: `get_edges`, `find_path`, `stats`, `upsert_edges`. Implementations: in-memory (current), SurrealDB.
- **VectorBackend**: `search(embedding, limit)`, `upsert(id, embedding, metadata)`. Implementation: Qdrant.
- **CacheBackend**: `get`, `set` (with TTL), `delete`. Implementations: in-memory, file, Redis.

Keep core and tools depending on traits; use config (e.g. `--graph-backend mem|surreal`) to choose implementation. Local/default remains in-memory.

## Data pipeline and enrichment

- Pipeline: landscape YAML → GitHub (and optional CNCF API, Artifact Hub) → optional LLM summaries/embeddings. Schedule: hourly landscape, 6h GitHub, daily enrichment where applicable.
- Use **LlmEnricher** (or equivalent) behind a flag; cache summaries and embeddings; support OpenAI-compatible and local (e.g. Ollama) endpoints.
- Prefer incremental updates (content hash, skip unchanged) over full re-index when adding persistence.

## Service layer

- Keep Tool Router → SearchService, GraphService, HealthService, etc. Services use storage traits, not concrete DBs. Plugin host and Notifier sit behind the same service layer.

## References

- BluePrint.md §2 (Target Architecture), §2a–2e (Storage, SurrealDB, Qdrant, LLM, Artifact Hub).
- DEEP_PLAN.md §2 (System Architecture), §4 (Data Pipeline).
