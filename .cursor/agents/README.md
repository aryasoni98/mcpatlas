# MCPAtlas — Execution-Specialized Subagents

Project-level subagents aligned with **MCP_BLUEPRINT.md** and **DEEP_PLAN.md** for production hardening and CNCF Sandbox readiness. All agents are **project-only** (`.cursor/agents/`).

## How to use

Invoke by name when you need deep review or design in that domain, e.g.:

- *"Use the architecture-reviewer subagent to audit the new storage layer."*
- *"Run security-auditor on the plugin host code."*
- *"Have cncf-readiness-advisor check our governance docs."*

## Stack overview

| # | Agent | Focus |
|---|--------|--------|
| **Core architecture** | | |
| 1 | `architecture-reviewer` | Rust modules, coupling, async safety, extensibility → risk table + refactor recommendations |
| 2 | `storage-designer` | GraphBackend, VectorBackend, CacheBackend traits; swappability, persistence, migrations |
| 3 | `vector-search-engineer` | Qdrant, embeddings, RRF, hybrid search, cost optimization |
| 4 | `graph-database-architect` | SurrealDB schema, traversals, indexing, embedded vs remote |
| **Security & compliance** | | |
| 5 | `security-auditor` | Auth, RBAC, plugin sandbox, API exposure → STRIDE table |
| 6 | `supply-chain-guardian` | SLSA, SBOM, Cosign, cargo-deny, cargo-audit, reproducible builds |
| **Infrastructure & scaling** | | |
| 7 | `kubernetes-operator-engineer` | CRDs, reconciliation, upgrade safety, multi-tenant |
| 8 | `helm-chart-architect` | values.yaml, HPA, secrets, Redis integration |
| 9 | `distributed-systems-reviewer` | Horizontal scaling, Redis session model, shared state, consistency |
| **Plugin ecosystem** | | |
| 10 | `wasm-plugin-architect` | Extism runtime, capability-based sandbox, limits, manifest/signing |
| 11 | `plugin-sdk-designer` | SDK ergonomics, ABI stability, examples, version negotiation |
| **Performance & observability** | | |
| 12 | `performance-profiler` | Search/graph/async hotspots, Tantivy, Tokio, memory |
| 13 | `observability-engineer` | Tracing, metrics, structured logs, log level reload, no PII |
| **Governance & CNCF** | | |
| 14 | `cncf-readiness-advisor` | Governance, CONTRIBUTING, Sandbox checklist |
| 15 | `documentation-architect` | In-app docs, terminology, snippets, API reference, diagrams |
| **Automation** | | |
| 16 | `tech-debt-cleaner` | Dead code, no-op fields, incomplete handlers |
| 17 | `test-coverage-maximizer` | Proptest, fuzz, graph/tool/plugin edge cases, integration tests |

## Recommended starter set (lean)

1. `architecture-reviewer`
2. `security-auditor`
3. `vector-search-engineer`
4. `wasm-plugin-architect`
5. `kubernetes-operator-engineer`
6. `performance-profiler`
7. `cncf-readiness-advisor`

## Strategic set (DevOps / infra / scaling focus)

1. `architecture-reviewer`
2. `security-auditor`
3. `distributed-systems-reviewer`
4. `wasm-plugin-architect`
5. `cncf-readiness-advisor`
