# MCPAtlas — AI Coding Guidelines

## Project Overview
MCPAtlas is an MCP server for the CNCF Landscape. Exposes 2,400+ cloud-native projects as MCP tools/resources/prompts for AI assistants.

## Architecture
- **Rust workspace** with 6 crates: `core`, `data`, `search`, `graph`, `plugins`, `cli`; optional `mcp-atlas-graph-surrealdb` when `--features graph-surrealdb`
- **Project site** (`site/`): Vite + React + Tailwind + Framer Motion landing with in-app docs at `/docs` (markdown in `site/src/content/docs/`); built with `npm ci && npm run build` in `site/`; deployed to GitHub Pages (`.github/workflows/pages.yml`) or Vercel (root `site`, `site/vercel.json`). For landing and release context see `docs/RELEASE_EXECUTION_REPORT.md` and `docs/FRONTEND_ARCHITECT_TASK.md`.
- MCP protocol over STDIO (local), SSE/HTTP (remote), and Streamable HTTP (MCP 2025-03-26)
- Tantivy for full-text search, serde for serialization
- Knowledge graph engine for project relationships (auto-inferred + 25 curated edges)
- Data sourced from `cncf/landscape` GitHub repo (landscape.yml)
- Maturity derived from `project:` field in YAML (graduated/incubating/sandbox/archived)
- Local JSON cache with configurable TTL (default 24h)

## Conventions
- Use `anyhow::Result` for application errors, `thiserror` for library error enums
- Use `tracing` (not `log`) for all logging
- All public types must derive `Debug, Clone, Serialize, Deserialize`
- Keep tool handlers in `crates/mcp-atlas-core/src/tools/`
- MCP responses follow JSON-RPC 2.0 format
- Tests go next to the code they test (`#[cfg(test)] mod tests`)

## Do NOT
- Add `unsafe` code
- Use `unwrap()` in non-test code — use `?` or `.context()`
- Add dependencies without justification
- Modify the MCP protocol wire format without updating the spec version
- Commit `.env` files or API tokens

## MCP Tools (14 implemented)
- `search_projects` — full-text search across 2,400+ projects
- `get_project` — detailed project lookup by name
- `compare_projects` — side-by-side comparison table
- `list_categories` — all landscape categories/subcategories
- `get_stats` — landscape-wide statistics
- `find_alternatives` — projects in the same subcategory
- `get_health_score` — health score from GitHub metrics
- `suggest_stack` — recommend a cloud-native stack for a use case
- `analyze_trends` — adoption metrics and trends for a category
- `get_relationships` — knowledge graph edges for a project
- `find_path` — shortest path between two projects in the graph
- `get_graph_stats` — knowledge graph statistics
- `get_good_first_issues` — list projects good for contributors (filter by language/category)
- `get_migration_path` — migration guide from one project to another

## MCP Prompts (4 implemented)
- `evaluate_tool` — structured analysis of a CNCF project for a use case
- `plan_migration` — migration plan between two CNCF tools
- `review_stack` — gap/redundancy analysis of a cloud-native stack
- `onboard_contributor` — onboarding guide for new contributors

## MCP Resources
- `cncf://landscape/overview` — landscape statistics
- `cncf://categories/all` — all categories and subcategories
- `cncf://projects/{name}` — project detail by name (template)
- `cncf://categories/{category}` — category listing (template)

## Auto-Completion
- Project names, categories, maturity levels, relation types via `completion/complete`

## HTTP Transport Features
- CORS (any origin, GET/POST/DELETE/OPTIONS)
- Request tracing via tower-http TraceLayer
- Concurrency limiting (semaphore-based, configurable `--rate-limit`)
- Graceful shutdown (SIGTERM/SIGINT)
- Streamable HTTP at `/mcp/stream` with session management (`Mcp-Session-Id`)
- Prometheus-compatible `/metrics` endpoint
- JSON-RPC batch request support (spec §6)
- MCP `logging/setLevel` for client-controlled log levels
- Search pagination with offset/limit and `_meta` response
- Request cancellation (`notifications/cancelled`) with in-flight tracking
- Content-Type validation (415 for non-JSON POSTs)
- `roots/list` handler and `notifications/roots/list_changed`
- Structured error `data` field in JSON-RPC tool call errors

## Commands
- `cargo build` — build all crates
- `cargo test --workspace` — run all tests (89 total)
- `cargo run -p mcp-atlas-core -- --transport stdio --skip-github --landscape-file data/landscape.yml` — run locally
- `cargo run -p mcp-atlas-core -- --transport sse --port 3000` — run as HTTP server
- `cargo run -p mcp-atlas-cli -- sync` — download fresh landscape data
- `cargo run -p mcp-atlas-cli -- validate data/landscape.yml` — validate landscape file
- `cargo clippy --workspace` — lint
- `cargo fmt --all` — format
- `cargo bench -p mcp-atlas-search` — run search benchmarks
- **Site:** `cd site && npm ci && npm run build` — build landing; `./scripts/verify-release.sh` from repo root — verify site + docs build and combine

## STDIO Transport
- Uses Content-Length framing (same as LSP): `Content-Length: N\r\n\r\n{json}`
- Logs go to stderr, JSON-RPC responses go to stdout

## Key Data Model
- `Project.maturity` uses `#[serde(alias = "project")]` to parse the YAML `project:` field
- `flatten_projects()` also derives maturity from `extra.graduated/incubating/accepted` dates as fallback
- `Maturity` enum: Sandbox, Incubating, Graduated, Archived, Unknown

## Productivity: Rules and Skills
- **Cursor rules** (`.cursor/rules/`): Apply when editing matching paths. `cncf-rust-standards` (Rust), `cncf-mcp-and-api` (MCP/core), `cncf-security-compliance` (always), `cncf-release-and-deploy` (CI/deploy), `cncf-productivity-skills` (when to use wrap-up, smart-commit, etc.).
- **Project skills** (`.cursor/skills/`):
  - **Workflow**: **cncf-pre-commit** (before commit: fmt, clippy, test, conventional commit). **cncf-release-checklist** (cutting releases). **cncf-rfc-and-governance** (RFCs, CONTRIBUTING/GOVERNANCE/SECURITY). **cncf-storage-and-architecture** (storage/graph/vector backends).
  - **Roles & expertise**: **product-manager** (roadmap, impact, prioritization, scope, success metrics, CNCF positioning). **solution-architect** (system design, deployment tiers, tradeoffs, scalability, security). **rust-expertise** (async/tokio, errors, serde, performance, testing). **mcp-expertise** (tools/resources/prompts, JSON-RPC, transports, version negotiation, completion, cancellation).
  - **Discipline skills**: **security** (threat model, RBAC, audit, supply chain, input validation, plugin sandbox). **devops** (CI/CD, Docker, Helm, K8s, observability). **technical-writer** (site in-app docs, CONTRIBUTING/SECURITY/GOVERNANCE, API and plugin docs). **testing-qa** (unit/integration/e2e, fixtures, benchmarks). **plugin-development** (WASM, Extism, manifest, host functions, sandbox).
- For product/enterprise flow: run quality gates before commit, wrap up with summary and learnings at session end, use session-handoff when pausing, and align with BluePrint.md and DEEP_PLAN.md for phases and architecture.
