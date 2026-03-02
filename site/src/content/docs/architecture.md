# Architecture

## Overview

- **Clients**  Claude, Cursor, VS Code, or any MCP client connect via STDIO, SSE, or Streamable HTTP.
- **Transport**  Content-Length framing (STDIO), SSE, or HTTP with session support (MCP 2025-03-26).
- **Core**  Tool router, resource provider, prompt templates; services for search, graph, health, recommendations, trends.
- **Storage**  Trait-based: search (Tantivy), graph (in-memory or SurrealDB), vector (optional Qdrant), cache (file or Redis).
- **Pipeline**  Landscape YAML → GitHub REST → Artifact Hub (optional) → embeddings (optional). Schedule: landscape refresh, GitHub enrichment, cache TTL.

## Data flow

1. On startup (or cache miss): load landscape, optionally enrich via GitHub, build search index and graph.
2. Tool/resource requests hit the core; core queries search, graph, or cache as needed.
3. Optional: vector search (Qdrant) and hybrid BM25+vector with RRF merge.

## Deployment

Single binary; no database required for default in-memory mode. For scale: add Redis (sessions/cache), SurrealDB (graph), Qdrant (vector). See [Deployment](/docs/deployment).
