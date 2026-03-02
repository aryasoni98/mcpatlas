//! Embedding provider abstraction for semantic search (Phase 2).
//! Supports Ollama and OpenAI-compatible APIs.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

/// Provider that turns text into embedding vectors for vector search.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Embed a single text. Returns a normalized vector.
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    /// Vector dimension (e.g. 768 for nomic, 1536 for OpenAI).
    fn dimensions(&self) -> usize;
}

/// No-op provider: always returns an error (hybrid search falls back to BM25).
#[derive(Debug, Clone)]
pub struct NoopEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for NoopEmbeddingProvider {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        anyhow::bail!("embedding disabled (no provider configured)")
    }
    fn dimensions(&self) -> usize {
        0
    }
}

/// Ollama embeddings (e.g. nomic-embed-text). Base URL default http://localhost:11434.
#[derive(Debug, Clone)]
pub struct OllamaEmbeddingProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaEmbeddingProvider {
    pub fn new(base_url: Option<String>, model: Option<String>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .context("build Ollama HTTP client")?;
        Ok(Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model: model.unwrap_or_else(|| "nomic-embed-text".to_string()),
            client,
        })
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);
        let body = serde_json::json!({ "model": self.model, "prompt": text });
        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Ollama embeddings request")?;
        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            anyhow::bail!("Ollama embeddings failed {}: {}", status, body);
        }
        let out: OllamaEmbedResponse = res.json().await.context("parse Ollama response")?;
        Ok(out.embedding)
    }
    fn dimensions(&self) -> usize {
        768
    }
}

#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

/// OpenAI-compatible embeddings (OpenAI, vLLM, LiteLLM). POST to /v1/embeddings.
#[derive(Debug, Clone)]
pub struct OpenAICompatibleEmbeddingProvider {
    base_url: String,
    model: String,
    #[allow(dead_code)] // used when building client; kept for potential future use (e.g. refresh)
    api_key: Option<String>,
    client: reqwest::Client,
}

impl OpenAICompatibleEmbeddingProvider {
    pub fn new(base_url: String, model: Option<String>, api_key: Option<String>) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(key) = &api_key {
            let mut auth = reqwest::header::HeaderValue::from_str(&format!("Bearer {key}"))
                .context("invalid API key")?;
            auth.set_sensitive(true);
            headers.insert(reqwest::header::AUTHORIZATION, auth);
        }
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("build OpenAI-compatible HTTP client")?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.unwrap_or_else(|| "text-embedding-3-small".to_string()),
            api_key,
            client,
        })
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAICompatibleEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/v1/embeddings", self.base_url);
        let body = serde_json::json!({
            "input": text,
            "model": self.model
        });
        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("OpenAI-compatible embeddings request")?;
        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            anyhow::bail!("Embeddings API failed {}: {}", status, err_body);
        }
        let out: OpenAIEmbedResponse = res.json().await.context("parse embeddings response")?;
        let embedding = out
            .data
            .into_iter()
            .next()
            .context("no embedding in response")?
            .embedding;
        Ok(embedding)
    }
    fn dimensions(&self) -> usize {
        if self.model.contains("embedding-3-small") {
            1536
        } else if self.model.contains("nomic") {
            768
        } else {
            1536
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbedResponse {
    data: Vec<OpenAIEmbeddingHit>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingHit {
    embedding: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_dimensions_zero() {
        let p = NoopEmbeddingProvider;
        assert_eq!(p.dimensions(), 0);
    }

    #[tokio::test]
    async fn noop_embed_errors() {
        let p = NoopEmbeddingProvider;
        let r = p.embed("hello").await;
        assert!(r.is_err());
    }
}
