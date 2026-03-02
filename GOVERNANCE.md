# Governance

This document describes how the CNCF MCP Server project is governed.

## Roles

- **Maintainers** — Technical direction, release management, PR review and merge, security response. Listed in MAINTAINERS.md when applicable.
- **Reviewers** — Code review and triage; earned through sustained contributions.
- **Contributors** — Anyone with a merged PR or substantive issue/design input.

## Decision Making

- **Routine changes** — [Lazy consensus](https://www.apache.org/foundation/glossary.html#LazyConsensus): proceed unless someone objects with a good reason.
- **Architectural or governance changes** — Require an RFC (see below) and 2/3 maintainer approval where applicable.
- **Public roadmap** — Maintained in the repo (e.g. ROADMAP.md or GitHub Projects); community input is welcome.

## RFC Process

For significant changes (e.g. new storage backends, plugin ABI, transport behavior, dependency or runtime requirements):

1. **Propose** — Open an RFC in `docs/rfcs/` (or the path specified in the repo). Include: context, problem, proposed solution, alternatives, migration path.
2. **Discuss** — Use the PR or GitHub Discussions; maintainers and community can comment.
3. **Decision** — Maintainers decide to accept, reject, or request changes. 2/3 maintainer vote may be required for governance or API-stability decisions.

## Releases

- Releases are cut from the main branch (or a release branch) and tagged (e.g. `v0.1.0`).
- Release workflow builds multi-arch binaries and container images; SBOM and signing are introduced as we mature (see MCP_BLUEPRINT.md).

## AI-Assisted Contributions

- AI-generated or AI-assisted contributions are welcome but must be disclosed (e.g. `AI-Assisted: true` in the PR description).
- The same quality bar applies: tests, clippy, and human review. AI-only contributors do not become reviewers.

## CNCF Alignment

This project aims to align with CNCF values and, when ready, to apply for CNCF Sandbox. Governance and community practices will evolve to meet CNCF expectations.
