---
name: cncf-rfc-and-governance
description: When and how to write RFCs, update GOVERNANCE, CONTRIBUTING, SECURITY for MCPAtlas. Use when making architectural decisions, adding governance docs, or preparing for CNCF Sandbox.
---

# MCPAtlas RFC and Governance

Use when changing architecture, adding major features, or updating project governance (BluePrint §10, DEEP_PLAN §11).

## When to write an RFC

- New storage backends (e.g. SurrealDB, Redis), plugin ABI, or transport behavior.
- Breaking changes to MCP tool/resource contracts or wire format.
- New dependencies or runtime requirements that affect deployment or security.

Place RFCs in `docs/rfcs/` (or path specified in GOVERNANCE). Template: context, problem, proposed solution, alternatives, migration path.

## Governance and community docs

Keep these aligned with BluePrint and CNCF expectations:

| File | Purpose |
|------|--------|
| **CONTRIBUTING.md** | Setup, architecture overview, PR checklist, how to run tests and CI. |
| **GOVERNANCE.md** | Lazy consensus, RFC process, 2/3 maintainer vote for governance changes, public roadmap. |
| **SECURITY.md** | Vulnerability disclosure, how to report, response process. |
| **CODE_OF_CONDUCT.md** | Adopt CNCF CoC. |
| **MAINTAINERS.md** | List of maintainers. |
| **ADOPTERS.md** | List of adopters (when applicable). |

## AI-assisted contributions

- PRs that are AI-assisted should say so (e.g. `AI-Assisted: true` in description).
- Same quality bar: tests, clippy, human review. AI-only contributors do not become reviewers.

## References

- BluePrint.md §10 (CNCF Sandbox Readiness), §11 (90-day execution).
- DEEP_PLAN.md §11 (Project Governance), §13 (Community & Ecosystem).
