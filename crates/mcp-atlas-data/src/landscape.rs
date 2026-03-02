use anyhow::{Context, Result};
use tracing::info;

use crate::models::{Landscape, Maturity, Project};

/// URL to fetch the raw CNCF landscape YAML from GitHub.
const LANDSCAPE_RAW_URL: &str =
    "https://raw.githubusercontent.com/cncf/landscape/master/landscape.yml";

/// Parse landscape YAML content into a structured [`Landscape`].
pub fn parse_landscape_yaml(yaml_content: &str) -> Result<Landscape> {
    let landscape: Landscape =
        serde_yaml::from_str(yaml_content).context("Failed to parse landscape YAML")?;
    Ok(landscape)
}

/// Flatten all categories/subcategories into a flat list of projects,
/// populating each project's `category` and `subcategory` fields.
pub fn flatten_projects(landscape: &Landscape) -> Vec<Project> {
    let mut projects = Vec::new();
    for category in &landscape.landscape {
        for subcategory in &category.subcategories {
            for item in &subcategory.items {
                let mut project = item.clone();
                project.category = category.name.clone();
                project.subcategory = subcategory.name.clone();
                // Derive maturity from extra fields if not already set
                if project.maturity.is_none() {
                    project.maturity = if project.extra.graduated.is_some() {
                        Some(Maturity::Graduated)
                    } else if project.extra.incubating.is_some() {
                        Some(Maturity::Incubating)
                    } else if project.extra.accepted.is_some() {
                        Some(Maturity::Sandbox)
                    } else {
                        None
                    };
                }
                projects.push(project);
            }
        }
    }
    projects
}

/// Fetch the landscape YAML from the CNCF GitHub repository.
pub async fn fetch_landscape_yaml() -> Result<String> {
    info!("Fetching CNCF landscape YAML from {}", LANDSCAPE_RAW_URL);
    let response = reqwest::get(LANDSCAPE_RAW_URL)
        .await
        .context("Failed to fetch landscape YAML")?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("HTTP {} fetching landscape YAML", status);
    }

    let body = response
        .text()
        .await
        .context("Failed to read landscape YAML response body")?;

    info!("Fetched landscape YAML ({} bytes)", body.len());
    Ok(body)
}

/// Convenience: fetch and parse the landscape in one call.
pub async fn load_landscape() -> Result<(Landscape, Vec<Project>)> {
    let yaml = fetch_landscape_yaml().await?;
    let landscape = parse_landscape_yaml(&yaml)?;
    let projects = flatten_projects(&landscape);
    info!("Loaded {} projects from CNCF landscape", projects.len());
    Ok((landscape, projects))
}

/// Load landscape from a local file path.
pub fn load_landscape_from_file(path: &std::path::Path) -> Result<(Landscape, Vec<Project>)> {
    let yaml = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read landscape file: {}", path.display()))?;
    let landscape = parse_landscape_yaml(&yaml)?;
    let projects = flatten_projects(&landscape);
    info!("Loaded {} projects from {}", projects.len(), path.display());
    Ok((landscape, projects))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_landscape() {
        let yaml = r#"
landscape:
  - name: Observability and Analysis
    subcategories:
      - name: Monitoring
        items:
          - name: Prometheus
            homepage_url: https://prometheus.io
            repo_url: https://github.com/prometheus/prometheus
            logo: prometheus.svg
            crunchbase: https://www.crunchbase.com/organization/cloud-native-computing-foundation
            description: Monitoring system and time series database
"#;
        let landscape = parse_landscape_yaml(yaml).unwrap();
        assert_eq!(landscape.landscape.len(), 1);
        assert_eq!(landscape.landscape[0].name, "Observability and Analysis");

        let projects = flatten_projects(&landscape);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Prometheus");
        assert_eq!(projects[0].category, "Observability and Analysis");
        assert_eq!(projects[0].subcategory, "Monitoring");
    }

    #[test]
    fn test_maturity_from_project_field() {
        let yaml = r#"
landscape:
  - name: Orchestration
    subcategories:
      - name: Scheduling
        items:
          - name: Kubernetes
            homepage_url: https://kubernetes.io
            project: graduated
            logo: k8s.svg
            crunchbase: https://www.crunchbase.com/organization/cncf
          - name: Keda
            homepage_url: https://keda.sh
            project: incubating
            logo: keda.svg
            crunchbase: https://www.crunchbase.com/organization/cncf
          - name: NewProject
            homepage_url: https://example.com
            project: sandbox
            logo: new.svg
            crunchbase: https://www.crunchbase.com/organization/cncf
"#;
        let landscape = parse_landscape_yaml(yaml).unwrap();
        let projects = flatten_projects(&landscape);
        assert_eq!(projects.len(), 3);

        assert_eq!(projects[0].maturity, Some(Maturity::Graduated));
        assert_eq!(projects[1].maturity, Some(Maturity::Incubating));
        assert_eq!(projects[2].maturity, Some(Maturity::Sandbox));
    }

    #[test]
    fn test_maturity_from_extra_dates() {
        let yaml = r#"
landscape:
  - name: Observability
    subcategories:
      - name: Monitoring
        items:
          - name: LegacyProject
            homepage_url: https://example.com
            logo: legacy.svg
            crunchbase: https://www.crunchbase.com/organization/cncf
            extra:
              accepted: "2020-01-01"
              graduated: "2022-01-01"
"#;
        let landscape = parse_landscape_yaml(yaml).unwrap();
        let projects = flatten_projects(&landscape);
        assert_eq!(projects.len(), 1);
        // Should derive Graduated from extra.graduated date
        assert_eq!(projects[0].maturity, Some(Maturity::Graduated));
    }
}
