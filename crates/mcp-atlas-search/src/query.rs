use mcp_atlas_data::models::Maturity;
use serde::{Deserialize, Serialize};

/// Structured search query for the CNCF landscape.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Free-text search query.
    pub query: String,
    /// Filter by category name.
    pub category: Option<String>,
    /// Filter by maturity level.
    pub maturity: Option<Maturity>,
    /// Minimum GitHub stars.
    pub min_stars: Option<u64>,
    /// Filter by primary programming language.
    pub language: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
}

impl SearchQuery {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn with_maturity(mut self, maturity: Maturity) -> Self {
        self.maturity = Some(maturity);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn effective_limit(&self) -> usize {
        self.limit.unwrap_or(10)
    }
}
