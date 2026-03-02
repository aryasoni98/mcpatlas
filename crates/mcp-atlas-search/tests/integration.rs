use mcp_atlas_data::models::{GitHubMetrics, Maturity, Project};
use mcp_atlas_search::SearchIndex;

// Re-create a minimal AppState for testing.
// In a real integration test, we'd import from mcp-atlas-core, but since
// AppState is not publicly exported, we reconstruct the tool dispatch logic here
// by testing the search index and data layer directly.

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
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: Some(GitHubMetrics {
                stars: 55000,
                forks: 8900,
                open_issues: 900,
                contributors: 1000,
                last_commit: Some("2026-02-28T10:00:00Z".into()),
                license: Some("Apache-2.0".into()),
                language: Some("Go".into()),
            }),
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
        Project {
            name: "Grafana".into(),
            description: Some(
                "Open and composable observability and data visualization platform".into(),
            ),
            homepage_url: Some("https://grafana.com".into()),
            repo_url: Some("https://github.com/grafana/grafana".into()),
            logo: None,
            crunchbase: None,
            category: "Observability and Analysis".into(),
            subcategory: "Monitoring".into(),
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: Some(GitHubMetrics {
                stars: 65000,
                forks: 12000,
                open_issues: 4000,
                contributors: 2000,
                last_commit: Some("2026-02-28T12:00:00Z".into()),
                license: Some("AGPL-3.0".into()),
                language: Some("TypeScript".into()),
            }),
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
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: Some(GitHubMetrics {
                stars: 25000,
                forks: 4500,
                open_issues: 1500,
                contributors: 800,
                last_commit: Some("2026-02-27T09:00:00Z".into()),
                license: Some("Apache-2.0".into()),
                language: Some("C++".into()),
            }),
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
        Project {
            name: "Istio".into(),
            description: Some("Connect, secure, control, and observe services".into()),
            homepage_url: Some("https://istio.io".into()),
            repo_url: Some("https://github.com/istio/istio".into()),
            logo: None,
            crunchbase: None,
            category: "Orchestration & Management".into(),
            subcategory: "Service Mesh".into(),
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: Some(GitHubMetrics {
                stars: 36000,
                forks: 7500,
                open_issues: 600,
                contributors: 900,
                last_commit: Some("2026-02-28T08:00:00Z".into()),
                license: Some("Apache-2.0".into()),
                language: Some("Go".into()),
            }),
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
        Project {
            name: "Linkerd".into(),
            description: Some("Ultralight, security-first service mesh for Kubernetes".into()),
            homepage_url: Some("https://linkerd.io".into()),
            repo_url: Some("https://github.com/linkerd/linkerd2".into()),
            logo: None,
            crunchbase: None,
            category: "Orchestration & Management".into(),
            subcategory: "Service Mesh".into(),
            maturity: Some(Maturity::Graduated),
            extra: Default::default(),
            github: Some(GitHubMetrics {
                stars: 10500,
                forks: 1200,
                open_issues: 250,
                contributors: 400,
                last_commit: Some("2026-02-26T14:00:00Z".into()),
                license: Some("Apache-2.0".into()),
                language: Some("Go".into()),
            }),
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
        Project {
            name: "WasmCloud".into(),
            description: Some("Build distributed applications with WebAssembly".into()),
            homepage_url: Some("https://wasmcloud.dev".into()),
            repo_url: Some("https://github.com/wasmCloud/wasmCloud".into()),
            logo: None,
            crunchbase: None,
            category: "Runtime".into(),
            subcategory: "Cloud Native Runtime".into(),
            maturity: Some(Maturity::Sandbox),
            extra: Default::default(),
            github: None,
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        },
    ]
}

#[test]
fn test_search_index_finds_monitoring_tools() {
    let projects = sample_projects();
    let index = SearchIndex::build(&projects).unwrap();

    let results = index.search("monitoring", 10, None, None).unwrap();
    assert!(!results.is_empty());
    // Should find Prometheus and/or Grafana
    let names: Vec<_> = results.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"Prometheus") || names.contains(&"Grafana"));
}

#[test]
fn test_search_index_finds_service_mesh() {
    let projects = sample_projects();
    let index = SearchIndex::build(&projects).unwrap();

    let results = index.search("service mesh", 10, None, None).unwrap();
    let names: Vec<_> = results.iter().map(|r| r.name.as_str()).collect();
    assert!(
        names.contains(&"Istio") || names.contains(&"Linkerd"),
        "Expected Istio or Linkerd in results, got: {:?}",
        names
    );
}

#[test]
fn test_search_index_proxy_query() {
    let projects = sample_projects();
    let index = SearchIndex::build(&projects).unwrap();

    let results = index.search("proxy", 10, None, None).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].name, "Envoy");
}

#[test]
fn test_search_index_doc_count() {
    let projects = sample_projects();
    let index = SearchIndex::build(&projects).unwrap();
    assert_eq!(index.doc_count(), 6);
}

#[test]
fn test_project_summary_conversion() {
    let projects = sample_projects();
    let prometheus = &projects[0];
    let summary = mcp_atlas_data::models::ProjectSummary::from(prometheus);

    assert_eq!(summary.name, "Prometheus");
    assert_eq!(summary.category, "Observability and Analysis");
    assert_eq!(summary.subcategory, "Monitoring");
    assert_eq!(summary.maturity, Some(Maturity::Graduated));
    assert_eq!(summary.stars, Some(55000));
}

#[test]
fn test_landscape_yaml_parsing() {
    let yaml = r#"
landscape:
  - name: Observability and Analysis
    subcategories:
      - name: Monitoring
        items:
          - name: Prometheus
            description: Monitoring system
            homepage_url: https://prometheus.io
            repo_url: https://github.com/prometheus/prometheus
            logo: prometheus.svg
            crunchbase: https://www.crunchbase.com/organization/cloud-native-computing-foundation
      - name: Logging
        items:
          - name: Fluentd
            description: Unified logging layer
            homepage_url: https://www.fluentd.org
            repo_url: https://github.com/fluent/fluentd
            logo: fluentd.svg
            crunchbase: https://www.crunchbase.com/organization/cloud-native-computing-foundation
  - name: Runtime
    subcategories:
      - name: Container Runtime
        items:
          - name: containerd
            description: An industry-standard container runtime
            homepage_url: https://containerd.io
            repo_url: https://github.com/containerd/containerd
            logo: containerd.svg
            crunchbase: https://www.crunchbase.com/organization/cloud-native-computing-foundation
"#;

    let landscape = mcp_atlas_data::landscape::parse_landscape_yaml(yaml).unwrap();
    assert_eq!(landscape.landscape.len(), 2);

    let projects = mcp_atlas_data::landscape::flatten_projects(&landscape);
    assert_eq!(projects.len(), 3);

    // Verify category/subcategory assignment
    let prometheus = projects.iter().find(|p| p.name == "Prometheus").unwrap();
    assert_eq!(prometheus.category, "Observability and Analysis");
    assert_eq!(prometheus.subcategory, "Monitoring");

    let containerd = projects.iter().find(|p| p.name == "containerd").unwrap();
    assert_eq!(containerd.category, "Runtime");
    assert_eq!(containerd.subcategory, "Container Runtime");
}

#[test]
fn test_github_url_parsing() {
    use mcp_atlas_data::github::parse_github_url;

    assert_eq!(
        parse_github_url("https://github.com/prometheus/prometheus"),
        Some(("prometheus".into(), "prometheus".into()))
    );
    assert_eq!(
        parse_github_url("https://github.com/istio/istio/"),
        Some(("istio".into(), "istio".into()))
    );
    assert_eq!(parse_github_url("not-a-url"), None);
    assert_eq!(
        parse_github_url("https://gitlab.com/foo/bar"),
        Some(("foo".into(), "bar".into()))
    );
}
