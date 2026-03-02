//! LLM-based project summarization with deterministic fallback (Phase 2).

use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tracing::{debug, instrument};

use crate::models::Project;

/// Content hash for a project to detect changes and skip re-summarization.
pub fn project_content_hash(project: &Project) -> String {
    let mut hasher = Sha256::new();
    hasher.update(project.name.as_bytes());
    hasher.update(project.description.as_deref().unwrap_or("").as_bytes());
    hasher.update(project.category.as_bytes());
    hasher.update(project.subcategory.as_bytes());
    if let Some(ref g) = project.github {
        hasher.update(g.language.as_deref().unwrap_or("").as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// Deterministic fallback when LLM is disabled or fails. No randomness.
pub fn fallback_summary(project: &Project) -> String {
    let desc = project
        .description
        .as_deref()
        .unwrap_or("No description")
        .trim();
    let mut s = format!(
        "{} — {}. Category: {} / {}.",
        project.name,
        if desc.is_empty() { "No description" } else { desc },
        project.category,
        project.subcategory
    );
    if s.len() > 500 {
        s.truncate(497);
        s.push_str("...");
    }
    s
}

const MAX_SUMMARY_LEN: usize = 500;

/// Provider that generates short project summaries (LLM or fallback).
#[derive(Debug, Clone)]
pub struct SummaryProvider {
    base_url: String,
    model: String,
    #[allow(dead_code)] // used when building client; kept for potential future use
    api_key: Option<String>,
    client: reqwest::Client,
}

impl SummaryProvider {
    /// OpenAI-compatible chat completion. Base URL e.g. https://api.openai.com or http://localhost:11434 for Ollama.
    pub fn new(base_url: String, model: Option<String>, api_key: Option<String>) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref key) = api_key {
            let mut auth =
                reqwest::header::HeaderValue::from_str(&format!("Bearer {key}")).context("API key")?;
            auth.set_sensitive(true);
            headers.insert(reqwest::header::AUTHORIZATION, auth);
        }
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("summary HTTP client")?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
            api_key,
            client,
        })
    }

    /// Generate a 2–3 sentence summary. On failure returns deterministic fallback.
    #[instrument(skip(self, project), level = "debug")]
    pub async fn summarize(&self, project: &Project) -> String {
        let prompt = format!(
            "In 2-3 sentences, summarize this CNCF project for a technical audience. Focus on what it does and where it fits. Project: {} | Category: {} / {} | Description: {}",
            project.name,
            project.category,
            project.subcategory,
            project.description.as_deref().unwrap_or("(none)")
        );

        let is_ollama = self.base_url.contains("11434");
        let body = if is_ollama {
            serde_json::json!({
                "model": self.model,
                "messages": [{"role": "user", "content": prompt}],
                "stream": false
            })
        } else {
            serde_json::json!({
                "model": self.model,
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 150
            })
        };

        let url = if is_ollama {
            format!("{}/api/chat", self.base_url)
        } else {
            format!("{}/v1/chat/completions", self.base_url)
        };

        match self.client.post(&url).json(&body).send().await {
            Ok(res) if res.status().is_success() => {
                let body = res.text().await.unwrap_or_default();
                let text = if is_ollama {
                    serde_json::from_str::<OllamaChatResponse>(&body)
                        .ok()
                        .map(|r| r.response)
                } else {
                    serde_json::from_str::<ChatResponse>(&body)
                        .ok()
                        .and_then(|r| r.choices.into_iter().next().and_then(|c| c.message.content))
                };
                if text.is_none() {
                    debug!("summary parse: no content from API");
                }
                if let Some(mut t) = text {
                    t = t.trim().to_string();
                    if t.len() > MAX_SUMMARY_LEN {
                        t.truncate(MAX_SUMMARY_LEN - 3);
                        t.push_str("...");
                    }
                    if !t.is_empty() {
                        return t;
                    }
                }
            }
            Ok(res) => {
                debug!("summary API status: {}", res.status());
            }
            Err(e) => {
                debug!("summary request error: {}", e);
            }
        }
        fallback_summary(project)
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: Option<String>,
}

/// Ollama /api/chat response (stream: false).
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    response: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Maturity;

    fn sample_project() -> Project {
        Project {
            name: "Prometheus".into(),
            description: Some("Monitoring system".into()),
            homepage_url: None,
            repo_url: None,
            logo: None,
            crunchbase: None,
            category: "Observability".into(),
            subcategory: "Monitoring".into(),
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: None,
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        }
    }

    #[test]
    fn fallback_summary_format() {
        let p = sample_project();
        let s = fallback_summary(&p);
        assert!(s.contains("Prometheus"));
        assert!(s.contains("Observability"));
        assert!(s.contains("Monitoring"));
    }

    #[test]
    fn content_hash_deterministic() {
        let p = sample_project();
        let h1 = project_content_hash(&p);
        let h2 = project_content_hash(&p);
        assert_eq!(h1, h2);
    }
}
