use anyhow::Result;
use tracing::info;

use crate::artifacthub::ArtifactHubClient;
use crate::github::{build_github_client, fetch_github_metrics, parse_github_url};
use crate::landscape;
use crate::models::Project;
use crate::summary::{project_content_hash, SummaryProvider};

/// Configuration for the data ingestion pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Optional GitHub personal access token for higher rate limits.
    pub github_token: Option<String>,
    /// Optional local path to landscape.yml (skips HTTP fetch if provided).
    pub landscape_file: Option<std::path::PathBuf>,
    /// Max concurrent GitHub API requests.
    pub github_concurrency: usize,
    /// Enable Artifact Hub enrichment (Helm packages per project).
    pub artifact_hub_enabled: bool,
    /// Max Artifact Hub packages to attach per project.
    pub artifact_hub_max_packages: usize,
    /// Delay in ms between Artifact Hub requests to stay under rate limit.
    pub artifact_hub_delay_ms: u64,
    /// Enable LLM summary enrichment (uses fallback when API unavailable).
    pub summary_enabled: bool,
    /// Summary API base URL (e.g. https://api.openai.com or http://localhost:11434).
    pub summary_api_base: Option<String>,
    /// Summary API key (for OpenAI-compatible).
    pub summary_api_key: Option<String>,
    /// Summary model (e.g. gpt-4o-mini, llama3.2).
    pub summary_model: Option<String>,
    /// Max projects to summarize per run (0 = no limit).
    pub summary_max_per_run: usize,
    /// Concurrency for summary requests.
    pub summary_concurrency: usize,
    /// Delay in ms between summary requests.
    pub summary_delay_ms: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            github_token: None,
            landscape_file: None,
            github_concurrency: 5,
            artifact_hub_enabled: false,
            artifact_hub_max_packages: 20,
            artifact_hub_delay_ms: 700,
            summary_enabled: false,
            summary_api_base: None,
            summary_api_key: None,
            summary_model: None,
            summary_max_per_run: 0,
            summary_concurrency: 2,
            summary_delay_ms: 500,
        }
    }
}

/// Run the full data ingestion pipeline.
///
/// 1. Load CNCF landscape (from file or URL)
/// 2. Enrich projects with GitHub metrics
/// 3. Return the enriched project list
pub async fn run_pipeline(config: &PipelineConfig) -> Result<Vec<Project>> {
    // Step 1: Load landscape data
    let (_landscape, mut projects) = match &config.landscape_file {
        Some(path) => landscape::load_landscape_from_file(path)?,
        None => landscape::load_landscape().await?,
    };

    info!("Pipeline loaded {} projects", projects.len());

    // Step 2: Enrich with GitHub metrics (best-effort, skip failures)
    let client = build_github_client(config.github_token.as_deref())?;

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(config.github_concurrency));

    let mut handles = Vec::new();
    for (i, project) in projects.iter().enumerate() {
        let repo_url = match &project.repo_url {
            Some(url) if url.contains("github.com") => url.clone(),
            _ => continue,
        };

        let client = client.clone();
        let sem = semaphore.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await;
            if let Some((owner, repo)) = parse_github_url(&repo_url) {
                match fetch_github_metrics(&client, &owner, &repo).await {
                    Ok(metrics) => Some((i, metrics)),
                    Err(e) => {
                        tracing::debug!("Failed to fetch GitHub metrics for {repo_url}: {e}");
                        None
                    }
                }
            } else {
                None
            }
        }));
    }

    let mut enriched_count = 0;
    for handle in handles {
        if let Ok(Some((i, metrics))) = handle.await {
            projects[i].github = Some(metrics);
            enriched_count += 1;
        }
    }

    info!(
        "Pipeline enriched {enriched_count}/{} projects with GitHub metrics",
        projects.len()
    );

    // Step 3: Optionally enrich with Artifact Hub (Helm packages)
    if config.artifact_hub_enabled {
        let ah_client = ArtifactHubClient::new(None)?;
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(2));
        let delay = std::time::Duration::from_millis(config.artifact_hub_delay_ms);
        let max_packages = config.artifact_hub_max_packages;

        let mut handles = Vec::new();
        for (i, project) in projects.iter().enumerate() {
            let name = project.name.clone();
            if name.is_empty() {
                continue;
            }
            let client = ah_client.clone();
            let sem = semaphore.clone();

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await;
                tokio::time::sleep(delay).await;
                match client
                    .search_packages(&name, 0, max_packages)
                    .await
                {
                    Ok(packages) if !packages.is_empty() => Some((i, packages)),
                    Ok(_) => None,
                    Err(e) => {
                        tracing::debug!("Artifact Hub search for '{}': {}", name, e);
                        None
                    }
                }
            }));
        }

        for handle in handles {
            if let Ok(Some((idx, packages))) = handle.await {
                projects[idx].artifact_hub_packages = Some(packages);
            }
        }

        let ah_count = projects
            .iter()
            .filter(|p| p.artifact_hub_packages.as_ref().is_some_and(|v| !v.is_empty()))
            .count();
        info!(
            "Pipeline enriched {ah_count}/{} projects with Artifact Hub packages",
            projects.len()
        );
    }

    // Step 4: Optionally add LLM summaries (with deterministic fallback on failure)
    if config.summary_enabled {
        if let Some(ref api_base) = config.summary_api_base {
            let provider = SummaryProvider::new(
                api_base.clone(),
                config.summary_model.clone(),
                config.summary_api_key.clone(),
            )?;
            let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(config.summary_concurrency));
            let delay = std::time::Duration::from_millis(config.summary_delay_ms);
            let max_per_run = config.summary_max_per_run;

            let to_summarize: Vec<usize> = projects
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    let hash = project_content_hash(p);
                    let skip = p
                        .summary_content_hash
                        .as_deref()
                        .is_some_and(|h| h == hash);
                    !skip
                })
                .map(|(i, _)| i)
                .take(if max_per_run > 0 {
                    max_per_run
                } else {
                    projects.len()
                })
                .collect();

            for idx in to_summarize {
                let _permit = sem.acquire().await;
                tokio::time::sleep(delay).await;
                let summary = provider.summarize(&projects[idx]).await;
                projects[idx].summary = Some(summary);
                projects[idx].summary_content_hash = Some(project_content_hash(&projects[idx]));
            }

            let with_summary = projects.iter().filter(|p| p.summary.is_some()).count();
            info!(
                "Pipeline set summaries for {with_summary}/{} projects",
                projects.len()
            );
        } else {
            tracing::warn!("summary_enabled but summary_api_base not set; skipping summaries");
        }
    }

    Ok(projects)
}
