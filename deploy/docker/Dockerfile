# Stage 1: Build
FROM rust:1.85-slim AS builder

WORKDIR /app

# Copy workspace manifests first for dependency caching
COPY Cargo.toml Cargo.lock* ./
COPY crates/cncf-mcp-core/Cargo.toml crates/cncf-mcp-core/Cargo.toml
COPY crates/cncf-mcp-data/Cargo.toml crates/cncf-mcp-data/Cargo.toml
COPY crates/cncf-mcp-search/Cargo.toml crates/cncf-mcp-search/Cargo.toml
COPY crates/cncf-mcp-graph/Cargo.toml crates/cncf-mcp-graph/Cargo.toml
COPY crates/cncf-mcp-plugins/Cargo.toml crates/cncf-mcp-plugins/Cargo.toml
COPY crates/cncf-mcp-cli/Cargo.toml crates/cncf-mcp-cli/Cargo.toml

# Create stub source files for dependency compilation
RUN mkdir -p crates/cncf-mcp-core/src/tools crates/cncf-mcp-core/src/resources \
    crates/cncf-mcp-core/src/prompts crates/cncf-mcp-data/src \
    crates/cncf-mcp-search/src crates/cncf-mcp-graph/src \
    crates/cncf-mcp-plugins/src crates/cncf-mcp-cli/src && \
    echo "fn main() {}" > crates/cncf-mcp-core/src/main.rs && \
    echo "fn main() {}" > crates/cncf-mcp-cli/src/main.rs && \
    touch crates/cncf-mcp-core/src/lib.rs crates/cncf-mcp-data/src/lib.rs \
    crates/cncf-mcp-search/src/lib.rs crates/cncf-mcp-graph/src/lib.rs \
    crates/cncf-mcp-plugins/src/lib.rs

# Build dependencies only (cached layer)
RUN cargo build --release 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/

# Build the final binary
RUN cargo build --release -p cncf-mcp-core

# Stage 2: Runtime (minimal image)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -r cncf && useradd -r -g cncf -d /app cncf && \
    mkdir -p /app/cache && chown -R cncf:cncf /app

COPY --from=builder /app/target/release/cncf-mcp /usr/local/bin/cncf-mcp

USER cncf
WORKDIR /app

ENV CNCF_MCP_CACHE_DIR=/app/cache

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

ENTRYPOINT ["cncf-mcp"]
CMD ["--transport", "sse", "--port", "3000", "--skip-github"]
