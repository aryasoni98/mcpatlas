# mcp-atlas-graph-surrealdb

Optional SurrealDB backend for the MCPAtlas knowledge graph (Blueprint §2b).

Built only when `mcp-atlas-core` is compiled with `--features graph-surrealdb`. Uses embedded in-memory SurrealDB (kv-mem) to store project relationship edges and implements the shared `GraphBackend` trait.

## Usage

From the repo root:

```bash
cargo build -p mcp-atlas --features graph-surrealdb
cargo run -p mcp-atlas -- --graph-backend surreal --transport sse --port 3000 --skip-github
```

With the default build (no feature), `--graph-backend mem` is used and this crate is not compiled.

## Schema

- **Table `edge`**: `from` (string), `to` (string), `relation` (string), `confidence` (float).
- Namespace/database: `cncf` / `graph`.

Future: optional remote SurrealDB via `--surreal-url` (client-server mode).
