# Introduction

**CNCF MCP Server** is an [MCP](https://modelcontextprotocol.io/) (Model Context Protocol) server that exposes the [CNCF Landscape](https://landscape.cncf.io/) — 2,400+ cloud-native projects — as tools, resources, and prompts for AI assistants (Claude, Cursor, custom agents).

## What you get

- **Tools** — Search projects, compare, get health scores, find alternatives, suggest stacks, analyze trends, graph relationships, migration paths, and more.
- **Resources** — Read landscape overview, categories, and project details via `cncf://` URIs.
- **Prompts** — Structured prompts for evaluating projects, planning migrations, reviewing stacks, and contributor onboarding.

## Transports

- **STDIO** — For local clients (e.g. Claude Desktop). Content-Length framed JSON-RPC.
- **SSE / Streamable HTTP** — For remote clients. CORS, metrics, session management, request cancellation.

## Next steps

- [Getting Started](getting-started.md) — Install and run the server.
- [Configuration](configuration.md) — Environment variables and CLI options.
- [Tools Reference](tools-reference.md) — All MCP tools and parameters.
