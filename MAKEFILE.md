# Makefile reference

This document describes the targets in the project `Makefile`. Run `make help` for a short summary.

## Prerequisites

- **Rust** 1.85+ (see `Cargo.toml` workspace `rust-version`)
- **Node.js** 20+ for site targets (optional)
- **Docker** for `docker-build` (optional)
- **Helm** 3.x for `helm-lint` / `helm-template` (optional)

---

## Rust

| Target | Description |
|--------|-------------|
| `make build` | Build all workspace crates (debug). |
| `make build-release` | Build release binaries for `mcp-atlas-core` and `mcp-atlas-cli`. |
| `make test` | Run all workspace tests (`cargo test --workspace`). |
| `make fmt` | Format code with `cargo fmt --all`. |
| `make fmt-check` | Check formatting only (no write); used in CI. |
| `make clippy` | Lint with Clippy (`cargo clippy --workspace --all-targets`). |
| `make check` | Run `fmt-check` then `clippy` (quick quality gate). |
| `make ci` | Run `check` then `test` (full CI locally). |
| `make bench` | Run search benchmarks (`cargo bench -p mcp-atlas-search`). |
| `make clean` | Run `cargo clean`. |

---

## Running the server and CLI

| Target | Description |
|--------|-------------|
| `make run-stdio` | Run MCP server over STDIO with local landscape file; skips GitHub. Uses `LANDSCAPE=data/landscape.yml` by default. |
| `make run-sse` | Run MCP server over SSE/HTTP on port 3000. Override with `make run-sse PORT=8080`. |
| `make run-http` | Alias for `run-sse`. |
| `make sync` | Sync landscape data (`mcp-atlas-cli sync`). |
| `make validate` | Validate landscape file. Default file: `data/landscape.yml`. Override with `make validate LANDSCAPE=path/to/landscape.yml`. |

**Examples:**

```bash
make run-stdio
make run-sse PORT=8080
make validate LANDSCAPE=./data/landscape.yml
```

---

## Site (landing and docs)

| Target | Description |
|--------|-------------|
| `make site-install` | Install site dependencies (`cd site && npm ci`). |
| `make site-build` | Build the site (`cd site && npm run build`). Output: `site/dist/`. |
| `make site-dev` | Start the Vite dev server for the site. |
| `make site-preview` | Preview the production build locally. |
| `make site-lint` | Run ESLint in `site/`. |
| `make verify-release` | Run `scripts/verify-release.sh` (site install + build; used for release verification). |

---

## Docker and Helm

| Target | Description |
|--------|-------------|
| `make docker-build` | Build the Docker image from `deploy/docker/Dockerfile`; tag `mcp-atlas:latest`. |
| `make helm-lint` | Lint the Helm chart at `deploy/helm/mcp-atlas`. |
| `make helm-template` | Run `helm template mcp-atlas deploy/helm/mcp-atlas`. |

---

## Suggested workflows

- **Before commit:** `make ci` (or at least `make check` and `make test`).
- **Local MCP (STDIO):** `make run-stdio` (ensure `data/landscape.yml` exists or run `make sync` first).
- **Local HTTP server:** `make run-sse` then connect to `http://localhost:3000`.
- **Release verification:** `make verify-release` then optionally serve `site/dist` and run Lighthouse.

---

## Overridable variables

| Variable | Default | Used by |
|----------|---------|--------|
| `LANDSCAPE` | `data/landscape.yml` | `run-stdio`, `validate` |
| `PORT` | `3000` | `run-sse` |

Example: `make run-sse PORT=8080` or `make validate LANDSCAPE=./custom.yml`.
