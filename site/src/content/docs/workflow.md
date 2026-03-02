# Workflow (CI/CD & release)

How we build, test, and release the project.

## CI

- **Format:** `cargo fmt --all`
- **Lint:** `cargo clippy --workspace --all-targets`
- **Tests:** `cargo test --workspace`
- **Site:** In the Pages workflow, `npm ci && npm run build` in `site/` to build the landing and docs.

See `.github/workflows/` for the full pipeline (e.g. `ci.yml`, `release.yml`, `pages.yml`).

## Release

- Tag a version (e.g. `v1.0.0`); the release workflow builds multi-arch binaries and the container image.
- Attach artifacts to the GitHub Release; optionally publish to registries and package managers (e.g. Homebrew).
- See [CONTRIBUTING.md](https://github.com/cncf-mcp/server/blob/main/CONTRIBUTING.md) and the release checklist for signing, SBOM, and provenance.

## Contributing

- Open a PR with a clear description and conventional commit type (`feat`, `fix`, `docs`, etc.).
- Ensure CI passes; follow [Contributing](/docs/contributing) for quality gates and code of conduct.
