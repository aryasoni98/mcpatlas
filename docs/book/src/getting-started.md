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

- `target/release/cncf-mcp` — MCP server
- `target/release/cncf-mcp-cli` — CLI for sync and validation

### Docker

```bash
docker run -p 3000:3000 ghcr.io/cncf-mcp/server:latest
```

### Homebrew

When a tap is available:

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

Point your MCP client at `http://localhost:3000/sse` (or your deployment URL).

## Configure your MCP client

### Claude Desktop / Claude Code

Add to your MCP config (e.g. `claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "cncf": {
      "command": "/path/to/cncf-mcp",
      "args": ["--transport", "stdio", "--skip-github"]
    }
  }
}
```

### Cursor / VS Code (HTTP)

Use the SSE endpoint URL, e.g. `http://localhost:3000/sse`, in your MCP client settings.

## Next

- [Configuration](configuration.md) — Environment variables and CLI options.
- [Tools Reference](tools-reference.md) — All available MCP tools.
