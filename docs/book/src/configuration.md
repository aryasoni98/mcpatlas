# Configuration

## Environment variables

| Variable | Description |
|----------|-------------|
| `GITHUB_TOKEN` | Optional. Improves rate limits for GitHub enrichment (stars, language, etc.). |
| `MCP_ATLAS_CACHE_DIR` | Cache directory for landscape and enrichment data. Default: platform cache dir. |

## Command-line options

| Option | Description |
|--------|-------------|
| `--transport` | `stdio`, `sse`, or `http`. Default: `stdio`. |
| `--port` | Port for HTTP/SSE (default: 3000). |
| `--skip-github` | Skip GitHub API calls for faster startup; use cached data only. |
| `--landscape-file` | Path to landscape YAML (default: fetch or use cache). |
| `--rate-limit` | Max concurrent requests for HTTP transport (default: 50). |

## Cache

Landscape and enrichment data are cached locally. TTL is configurable (default 24h). Use the CLI to sync:

```bash
cargo run -p mcp-atlas-cli -- sync
```

## Optional backends

- **Graph:** `--graph-backend mem` (default) or `surreal` (requires `graph-surrealdb` feature).
- **Vector / embeddings:** Set embedding API base and Qdrant URL for hybrid (BM25 + vector) search; see project README and MCP_BLUEPRINT.md.
