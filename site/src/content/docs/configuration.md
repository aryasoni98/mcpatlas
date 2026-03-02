# Configuration

## Environment variables

| Variable | Description |
|----------|-------------|
| `GITHUB_TOKEN` | Optional. Improves rate limits for GitHub enrichment (stars, language, etc.). |
| `CNCF_MCP_CACHE_DIR` | Cache directory for landscape and enrichment data. Default: platform cache dir. |

## Command-line options

- `--transport`  `stdio`, `sse`, or `http`. Default: `stdio`.
- `--port`  Port for HTTP/SSE (default: 3000).
- `--skip-github`  Skip GitHub API calls for faster startup; use cached data only.
- `--landscape-file`  Path to landscape YAML (default: fetch or use cache).
- `--rate-limit`  Max concurrent requests for HTTP transport (default: 50).

## Cache

Landscape and enrichment data are cached locally. TTL is configurable (default 24h). Use the CLI to sync: `cargo run -p cncf-mcp-cli -- sync`.
