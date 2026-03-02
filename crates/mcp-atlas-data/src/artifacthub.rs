//! Artifact Hub API client for enriching projects with Helm chart metadata.
//! See COMPLETION_AND_LAUNCH_PLAN.md Phase 2 §1.3.

use anyhow::{Context, Result};
use reqwest::Url;
use serde::Deserialize;
use tracing::{debug, instrument};

use crate::models::ArtifactHubPackage;

const DEFAULT_BASE_URL: &str = "https://artifacthub.io/api/v1";
/// Max packages to attach per project to avoid huge payloads.
const MAX_PACKAGES_PER_PROJECT: usize = 20;

/// Client for the Artifact Hub public API.
#[derive(Debug, Clone)]
pub struct ArtifactHubClient {
    base_url: String,
    client: reqwest::Client,
}

impl ArtifactHubClient {
    /// Build a new client. Uses default base URL if not provided.
    pub fn new(base_url: Option<String>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("mcp-atlas/1.0 (MCPAtlas - CNCF Landscape MCP Server)")
            .build()
            .context("build Artifact Hub HTTP client")?;
        Ok(Self {
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            client,
        })
    }

    /// Search packages by text query. Kind 0 = Helm charts.
    /// Rate: stay under ~100 req/min (no auth); use semaphore in pipeline.
    #[instrument(skip(self), level = "debug")]
    pub async fn search_packages(
        &self,
        ts_query: &str,
        kind: u8,
        limit: usize,
    ) -> Result<Vec<ArtifactHubPackage>> {
        let limit = limit.min(MAX_PACKAGES_PER_PROJECT);
        let url = Url::parse_with_params(
            &format!("{}/packages/search", self.base_url),
            &[
                ("ts_query_web", ts_query),
                ("kind", kind.to_string().as_str()),
                ("limit", limit.to_string().as_str()),
            ],
        )
        .context("build Artifact Hub search URL")?;

        let res = self
            .client
            .get(url)
            .send()
            .await
            .context("Artifact Hub search request")?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            anyhow::bail!("Artifact Hub search failed {}: {}", status, body);
        }

        let body = res.text().await.context("read Artifact Hub response")?;
        let parsed: ArtifactHubSearchResponse =
            serde_json::from_str(&body).context("parse Artifact Hub search response")?;

        let packages: Vec<ArtifactHubPackage> = parsed
            .packages
            .into_iter()
            .map(|p| ArtifactHubPackage {
                name: p.name,
                version: p.version,
                repository: p.repository.name,
                description: Some(p.description).filter(|s| !s.is_empty()),
                stars: Some(p.stars),
                signed: Some(p.signed),
                has_values_schema: Some(p.has_values_schema),
            })
            .collect();

        debug!(
            "Artifact Hub search '{}' returned {} packages",
            ts_query,
            packages.len()
        );
        Ok(packages)
    }
}

/// Response shape for GET /packages/search (PackageSummary from Artifact Hub API).
#[derive(Debug, Deserialize)]
struct ArtifactHubSearchResponse {
    packages: Vec<ArtifactHubPackageHit>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ArtifactHubPackageHit {
    name: String,
    version: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    stars: u64,
    repository: ArtifactHubRepoRef,
    #[serde(default)]
    signed: bool,
    #[serde(default)]
    has_values_schema: bool,
}

#[derive(Debug, Deserialize)]
struct ArtifactHubRepoRef {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_builds_with_default_url() {
        let client = ArtifactHubClient::new(None).unwrap();
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn client_builds_with_custom_url() {
        let client = ArtifactHubClient::new(Some("https://example.com/api".into())).unwrap();
        assert_eq!(client.base_url, "https://example.com/api");
    }
}
