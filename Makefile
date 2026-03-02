# MCPAtlas — Makefile
# Run `make help` or see Makefile.md for usage.

.PHONY: help build build-release test fmt fmt-check clippy bench clean
.PHONY: sync validate run-stdio run-sse run-http
.PHONY: site-install site-build site-dev site-preview site-lint
.PHONY: verify-release check ci
.PHONY: docker-build helm-lint helm-template

# Default target
help:
	@echo "MCPAtlas — common targets (see Makefile.md for full list)"
	@echo ""
	@echo "  build          Build all crates (debug)"
	@echo "  build-release  Build release binaries (core + cli)"
	@echo "  test           Run all workspace tests"
	@echo "  fmt            Format code (cargo fmt)"
	@echo "  fmt-check      Check format only (CI)"
	@echo "  clippy         Lint with clippy"
	@echo "  check          fmt-check + clippy (quick quality gate)"
	@echo "  ci             check + test (full CI locally)"
	@echo ""
	@echo "  run-stdio      Run MCP server over STDIO (local, skip-github)"
	@echo "  run-sse        Run MCP server over SSE (port 3000)"
	@echo "  run-http       Alias for run-sse"
	@echo "  sync           Sync landscape data (mcp-atlas-cli sync)"
	@echo "  validate       Validate data/landscape.yml"
	@echo ""
	@echo "  site-install   npm ci in site/"
	@echo "  site-build     Build site (npm run build)"
	@echo "  site-dev      Start site dev server"
	@echo "  site-preview   Preview production site build"
	@echo "  site-lint      ESLint site/"
	@echo "  verify-release Run scripts/verify-release.sh (site install + build)"
	@echo ""
	@echo "  bench         Run search benchmarks"
	@echo "  clean         cargo clean"
	@echo "  docker-build   Build Docker image (deploy/docker/Dockerfile)"
	@echo "  helm-lint     Lint Helm chart"
	@echo "  helm-template  helm template mcp-atlas deploy/helm/mcp-atlas"

# --- Rust ---
build:
	cargo build

build-release:
	cargo build --release -p mcp-atlas-core -p mcp-atlas-cli

test:
	cargo test --workspace

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets

check: fmt-check clippy

ci: check test

bench:
	cargo bench -p mcp-atlas-search

clean:
	cargo clean

# --- CLI & run ---
LANDSCAPE ?= data/landscape.yml
PORT ?= 3000

sync:
	cargo run -p mcp-atlas-cli -- sync

validate:
	cargo run -p mcp-atlas-cli -- validate $(LANDSCAPE)

run-stdio:
	cargo run -p mcp-atlas-core -- --transport stdio --skip-github --landscape-file $(LANDSCAPE)

run-sse:
	cargo run -p mcp-atlas-core -- --transport sse --port $(PORT)

run-http: run-sse

# --- Site ---
site-install:
	cd site && npm ci

site-build:
	cd site && npm run build

site-dev:
	cd site && npm run dev

site-preview:
	cd site && npm run preview

site-lint:
	cd site && npm run lint

verify-release:
	./scripts/verify-release.sh

# --- Docker & Helm ---
docker-build:
	docker build -t mcp-atlas:latest -f deploy/docker/Dockerfile .

helm-lint:
	helm lint deploy/helm/mcp-atlas

helm-template:
	helm template mcp-atlas deploy/helm/mcp-atlas
