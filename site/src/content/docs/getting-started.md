# Getting Started

## Prerequisites

- **Rust** 1.85+ (or use a pre-built binary / Docker)
- Optional: **GitHub token** for richer project data (stars, language, etc.)

## Install

### From source

```bash
git clone https://github.com/mcp-atlas/server.git
cd server
cargo build --release
```

Binaries:

- `target/release/mcp-atlas`  MCP server
- `target/release/mcp-atlas-cli`  CLI for sync and validation

### Docker

```bash
docker run -p 3000:3000 ghcr.io/mcp-atlas/server:latest
```

### Homebrew

```bash
brew tap aryasoni98/mcpatlas
brew install mcp-atlas
```

## Run the server

### STDIO (local clients, e.g. Claude Desktop)

```bash
./target/release/mcp-atlas --transport stdio --skip-github
```

Use `--skip-github` for a fast start without GitHub API calls. Add `GITHUB_TOKEN` and omit `--skip-github` for full enrichment.

### HTTP (remote clients, e.g. Cursor, VS Code)

```bash
./target/release/mcp-atlas --transport sse --port 3000
```

Then point your MCP client at `http://localhost:3000/sse` (or the URL of your deployment).

## Configure your MCP client

### Claude Desktop / Claude Code

Add to your MCP config (e.g. `claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "cncf-landscape": {
      "command": "/path/to/mcp-atlas",
      "args": ["--transport", "stdio", "--skip-github"],
      "env": {
        "GITHUB_TOKEN": "<optional>",
        "MCP_ATLAS_CACHE_DIR": "~/.cache/mcp-atlas"
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
- [Tools reference](/docs/tools-reference)  All 15 tools and their parameters.
