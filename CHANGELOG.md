# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Landing site** (`site/`): Vite + React + TypeScript + Tailwind + Framer Motion; Hero, Problem, Solution, Features, Architecture, Use cases, Roadmap, CTA
- **2026 AI-native landing redesign**: New hero with two-column layout, MCP flow diagram (Client→Transport→Core→Tools→Storage), microterminal block with line-by-line reveal and cursor blink; shared motion system (`sectionReveal`, `staggerContainer`/`staggerItem`) with reduced-motion fallbacks; design tokens (`--accent-brand`, `--accent-glow`, `shadow-glow`); Success metrics table in Roadmap (from ROADMAP.md); scroll-linked navbar glass (stronger background after 80px)
- **In-app docs** at `/docs`: Markdown content in `site/src/content/docs/` (introduction, getting-started, configuration, architecture, deployment, roadmap, contributing, tools-reference); sidebar layout and `react-markdown` + `remark-gfm` rendering
- **Vercel deployment**: `site/vercel.json` with build command, output directory, and SPA rewrites; set Vercel root to `site` for one-click deploy
- **404.html** in site for SPA fallback on GitHub Pages
- **Release verification script** `scripts/verify-release.sh` (install, build site, serve instructions)
- **CI site job**: build and lint the Vite site on every push/PR

### Changed

- **Hero**: Replaced floating-icons hero with MCP flow diagram + terminal block; headline "The CNCF Landscape, in your AI assistant."; gradient + grid background with optional parallax
- **Sections**: Problem, Solution, Features, Architecture, Use cases, Roadmap, CTA use shared motion variants and design tokens; Features/UseCases/Roadmap use stagger-on-scroll; Architecture includes MCPFlowDiagram and tokenized pre block
- **Roadmap**: Phase cards show "Phase N of 4"; Success metrics table (3/6/12 mo) added below phases
- **Navbar**: Scroll-based glass (opacity increase after 80px); tokenized link colors
- **Docs delivery**: Documentation is served from the Vite app at `/docs` (in-app routes; markdown in `site/src/content/docs/`)
- **Pages workflow** (`.github/workflows/pages.yml`): Builds and deploys the Vite site (landing + in-app docs)
- **Navbar**: "Docs" links to in-app `/docs`; internal links use React Router `<Link>`
- **LAUNCH_READINESS_REPORT** and **RELEASE_EXECUTION_REPORT**: Updated for in-app docs and Vercel

### Fixed

### Security

---

## [0.1.0] - TBD

### Added

- MCP server for CNCF Landscape (2,400+ projects) over STDIO, SSE/HTTP, and Streamable HTTP
- Tools: `search_projects`, `get_project`, `compare_projects`, `list_categories`, `get_stats`, `find_alternatives`, `get_health_score`, `suggest_stack`, `analyze_trends`, `get_relationships`, `find_path`, `get_graph_stats`, `get_good_first_issues`, `get_migration_path`
- Prompts: `evaluate_tool`, `plan_migration`, `review_stack`, `onboard_contributor`
- Resources: `cncf://landscape/overview`, `cncf://categories/all`, `cncf://projects/{name}`, `cncf://categories/{category}`
- Full-text search (Tantivy), knowledge graph engine, local JSON cache with configurable TTL
- CLI: `sync`, `validate` for landscape data
- HTTP transport: CORS, rate limiting, `/metrics`, streamable sessions, request cancellation
- Multi-arch release binaries and container image (ghcr.io)

[Unreleased]: https://github.com/mcp-atlas/mcp-atlas/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/mcp-atlas/mcp-atlas/releases/tag/v0.1.0
