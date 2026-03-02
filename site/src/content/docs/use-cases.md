# Use cases & setup

Set up the CNCF MCP server for your environment.

## Cursor

1. Start the server (STDIO or HTTP). For HTTP: `--transport sse --port 3000`.
2. In Cursor settings, add the MCP server: point to `http://localhost:3000/sse` or configure the STDIO command and args (`cncf-mcp`, `--transport stdio --skip-github`).
3. Use the tools from the Cursor AI panel (search, get project, compare, etc.).

## Claude Desktop / Claude Code

Add to your MCP config (e.g. `claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "cncf-landscape": {
      "command": "/path/to/cncf-mcp",
      "args": ["--transport", "stdio", "--skip-github"],
      "env": {
        "GITHUB_TOKEN": "<optional>",
        "CNCF_MCP_CACHE_DIR": "~/.cache/cncf-mcp"
      }
    }
  }
}
```

## Docker

```bash
docker run -p 3000:3000 ghcr.io/cncf-mcp/server:latest
```

Point your client at `http://localhost:3000/sse`. Use environment variables or a mounted config for cache and tokens.

## Kubernetes

Use the Helm chart in `deploy/helm/` or apply manifests from `deploy/`. Expose the server via Service/Ingress and configure your MCP client with the service URL (e.g. `http://cncf-mcp:3000/sse`).

## Next steps

- [Configuration](/docs/configuration)  Environment variables and CLI options.
- [Deployment](/docs/deployment)  Compose, Helm, and production options.
