//! SurrealDB-backed graph backend for CNCF MCP (Blueprint §2b).
//!
//! Uses embedded in-memory SurrealDB (kv-mem) to store project relationship edges.
//! Implements [GraphBackend] for use when `--graph-backend surreal` is set.

mod backend;

pub use backend::SurrealGraphBackend;
