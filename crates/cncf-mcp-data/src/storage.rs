//! Storage backend traits for graph, vector, and cache (Blueprint §2a).
//!
//! Allows swapping in-memory implementations for SurrealDB (graph), Qdrant (vector),
//! and Redis (cache) in future phases.

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Relationship types between CNCF projects.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Relation {
    /// Project A is an alternative to Project B (same subcategory).
    AlternativeTo,
    /// Project A depends on / integrates with Project B.
    IntegratesWith,
    /// Project A is a component of Project B.
    ComponentOf,
    /// Project A extends / plugins into Project B.
    Extends,
    /// Project A supersedes Project B.
    Supersedes,
}

/// An edge in the project knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEdge {
    pub from: String,
    pub to: String,
    pub relation: Relation,
    pub confidence: f64,
}

/// Graph-level statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub relation_counts: HashMap<String, usize>,
}

/// Backend for the project relationship graph.
#[async_trait]
pub trait GraphBackend: Send + Sync {
    /// Return all edges originating from the given project.
    async fn get_edges(&self, project: &str) -> anyhow::Result<Vec<ProjectEdge>>;

    /// Find a shortest path between two projects (BFS, up to max_depth hops).
    async fn find_path(
        &self,
        from: &str,
        to: &str,
        max_depth: u8,
    ) -> anyhow::Result<Option<Vec<ProjectEdge>>>;

    /// Return graph statistics.
    async fn stats(&self) -> anyhow::Result<GraphStats>;

    /// Upsert edges (for pipeline sync; optional, no-op for read-only backends).
    async fn upsert_edges(&self, _edges: &[ProjectEdge]) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Backend for vector search (e.g. Qdrant); optional in Phase 1.
#[async_trait]
pub trait VectorBackend: Send + Sync {
    /// Search by embedding vector; returns (project_id, score) pairs.
    async fn search(
        &self,
        _embedding: &[f32],
        _limit: usize,
    ) -> anyhow::Result<Vec<(String, f64)>> {
        Ok(Vec::new())
    }

    /// Upsert a single vector with metadata.
    async fn upsert(
        &self,
        _id: &str,
        _embedding: &[f32],
        _metadata: serde_json::Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Backend for key-value cache (e.g. Redis, file); optional in Phase 1.
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Get raw bytes by key.
    async fn get(&self, _key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(None)
    }

    /// Set key to value with TTL.
    async fn set(&self, _key: &str, _value: &[u8], _ttl: Duration) -> anyhow::Result<()> {
        Ok(())
    }

    /// Delete a key.
    async fn delete(&self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
