# RFCs

Architectural and process changes are proposed via **RFCs** (Request for Comments). See [GOVERNANCE.md](../../GOVERNANCE.md) for the decision process.

## Submitting an RFC

1. Copy [0000-template.md](0000-template.md) to `NNNN-short-title.md` (e.g. `0001-redis-cache-backend.md`).
2. Fill in the sections: Summary, Context, Problem, Proposed solution, Alternatives, Migration path.
3. Open a PR; discuss in the PR or GitHub Discussions.
4. Maintainers decide to accept, reject, or request changes. 2/3 maintainer vote may apply for governance or API-stability decisions.

## When an RFC is needed

- New storage backends (e.g. SurrealDB, Redis), plugin ABI, or transport behavior.
- Breaking changes to MCP tool/resource contracts or wire format.
- New dependencies or runtime requirements that affect deployment or security.

Routine fixes and features that don’t change architecture or contracts typically don’t require an RFC.
