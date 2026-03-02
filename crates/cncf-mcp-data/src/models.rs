use serde::{Deserialize, Serialize};

/// Maturity level of a CNCF project.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Maturity {
    Sandbox,
    Incubating,
    Graduated,
    Archived,
    /// Non-CNCF projects/products in the landscape.
    #[serde(other)]
    Unknown,
}

/// A single project or product in the CNCF Landscape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: Option<String>,
    pub homepage_url: Option<String>,
    pub repo_url: Option<String>,
    pub logo: Option<String>,
    pub crunchbase: Option<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub subcategory: String,
    /// Maturity level — populated from the `project` field in the YAML
    /// (e.g. `project: graduated`), or derived from `extra` dates.
    #[serde(default, alias = "project")]
    pub maturity: Option<Maturity>,
    #[serde(default)]
    pub extra: ProjectExtra,
    #[serde(default)]
    pub github: Option<GitHubMetrics>,
    /// Helm charts and other packages from Artifact Hub (when enrichment enabled).
    #[serde(default)]
    pub artifact_hub_packages: Option<Vec<ArtifactHubPackage>>,
    /// Short LLM-generated or fallback summary (when summary enrichment enabled).
    #[serde(default)]
    pub summary: Option<String>,
    /// Hash of project content when summary was generated; used to skip re-summarization.
    #[serde(default)]
    pub summary_content_hash: Option<String>,
}

/// A package (e.g. Helm chart) from Artifact Hub linked to a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactHubPackage {
    pub name: String,
    pub version: String,
    pub repository: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub stars: Option<u64>,
    #[serde(default)]
    pub signed: Option<bool>,
    #[serde(default)]
    pub has_values_schema: Option<bool>,
}

/// Extra metadata fields from the CNCF landscape YAML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectExtra {
    pub accepted: Option<String>,
    pub incubating: Option<String>,
    pub graduated: Option<String>,
    pub dev_stats_url: Option<String>,
    pub artwork_url: Option<String>,
    pub blog_url: Option<String>,
    pub slack_url: Option<String>,
    pub twitter_url: Option<String>,
    pub youtube_url: Option<String>,
    pub stack_overflow_url: Option<String>,
}

/// GitHub repository metrics for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubMetrics {
    pub stars: u64,
    pub forks: u64,
    pub open_issues: u64,
    pub contributors: u64,
    pub last_commit: Option<String>,
    pub license: Option<String>,
    pub language: Option<String>,
}

/// A category in the CNCF Landscape (e.g., "Observability and Analysis").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub subcategories: Vec<Subcategory>,
}

/// A subcategory within a landscape category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subcategory {
    pub name: String,
    pub items: Vec<Project>,
}

/// Top-level structure of the CNCF landscape YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Landscape {
    pub landscape: Vec<Category>,
}

/// Health score for a project (computed metric).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub project_name: String,
    pub overall: f64,
    pub commit_frequency: f64,
    pub issue_response_time: f64,
    pub release_cadence: f64,
    pub contributor_growth: f64,
}

/// Summary result returned by search operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub subcategory: String,
    pub maturity: Option<Maturity>,
    pub stars: Option<u64>,
    /// Primary programming language from GitHub.
    pub language: Option<String>,
    pub homepage_url: Option<String>,
    pub repo_url: Option<String>,
}

impl From<&Project> for ProjectSummary {
    fn from(p: &Project) -> Self {
        Self {
            name: p.name.clone(),
            description: p.description.clone(),
            category: p.category.clone(),
            subcategory: p.subcategory.clone(),
            maturity: p.maturity.clone(),
            stars: p.github.as_ref().map(|g| g.stars),
            language: p.github.as_ref().and_then(|g| g.language.clone()),
            homepage_url: p.homepage_url.clone(),
            repo_url: p.repo_url.clone(),
        }
    }
}
