use anyhow::{Context, Result};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, doc};
use tracing::info;

use mcp_atlas_data::models::{Project, ProjectSummary};

/// In-memory full-text search index over CNCF projects.
pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    _schema: Schema,
    // Field handles
    f_name: Field,
    f_description: Field,
    f_category: Field,
    _f_subcategory: Field,
    f_json: Field,
}

impl SearchIndex {
    /// Build a new search index from a list of projects.
    pub fn build(projects: &[Project]) -> Result<Self> {
        let mut schema_builder = Schema::builder();
        let f_name = schema_builder.add_text_field("name", TEXT | STORED);
        let f_description = schema_builder.add_text_field("description", TEXT);
        let f_category = schema_builder.add_text_field("category", TEXT | STORED);
        let f_subcategory = schema_builder.add_text_field("subcategory", TEXT | STORED);
        let f_json = schema_builder.add_text_field("json", STORED);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema.clone());
        let mut writer: IndexWriter = index
            .writer(50_000_000) // 50 MB heap
            .context("Failed to create index writer")?;

        for project in projects {
            let json = serde_json::to_string(&ProjectSummary::from(project)).unwrap_or_default();
            writer.add_document(doc!(
                f_name => project.name.as_str(),
                f_description => project.description.as_deref().unwrap_or(""),
                f_category => project.category.as_str(),
                f_subcategory => project.subcategory.as_str(),
                f_json => json.as_str(),
            ))?;
        }

        writer.commit().context("Failed to commit search index")?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .context("Failed to create index reader")?;

        info!("Search index built with {} documents", projects.len());

        Ok(Self {
            index,
            reader,
            _schema: schema,
            f_name,
            f_description,
            f_category,
            _f_subcategory: f_subcategory,
            f_json,
        })
    }

    /// Search projects by a free-text query string.
    /// Optional `min_stars` and `language` are applied as post-filters after Tantivy results.
    pub fn search(
        &self,
        query_str: &str,
        limit: usize,
        min_stars: Option<u64>,
        language: Option<&str>,
    ) -> Result<Vec<ProjectSummary>> {
        let searcher = self.reader.searcher();
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.f_name, self.f_description, self.f_category],
        );

        let query = query_parser
            .parse_query(query_str)
            .context("Failed to parse search query")?;

        // When filters are present, fetch more docs so we have enough after post-filtering
        let fetch_limit = if min_stars.is_some() || language.is_some() {
            limit.saturating_mul(5).max(100)
        } else {
            limit
        };

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(fetch_limit))
            .context("Search execution failed")?;

        let mut results = Vec::with_capacity(top_docs.len());
        for (_score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .context("Failed to retrieve document")?;

            if let Some(json_value) = doc.get_first(self.f_json)
                && let Some(json_str) = json_value.as_str()
                && let Ok(summary) = serde_json::from_str::<ProjectSummary>(json_str)
            {
                let passes_min_stars = min_stars
                    .map(|min| summary.stars.map(|s| s >= min).unwrap_or(false))
                    .unwrap_or(true);
                let passes_language = language
                    .map(|lang| {
                        summary
                            .language
                            .as_ref()
                            .map(|l| l.eq_ignore_ascii_case(lang))
                            .unwrap_or(false)
                    })
                    .unwrap_or(true);
                if passes_min_stars && passes_language {
                    results.push(summary);
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get total number of indexed documents.
    pub fn doc_count(&self) -> u64 {
        self.reader.searcher().num_docs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_atlas_data::models::Project;

    fn sample_projects() -> Vec<Project> {
        vec![
            Project {
                name: "Prometheus".into(),
                description: Some("Monitoring system and time series database".into()),
                homepage_url: Some("https://prometheus.io".into()),
                repo_url: Some("https://github.com/prometheus/prometheus".into()),
                logo: None,
                crunchbase: None,
                category: "Observability and Analysis".into(),
                subcategory: "Monitoring".into(),
                maturity: Some(mcp_atlas_data::models::Maturity::Graduated),
                extra: Default::default(),
                github: None,
                artifact_hub_packages: None,
                summary: None,
                summary_content_hash: None,
            },
            Project {
                name: "Envoy".into(),
                description: Some("Cloud-native high-performance edge/middle/service proxy".into()),
                homepage_url: Some("https://www.envoyproxy.io".into()),
                repo_url: Some("https://github.com/envoyproxy/envoy".into()),
                logo: None,
                crunchbase: None,
                category: "Orchestration & Management".into(),
                subcategory: "Service Proxy".into(),
                maturity: Some(mcp_atlas_data::models::Maturity::Graduated),
                extra: Default::default(),
                github: None,
                artifact_hub_packages: None,
                summary: None,
                summary_content_hash: None,
            },
        ]
    }

    #[test]
    fn test_build_and_search() {
        let projects = sample_projects();
        let index = SearchIndex::build(&projects).unwrap();
        assert_eq!(index.doc_count(), 2);

        let results = index.search("monitoring", 10, None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Prometheus");

        let results = index.search("proxy", 10, None, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Envoy");
    }
}
