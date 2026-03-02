//! Qdrant implementation of [mcp_atlas_data::storage::VectorBackend].

use anyhow::{Context, Result};
use async_trait::async_trait;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, QueryPointsBuilder, UpsertPointsBuilder,
    VectorParamsBuilder,
};
use tracing::{debug, info};

use mcp_atlas_data::storage::VectorBackend;

const DEFAULT_COLLECTION: &str = "cncf_projects";

/// Qdrant-backed vector search. Uses project name as point id (via hash for PointId).
pub struct QdrantVectorBackend {
    client: Qdrant,
    collection: String,
    #[allow(dead_code)] // used when creating collection; kept for validation
    dimensions: u64,
}

impl QdrantVectorBackend {
    /// Build backend from URL (e.g. http://localhost:6334). Creates collection if missing.
    pub async fn new(url: &str, collection: Option<String>, dimensions: u32) -> Result<Self> {
        let client = Qdrant::from_url(url)
            .build()
            .map_err(|e| anyhow::anyhow!("Qdrant connect: {:?}", e))?;
        let collection = collection.unwrap_or_else(|| DEFAULT_COLLECTION.to_string());
        let dimensions_u64 = dimensions as u64;

        // Create collection if not exists (ignore "already exists" error)
        if let Err(e) = client
            .create_collection(
                CreateCollectionBuilder::new(&collection)
                    .vectors_config(VectorParamsBuilder::new(dimensions_u64, Distance::Cosine)),
            )
            .await
        {
            let msg = format!("{:?}", e);
            if !msg.contains("already exists") && !msg.contains("AlreadyExists") {
                anyhow::bail!("create Qdrant collection: {:?}", e);
            }
            // Collection exists, continue
        } else {
            info!(%collection, size = dimensions, "created Qdrant collection");
        }

        Ok(Self {
            client,
            collection: collection.clone(),
            dimensions: dimensions_u64,
        })
    }

    /// Point id from project name (Qdrant accepts u64; we use stable hash of name).
    fn point_id(name: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        name.hash(&mut h);
        h.finish()
    }
}

#[async_trait]
impl VectorBackend for QdrantVectorBackend {
    async fn search(&self, embedding: &[f32], limit: usize) -> anyhow::Result<Vec<(String, f64)>> {
        let results = self
            .client
            .query(
                QueryPointsBuilder::new(&self.collection)
                    .query(embedding.to_vec())
                    .limit(limit as u64)
                    .with_payload(true),
            )
            .await
            .context("Qdrant query")?;

        let points = results.result;
        let mut out = Vec::with_capacity(points.len());
        for pt in points {
            let id = pt
                .id
                .and_then(|id| id.point_id_options)
                .map(|opt| {
                    use qdrant_client::qdrant::point_id::PointIdOptions;
                    match opt {
                        PointIdOptions::Uuid(s) => s,
                        PointIdOptions::Num(x) => x.to_string(),
                    }
                })
                .unwrap_or_default();
            let score = pt.score as f64;
            // Payload may contain "name" if we stored it; otherwise id is the name for us (we use hash, so we need name in payload)
            let name = pt
                .payload
                .get("name")
                .and_then(|v| {
                    use qdrant_client::qdrant::value::Kind;
                    match &v.kind {
                        Some(Kind::StringValue(s)) => Some(s.clone()),
                        _ => None,
                    }
                })
                .unwrap_or_else(|| id.clone());
            out.push((name, score));
        }
        debug!(count = out.len(), "Qdrant search completed");
        Ok(out)
    }

    async fn upsert(
        &self,
        id: &str,
        embedding: &[f32],
        _metadata: serde_json::Value,
    ) -> anyhow::Result<()> {
        let payload = [("name", qdrant_client::qdrant::Value::from(id))];
        let point = PointStruct::new(Self::point_id(id), embedding.to_vec(), payload);
        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection, vec![point]))
            .await
            .context("Qdrant upsert")?;
        Ok(())
    }
}
