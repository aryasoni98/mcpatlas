# Security

## Reporting a Vulnerability

If you believe you have found a security vulnerability, please report it responsibly.

**Do not** open a public GitHub issue for security-sensitive findings.

### How to report

1. **Email** the maintainers (see [MAINTAINERS.md](MAINTAINERS.md) if present) or use the security contact for the repository.
2. **Include** a clear description, steps to reproduce, and impact.
3. **Allow** a reasonable time for a fix before any public disclosure (we aim for 90 days or coordination with the reporter).

### What to expect

- We will acknowledge receipt and aim to respond within a few business days.
- We will work with you to understand and address the issue.
- We will keep you updated on progress and may ask for your input on the fix.
- We will credit you in advisories if you wish (unless you prefer to stay anonymous).

### Scope

- **In scope:** vulnerabilities in this repository (server, CLI, data pipeline, dependencies used by the project).
- **Out of scope:** general hardening ideas, issues in the CNCF Landscape upstream data or in third-party services we integrate with (e.g. GitHub API), unless they directly affect how we handle or expose data.

### Security practices in this project

- No secrets in repo or logs; use environment variables or external secret stores.
- Input validation and bounds on MCP tool arguments (query length, payload size).
- Supply chain: `cargo-deny` and `cargo-audit` in CI; signed releases and SBOM as we mature (see [MCP_BLUEPRINT.md](MCP_BLUEPRINT.md)).

Thank you for helping keep CNCF MCP and its users safe.
