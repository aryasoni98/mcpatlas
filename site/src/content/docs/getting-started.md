# Getting Started

## Prerequisites

- **Rust** 1.85+ (or use a pre-built binary / Docker)
- Optional: **GitHub token** for richer project data (stars, language, etc.)

## Install

### From source

```bash
git clone https://github.com/cncf-mcp/server.git
cd server
cargo build --release
```

Binaries:

- `target/release/cncf-mcp`  MCP server
- `target/release/cncf-mcp-cli`  CLI for sync and validation

### Docker

```bash
docker run -p 3000:3000 ghcr.io/cncf-mcp/server:latest
```

### Homebrew (when available)

```bash
brew install cncf-mcp/tap/cncf-mcp
```

## Run the server

### STDIO (local clients, e.g. Claude Desktop)

```bash
./target/release/cncf-mcp --transport stdio --skip-github
```

Use `--skip-github` for a fast start without GitHub API calls. Add `GITHUB_TOKEN` and omit `--skip-github` for full enrichment.

### HTTP (remote clients, e.g. Cursor, VS Code)

```bash
./target/release/cncf-mcp --transport sse --port 3000
```

Then point your MCP client at `http://localhost:3000/sse` (or the URL of your deployment).

## Configure your MCP client

### Claude Desktop / Claude Code

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

### Cursor / VS Code (HTTP)

Point the client at your server URL, e.g. `http://localhost:3000/sse`, with optional headers (e.g. `Authorization`) if you add auth later.

## Verify

- **STDIO:** Start the server; your client should list tools such as `search_projects` and `get_project`.
- **HTTP:** Open `http://localhost:3000/health`  you should get a healthy response and project count.

## Next steps

- [Project structure](/docs/project-structure)  Repo layout and key files.
- [Configuration](/docs/configuration)  Cache, rate limits, and options.
- [Deployment](/docs/deployment)  Docker Compose and Kubernetes.
- [Tools reference](/docs/tools-reference)  All 14 tools and their parameters.
