use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{info, warn};

use crate::models::GitHubMetrics;

/// Extract owner/repo from a GitHub URL.
/// e.g., "https://github.com/prometheus/prometheus" → ("prometheus", "prometheus")
pub fn parse_github_url(url: &str) -> Option<(String, String)> {
    let url = url.trim_end_matches('/');
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 2 {
        let repo = parts[parts.len() - 1].to_string();
        let owner = parts[parts.len() - 2].to_string();
        if !owner.is_empty() && !repo.is_empty() {
            return Some((owner, repo));
        }
    }
    None
}

/// Response shape from the GitHub REST API for a repository.
#[derive(Debug, Deserialize)]
struct GitHubRepoResponse {
    stargazers_count: u64,
    forks_count: u64,
    open_issues_count: u64,
    license: Option<GitHubLicense>,
    language: Option<String>,
    pushed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubLicense {
    spdx_id: Option<String>,
}

/// Fetch GitHub metrics for a single repository using the REST API.
pub async fn fetch_github_metrics(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
) -> Result<GitHubMetrics> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}");
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .with_context(|| format!("Failed to fetch GitHub repo {owner}/{repo}"))?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("GitHub API returned HTTP {status} for {owner}/{repo}");
    }

    let data: GitHubRepoResponse = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    Ok(GitHubMetrics {
        stars: data.stargazers_count,
        forks: data.forks_count,
        open_issues: data.open_issues_count,
        contributors: 0, // Requires separate API call
        last_commit: data.pushed_at,
        license: data.license.and_then(|l| l.spdx_id),
        language: data.language,
    })
}

/// Build a reqwest client configured for GitHub API access.
pub fn build_github_client(token: Option<&str>) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .user_agent("mcp-atlas/0.1.0")
        .timeout(std::time::Duration::from_secs(30));

    if let Some(token) = token {
        use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))
                .context("Invalid GitHub token format")?,
        );
        builder = builder.default_headers(headers);
        info!("GitHub client configured with authentication token");
    } else {
        warn!("GitHub client running without token — rate limits will be strict (60 req/hr)");
    }

    builder.build().context("Failed to build HTTP client")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        assert_eq!(
            parse_github_url("https://github.com/prometheus/prometheus"),
            Some(("prometheus".into(), "prometheus".into()))
        );
        assert_eq!(
            parse_github_url("https://github.com/kubernetes/kubernetes/"),
            Some(("kubernetes".into(), "kubernetes".into()))
        );
        assert_eq!(parse_github_url("https://example.com"), None);
    }
}
