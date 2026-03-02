---
name: technical-writer
description: Documentation structure and style for CNCF MCP: in-app docs (site/src/content/docs/), CONTRIBUTING, SECURITY, GOVERNANCE, API and plugin docs. Use when writing or reorganizing docs.
---

# Technical Writer — CNCF MCP

Align docs with BluePrint §3 (Phase 1 docs site), §10 (CNCF Sandbox files), DEEP_PLAN §10.

## Doc structure

- **README.md**: Project overview, quick start (install, run locally), link to full docs. Keep concise.
- **site/src/content/docs/**: In-app docs (markdown) served at `/docs`. Chapters: Introduction, Getting Started, Configuration, Architecture, Deployment, Roadmap, Contributing, Tools reference.
- **In-repo governance**: CONTRIBUTING.md, CODE_OF_CONDUCT.md (CNCF CoC), SECURITY.md (disclosure, response), GOVERNANCE.md (lazy consensus, RFC, maintainers), MAINTAINERS.md, ADOPTERS.md, ROADMAP.md (from DEEP_PLAN/BluePrint).

## Style

- Audience: developers and operators integrating or extending the server. Assume familiarity with MCP and Rust basics; link to spec and glossary where helpful.
- Use present tense, second person (“you”) or passive. Short sentences; bullet lists for options and steps.
- Code blocks: label language (bash, json, rust, yaml). Show minimal, runnable examples. Keep line length readable (wrap or omit irrelevant parts).
- Keep version-specific notes in a single “Versioning” or “Releases” section; avoid scattering “as of v0.x” in body.

## Tools and API docs

- **Tools Reference**: One section per tool (or group). Name, description, input schema (params with types and defaults), example request/response, errors. Sync with server registration.
- **Resources**: List URI templates, description, MIME type, example. **Prompts**: Name, arguments, example usage.
- Prefer generated schema (e.g. from Rust types or OpenAPI) where possible; otherwise keep manual docs in sync with code.

## Plugin docs

- **Plugin Development**: How to build a plugin (WASM, Extism), manifest (plugin.toml), permissions, host functions, lifecycle. Example plugin and step-by-step.
- **Plugin SDK**: Link to PDK (Rust/Go/JS); list available host calls and limits. Security (sandbox, allowlist) in one place.
- Document how to install/load plugins (CLI, config, K8s) and how to publish to registry (if any).

## Contributing and governance

- **CONTRIBUTING.md**: Clone, build, test (cargo test, clippy), run locally (--transport stdio --skip-github). PR checklist (tests, no secrets, AI-assisted disclosure). Where to ask (GitHub Discussions, Slack).
- **SECURITY.md**: How to report vulnerabilities (private channel, no public issues); expected response timeline; no disclosure before fix.
- **GOVERNANCE.md**: Roles (maintainer, reviewer, contributor), lazy consensus, RFC process (where to submit, template), 2/3 vote for governance changes. Link to ROADMAP and code owners.

When adding a feature: add or update the corresponding doc section and ToC in the same PR.
