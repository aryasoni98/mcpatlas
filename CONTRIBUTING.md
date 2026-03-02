# Contributing to CNCF MCP Server

Thank you for your interest in contributing. This document explains how to set up the project, run tests, and submit changes.

## Prerequisites

- **Rust** 1.85+ (see `rust-toolchain` or `Cargo.toml` workspace `rust-version`)
- **Git**
- Optional: **Docker** for containerized runs

## Development Setup

```bash
git clone https://github.com/mcp-atlas/mcp-atlas.git
cd server
cargo build
cargo test --workspace
```

## Architecture Overview

- **`crates/mcp-atlas-core`** — MCP server: transport (STDIO/SSE/Streamable HTTP), tool handlers, resources, prompts
- **`crates/mcp-atlas-data`** — Data models, landscape YAML parsing, GitHub enrichment, cache, pipeline
- **`crates/mcp-atlas-search`** — Tantivy full-text search index and query types
- **`crates/mcp-atlas-graph`** — In-memory knowledge graph (alternatives, relationships, path finding)
- **`crates/mcp-atlas-plugins`** — Plugin host interface (WASM/Extism planned)
- **`crates/mcp-atlas-cli`** — CLI for sync, validate, and data operations
- **`site/`** — Landing site (Vite + React + Tailwind); see [site/README.md](site/README.md) for build and deploy.

Tool handlers live in `crates/mcp-atlas-core/src/tools/`. MCP protocol routing is in `tools/mod.rs`.

## Running Locally

```bash
# STDIO (e.g. for Claude Desktop) — skip GitHub for fast startup
cargo run -p mcp-atlas -- --transport stdio --skip-github --landscape-file data/landscape.yml

# HTTP SSE server
cargo run -p mcp-atlas -- --transport sse --port 3000
```

## Quality Gates (before submitting a PR)

1. **Format:** `cargo fmt --all`
2. **Lint:** `cargo clippy --workspace --all-targets`
3. **Tests:** `cargo test --workspace`
4. **No `unwrap()` in non-test code** — use `?` or `.context()` from anyhow
5. **Logging:** use `tracing` (not `log`)

## Pull Request Checklist

- [ ] Code follows project conventions (see `CLAUDE.md` and `.cursor/rules/`)
- [ ] New behavior has tests where appropriate
- [ ] CI passes (format, clippy, tests)
- [ ] If AI-assisted: add `AI-Assisted: true` in the PR description
- [ ] No secrets or `.env` committed

## Conventional Commits

Use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages:

- `feat:` new feature
- `fix:` bug fix
- `docs:` documentation only
- `refactor:` code change that neither fixes a bug nor adds a feature
- `test:` adding or updating tests
- `chore:` tooling, deps, CI

## Architectural or Process Changes

For significant changes (new storage backends, plugin ABI, transport behavior), please open an RFC in `docs/rfcs/` and follow the process in [GOVERNANCE.md](GOVERNANCE.md).

## Questions

- **GitHub Discussions** — for questions and ideas
- **Issues** — for bugs and feature requests

Thank you for contributing.
