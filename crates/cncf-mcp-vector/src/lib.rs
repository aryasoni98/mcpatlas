//! Qdrant vector backend for CNCF MCP (Phase 2).
//! Implements [VectorBackend] for hybrid BM25 + vector search.

mod backend;

pub use backend::QdrantVectorBackend;
