//! SurrealDB implementation of GraphBackend.

use std::collections::{HashMap, HashSet};

use anyhow::Context;
use async_trait::async_trait;
use mcp_atlas_data::storage::{GraphBackend, GraphStats, ProjectEdge, Relation};
use surrealdb::Surreal;
use surrealdb::engine::local::Mem;
use tracing::info;

/// SurrealDB-backed graph store (embedded in-memory).
pub struct SurrealGraphBackend {
    db: Surreal<surrealdb::engine::local::Db>,
}

impl SurrealGraphBackend {
    /// Create an in-memory SurrealDB instance and upsert the given edges.
    pub async fn new(edges: &[ProjectEdge]) -> anyhow::Result<Self> {
        let db = Surreal::new::<Mem>(())
            .await
            .context("SurrealDB Mem init")?;
        db.use_ns("cncf").use_db("graph").await?;

        // Single table: edge (from, to, relation, confidence)
        db.query(
            "DEFINE TABLE edge SCHEMAFULL; \
             DEFINE FIELD from ON edge TYPE string; \
             DEFINE FIELD to ON edge TYPE string; \
             DEFINE FIELD relation ON edge TYPE string; \
             DEFINE FIELD confidence ON edge TYPE float; \
             DEFINE INDEX idx_from ON edge COLUMNS from;",
        )
        .await
        .context("SurrealDB schema")?;

        let backend = Self { db };
        backend.upsert_edges(edges).await?;
        Ok(backend)
    }

    fn relation_to_str(r: &Relation) -> &'static str {
        match r {
            Relation::AlternativeTo => "AlternativeTo",
            Relation::IntegratesWith => "IntegratesWith",
            Relation::ComponentOf => "ComponentOf",
            Relation::Extends => "Extends",
            Relation::Supersedes => "Supersedes",
        }
    }

    fn str_to_relation(s: &str) -> Option<Relation> {
        match s {
            "AlternativeTo" => Some(Relation::AlternativeTo),
            "IntegratesWith" => Some(Relation::IntegratesWith),
            "ComponentOf" => Some(Relation::ComponentOf),
            "Extends" => Some(Relation::Extends),
            "Supersedes" => Some(Relation::Supersedes),
            _ => None,
        }
    }
}

#[async_trait]
impl GraphBackend for SurrealGraphBackend {
    async fn get_edges(&self, project: &str) -> anyhow::Result<Vec<ProjectEdge>> {
        let from_key = project.to_string();
        let mut res = self
            .db
            .query("SELECT * FROM edge WHERE from = $from")
            .bind(("from", from_key))
            .await
            .context("SurrealDB get_edges query")?;

        let rows: Vec<SurrealEdgeRow> = res.take(0)?;
        let edges: Vec<ProjectEdge> = rows
            .into_iter()
            .filter_map(|r| {
                Some(ProjectEdge {
                    from: r.from,
                    to: r.to,
                    relation: SurrealGraphBackend::str_to_relation(&r.relation)?,
                    confidence: r.confidence,
                })
            })
            .collect();
        Ok(edges)
    }

    async fn find_path(
        &self,
        from: &str,
        to: &str,
        max_depth: u8,
    ) -> anyhow::Result<Option<Vec<ProjectEdge>>> {
        let from_lower = from.to_lowercase();
        let to_lower = to.to_lowercase();
        let max_d = max_depth.min(10) as u32;

        let mut visited: HashMap<String, (String, ProjectEdge)> = HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((from.to_string(), 0u32));

        while let Some((current, depth)) = queue.pop_front() {
            if depth > max_d {
                continue;
            }
            let edges = self.get_edges(&current).await?;
            for edge in edges {
                let to_key = edge.to.to_lowercase();
                if visited.contains_key(&edge.to) || to_key == from_lower {
                    continue;
                }
                visited.insert(edge.to.clone(), (current.clone(), edge.clone()));

                if to_key == to_lower {
                    let mut path = Vec::new();
                    let mut cur = edge.to.clone();
                    while let Some((prev, e)) = visited.get(&cur) {
                        path.push(e.clone());
                        if prev.to_lowercase() == from_lower {
                            break;
                        }
                        cur = prev.clone();
                    }
                    path.reverse();
                    return Ok(Some(path));
                }
                queue.push_back((edge.to.clone(), depth + 1));
            }
        }
        Ok(None)
    }

    async fn stats(&self) -> anyhow::Result<GraphStats> {
        let mut res = self
            .db
            .query("SELECT * FROM edge")
            .await
            .context("SurrealDB stats query")?;
        let rows: Vec<SurrealEdgeRow> = res.take(0)?;
        let edges: Vec<ProjectEdge> = rows
            .into_iter()
            .filter_map(|r| {
                Some(ProjectEdge {
                    from: r.from,
                    to: r.to,
                    relation: SurrealGraphBackend::str_to_relation(&r.relation)?,
                    confidence: r.confidence,
                })
            })
            .collect();

        let total_edges = edges.len();
        let mut node_set = HashSet::new();
        let mut relation_counts: HashMap<String, usize> = HashMap::new();
        for e in &edges {
            node_set.insert(e.from.clone());
            node_set.insert(e.to.clone());
            let key = format!("{:?}", e.relation);
            *relation_counts.entry(key).or_default() += 1;
        }

        Ok(GraphStats {
            total_nodes: node_set.len(),
            total_edges,
            relation_counts,
        })
    }

    async fn upsert_edges(&self, edges: &[ProjectEdge]) -> anyhow::Result<()> {
        self.db.query("DELETE FROM edge").await?;
        for e in edges {
            let from_val = e.from.clone();
            let to_val = e.to.clone();
            let rel = Self::relation_to_str(&e.relation);
            let conf = e.confidence;
            self.db
                .query("CREATE edge SET from = $from, to = $to, relation = $relation, confidence = $confidence")
                .bind(("from", from_val))
                .bind(("to", to_val))
                .bind(("relation", rel))
                .bind(("confidence", conf))
                .await
                .context("SurrealDB create edge")?;
        }
        info!("Upserted {} edges into SurrealDB", edges.len());
        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct SurrealEdgeRow {
    from: String,
    to: String,
    relation: String,
    confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_surreal_backend_get_edges_and_stats() {
        let edges = vec![
            ProjectEdge {
                from: "A".into(),
                to: "B".into(),
                relation: Relation::AlternativeTo,
                confidence: 0.9,
            },
            ProjectEdge {
                from: "A".into(),
                to: "C".into(),
                relation: Relation::IntegratesWith,
                confidence: 0.8,
            },
        ];
        let backend = SurrealGraphBackend::new(&edges).await.unwrap();
        let out = backend.get_edges("A").await.unwrap();
        assert_eq!(out.len(), 2);
        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.total_edges, 2);
        assert!(stats.total_nodes >= 2);
    }

    #[tokio::test]
    async fn test_surreal_backend_find_path() {
        let edges = vec![
            ProjectEdge {
                from: "X".into(),
                to: "Y".into(),
                relation: Relation::Extends,
                confidence: 1.0,
            },
            ProjectEdge {
                from: "Y".into(),
                to: "Z".into(),
                relation: Relation::ComponentOf,
                confidence: 1.0,
            },
        ];
        let backend = SurrealGraphBackend::new(&edges).await.unwrap();
        let path = backend.find_path("X", "Z", 5).await.unwrap();
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].from, "X");
        assert_eq!(path[0].to, "Y");
        assert_eq!(path[1].from, "Y");
        assert_eq!(path[1].to, "Z");
    }
}
