# Stage 1: Build
FROM rust:1.88-slim AS builder

WORKDIR /app

# Copy workspace manifests first for dependency caching
COPY Cargo.toml Cargo.lock* ./
COPY crates/mcp-atlas-core/Cargo.toml crates/mcp-atlas-core/
COPY crates/mcp-atlas-data/Cargo.toml crates/mcp-atlas-data/
COPY crates/mcp-atlas-search/Cargo.toml crates/mcp-atlas-search/
COPY crates/mcp-atlas-graph/Cargo.toml crates/mcp-atlas-graph/
COPY crates/mcp-atlas-plugins/Cargo.toml crates/mcp-atlas-plugins/
COPY crates/mcp-atlas-cli/Cargo.toml crates/mcp-atlas-cli/

# Create stub source files for dependency compilation
RUN mkdir -p crates/mcp-atlas-core/src/tools crates/mcp-atlas-core/src/resources \
    crates/mcp-atlas-core/src/prompts crates/mcp-atlas-data/src \
    crates/mcp-atlas-search/src crates/mcp-atlas-graph/src \
    crates/mcp-atlas-plugins/src crates/mcp-atlas-cli/src && \
    echo "fn main() {}" > crates/mcp-atlas-core/src/main.rs && \
    echo "fn main() {}" > crates/mcp-atlas-cli/src/main.rs && \
    touch crates/mcp-atlas-core/src/lib.rs crates/mcp-atlas-data/src/lib.rs \
    crates/mcp-atlas-search/src/lib.rs crates/mcp-atlas-graph/src/lib.rs \
    crates/mcp-atlas-plugins/src/lib.rs

# Build dependencies only (cached layer)
RUN cargo build --release 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/

# Build the final binary
RUN cargo build --release -p mcp-atlas-core

# Stage 2: Runtime (minimal image)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -r mcpatlas && useradd -r -g mcpatlas -d /app mcpatlas && \
    mkdir -p /app/cache && chown -R mcpatlas:mcpatlas /app

COPY --from=builder /app/target/release/mcp-atlas /usr/local/bin/mcp-atlas

USER mcpatlas
WORKDIR /app

ENV MCP_ATLAS_CACHE_DIR=/app/cache

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

ENTRYPOINT ["mcp-atlas"]
CMD ["--transport", "sse", "--port", "3000", "--skip-github"]
