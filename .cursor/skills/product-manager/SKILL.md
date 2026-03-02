---
name: product-manager
description: Align MCPAtlas work with roadmap, impact, and success metrics. Use when prioritizing features, defining scope, writing roadmap/ROADMAP.md, or discussing user value and CNCF Sandbox positioning.
---

# Product Manager — MCPAtlas

Apply product thinking: user impact, roadmap alignment, scope, and success metrics (BluePrint §11, DEEP_PLAN §1, §12–13).

## Problem and impact (keep front of mind)

- **Problem**: 2,400+ CNCF projects; developers and AI lack structured, real-time access. Discovery and cross-project knowledge are fragmented.
- **Impact**: 10x faster discovery, AI-augmented contributions with full CNCF context, ecosystem intelligence (trends, health), contributor onboarding paths, maturity/health for CNCF governance.
- **Value prop**: Living, queryable knowledge graph over the landscape via MCP (Anthropic, OpenAI, Google, Microsoft — 97M+ SDK downloads).

## Prioritization

- **Phase 1 (Foundation)**: Core MCP server, search, get_project, compare, resources, STDIO/SSE, Docker, Homebrew, docs. Goal: installable, usable.
- **Phase 2 (Intelligence)**: Semantic search, health scoring, suggest_stack, find_alternatives, analyze_trends, LLM summaries, Streamable HTTP. Goal: CNCF Sandbox application.
- **Phase 3 (Extensibility)**: WASM plugins, get_good_first_issues, get_migration_path, K8s operator, contributor automation. Goal: ecosystem and community.
- **Phase 4 (Scale & maturity)**: Horizontal scaling, Helm, RBAC, audit, SLSA, performance, enterprise features. Goal: production and Incubating prep.

When choosing work: close gaps in current phase before jumping; fix tech debt that blocks scaling or security; prefer items that unblock multiple streams (e.g. storage traits unblock SurrealDB and Redis).

## Scope and “out of scope”

- **In scope**: MCP server for CNCF Landscape (tools, resources, prompts), plugin system, data pipeline, deployment (local, Docker, K8s), docs and governance.
- **Out of scope (for now)**: Writing back to CNCF repos via MCP, real-time event stream, cross-landscape federation (LF AI, OpenSSF), marketplace for premium plugins — treat as later roadmap.

## Success metrics (DEEP_PLAN §13)

| Metric        | 3 mo   | 6 mo    | 12 mo   |
|---------------|--------|---------|---------|
| GitHub stars  | 500    | 2,000   | 10,000  |
| Monthly downloads | 1K | 10K     | 100K    |
| Contributors  | 5      | 20      | 50+     |
| MCP tools     | 6      | 10      | 20+     |
| Plugins       | 0      | 3       | 15+     |
| CNCF status   | —      | Sandbox | Incubating prep |

Use these when justifying effort or writing ROADMAP.md / ADOPTERS.md.

## Stakeholder and community

- **Users**: Developers and AI agents (Claude, Cursor, VS Code, custom agents). Optimize for “ask and get an answer” latency and correctness.
- **CNCF**: Sandbox → Incubating path. Governance, CoC, SECURITY.md, CONTRIBUTING.md, maintainers from multiple orgs.
- **Partners**: IDE/tool integrations (Anthropic, Cursor, VS Code), CNCF TOC. Document integration guides and adoption.

When drafting docs or features, state who benefits and how it moves a phase or metric.
