use std::collections::HashMap;

use async_trait::async_trait;
use tracing::info;

use mcp_atlas_data::models::Project;
use mcp_atlas_data::storage::GraphBackend;

use crate::schema::{GraphStats, ProjectEdge, Relation};

/// In-memory knowledge graph built from CNCF project data.
///
/// Relationships are inferred automatically:
/// - **AlternativeTo**: projects in the same subcategory
/// - **IntegratesWith**: curated map of known CNCF integrations
/// - **ComponentOf**: curated map of project component relationships
#[derive(Debug)]
pub struct ProjectGraph {
    /// Adjacency list: project name → list of edges originating from it.
    edges: HashMap<String, Vec<ProjectEdge>>,
    /// Total edge count (for stats).
    edge_count: usize,
}

impl ProjectGraph {
    /// Build the graph from a list of projects (infers + curated edges).
    pub fn build(projects: &[Project]) -> Self {
        let edge_list = compute_edges(projects);
        Self::from_edges(&edge_list)
    }

    /// Build the graph from a precomputed list of edges (shared with SurrealDB backend).
    pub fn from_edges(edges: &[ProjectEdge]) -> Self {
        let mut by_from: HashMap<String, Vec<ProjectEdge>> = HashMap::new();
        for e in edges {
            by_from.entry(e.from.clone()).or_default().push(e.clone());
        }
        let edge_count = edges.len();
        info!(
            "Built knowledge graph: {} edges across {} projects",
            edge_count,
            by_from.len()
        );
        Self {
            edges: by_from,
            edge_count,
        }
    }

    /// Get all relationships for a project (inherent; see also `GraphBackend`).
    pub fn get_edges_sync(&self, project: &str) -> &[ProjectEdge] {
        let key = self.find_key(project);
        match key {
            Some(k) => self.edges.get(k).map(|v| v.as_slice()).unwrap_or(&[]),
            None => &[],
        }
    }

    /// Convenience alias for `get_edges_sync` (backward compat).
    pub fn get_edges(&self, project: &str) -> &[ProjectEdge] {
        self.get_edges_sync(project)
    }

    /// Get relationships of a specific type for a project.
    pub fn get_edges_by_type(&self, project: &str, relation: &Relation) -> Vec<&ProjectEdge> {
        self.get_edges_sync(project)
            .iter()
            .filter(|e| &e.relation == relation)
            .collect()
    }

    /// Find the shortest path between two projects (BFS, up to max_depth hops).
    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<ProjectEdge>> {
        self.find_path_with_depth(from, to, 5)
    }

    /// Find path with configurable max depth (used by `GraphBackend`).
    pub fn find_path_with_depth(
        &self,
        from: &str,
        to: &str,
        max_depth: u8,
    ) -> Option<Vec<ProjectEdge>> {
        let from_key = self.find_key(from)?;
        let to_lower = to.to_lowercase();
        let max_d = max_depth.min(10) as u32;

        let mut visited: HashMap<String, (String, ProjectEdge)> = HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((from_key.to_string(), 0u32));

        while let Some((current, depth)) = queue.pop_front() {
            if depth > max_d {
                continue;
            }

            let edges = self
                .edges
                .get(&current)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            for edge in edges {
                if visited.contains_key(&edge.to)
                    || edge.to.to_lowercase() == from_key.to_lowercase()
                {
                    continue;
                }

                visited.insert(edge.to.clone(), (current.clone(), edge.clone()));

                if edge.to.to_lowercase() == to_lower {
                    // Reconstruct path
                    let mut path = Vec::new();
                    let mut cur = edge.to.clone();
                    while let Some((prev, e)) = visited.get(&cur) {
                        path.push(e.clone());
                        cur = prev.clone();
                        if cur.to_lowercase() == from_key.to_lowercase() {
                            break;
                        }
                    }
                    path.reverse();
                    return Some(path);
                }

                queue.push_back((edge.to.clone(), depth + 1));
            }
        }

        None
    }

    /// Get graph statistics (sync; see also `GraphBackend::stats`).
    pub fn stats_sync(&self) -> GraphStats {
        let mut relation_counts: HashMap<String, usize> = HashMap::new();
        for edges in self.edges.values() {
            for e in edges {
                let key = format!("{:?}", e.relation);
                *relation_counts.entry(key).or_default() += 1;
            }
        }

        GraphStats {
            total_nodes: self.edges.len(),
            total_edges: self.edge_count,
            relation_counts,
        }
    }

    /// Convenience: same as `stats_sync()` for backward compatibility.
    pub fn stats(&self) -> GraphStats {
        self.stats_sync()
    }

    /// Case-insensitive key lookup.
    fn find_key(&self, name: &str) -> Option<&str> {
        let lower = name.to_lowercase();
        self.edges
            .keys()
            .find(|k| k.to_lowercase() == lower)
            .map(|k| k.as_str())
    }
}

#[async_trait]
impl GraphBackend for ProjectGraph {
    async fn get_edges(&self, project: &str) -> anyhow::Result<Vec<ProjectEdge>> {
        Ok(self.get_edges_sync(project).to_vec())
    }

    async fn find_path(
        &self,
        from: &str,
        to: &str,
        max_depth: u8,
    ) -> anyhow::Result<Option<Vec<ProjectEdge>>> {
        Ok(self.find_path_with_depth(from, to, max_depth))
    }

    async fn stats(&self) -> anyhow::Result<GraphStats> {
        Ok(self.stats_sync())
    }
}

/// Compute confidence for an AlternativeTo edge based on shared signals.
fn compute_alternative_confidence(projects: &[Project], a: &str, b: &str) -> f64 {
    let find = |name: &str| projects.iter().find(|p| p.name == name);
    let pa = match find(a) {
        Some(p) => p,
        None => return 0.5,
    };
    let pb = match find(b) {
        Some(p) => p,
        None => return 0.5,
    };

    let mut score: f64 = 0.5; // Base confidence for same subcategory

    // Both have maturity ratings → higher confidence they're true alternatives
    if pa.maturity.is_some() && pb.maturity.is_some() {
        score += 0.1;
    }

    // Same programming language → even more likely true alternatives
    let lang_a = pa.github.as_ref().and_then(|g| g.language.as_deref());
    let lang_b = pb.github.as_ref().and_then(|g| g.language.as_deref());
    if let (Some(la), Some(lb)) = (lang_a, lang_b)
        && la.to_lowercase() == lb.to_lowercase()
    {
        score += 0.1;
    }

    // Both graduated → high confidence
    use mcp_atlas_data::models::Maturity;
    if pa.maturity == Some(Maturity::Graduated) && pb.maturity == Some(Maturity::Graduated) {
        score += 0.1;
    }

    score.min(1.0)
}

/// Compute all graph edges from projects (inferred + curated). Shared by in-memory and SurrealDB backends.
pub fn compute_edges(projects: &[Project]) -> Vec<ProjectEdge> {
    let mut out: Vec<ProjectEdge> = Vec::new();

    // Phase 1: Infer AlternativeTo from shared subcategories
    let mut by_subcategory: HashMap<&str, Vec<&str>> = HashMap::new();
    for p in projects {
        if !p.subcategory.is_empty() {
            by_subcategory
                .entry(p.subcategory.as_str())
                .or_default()
                .push(p.name.as_str());
        }
    }

    for names in by_subcategory.values() {
        if names.len() < 2 {
            continue;
        }
        for (i, &a) in names.iter().enumerate() {
            for &b in &names[i + 1..] {
                let confidence = compute_alternative_confidence(projects, a, b);
                out.push(ProjectEdge {
                    from: a.to_string(),
                    to: b.to_string(),
                    relation: Relation::AlternativeTo,
                    confidence,
                });
                out.push(ProjectEdge {
                    from: b.to_string(),
                    to: a.to_string(),
                    relation: Relation::AlternativeTo,
                    confidence,
                });
            }
        }
    }

    // Phase 2: Add curated integration edges
    let project_set: HashMap<&str, bool> =
        projects.iter().map(|p| (p.name.as_str(), true)).collect();
    for (from, to, relation, confidence) in curated_edges() {
        if project_set.contains_key(from) && project_set.contains_key(to) {
            out.push(ProjectEdge {
                from: from.to_string(),
                to: to.to_string(),
                relation,
                confidence,
            });
        }
    }

    out
}

/// Curated list of known CNCF project integrations.
///
/// Format: (from, to, relation, confidence).
/// Names must match the `Project.name` field from the landscape YAML.
fn curated_edges() -> Vec<(&'static str, &'static str, Relation, f64)> {
    vec![
        // ── Kubernetes core components ──────────────────────────────
        ("etcd", "Kubernetes", Relation::ComponentOf, 0.99),
        ("CoreDNS", "Kubernetes", Relation::ComponentOf, 0.95),
        ("containerd", "Kubernetes", Relation::ComponentOf, 0.90),
        ("CRI-O", "Kubernetes", Relation::ComponentOf, 0.85),
        // ── Kubernetes extensions ───────────────────────────────────
        ("Helm", "Kubernetes", Relation::Extends, 0.95),
        ("Argo", "Kubernetes", Relation::Extends, 0.95),
        ("Flux", "Kubernetes", Relation::Extends, 0.95),
        ("Knative", "Kubernetes", Relation::Extends, 0.95),
        ("KubeVirt", "Kubernetes", Relation::Extends, 0.90),
        ("KubeEdge", "Kubernetes", Relation::Extends, 0.90),
        ("KEDA", "Kubernetes", Relation::Extends, 0.90),
        ("Crossplane", "Kubernetes", Relation::Extends, 0.90),
        ("Operator Framework", "Kubernetes", Relation::Extends, 0.90),
        ("Virtual Kubelet", "Kubernetes", Relation::Extends, 0.85),
        ("Volcano", "Kubernetes", Relation::Extends, 0.85),
        ("Tekton", "Kubernetes", Relation::Extends, 0.90),
        ("cert-manager", "Kubernetes", Relation::Extends, 0.90),
        ("external-secrets", "Kubernetes", Relation::Extends, 0.85),
        ("Longhorn", "Kubernetes", Relation::Extends, 0.85),
        ("MetalLB", "Kubernetes", Relation::Extends, 0.85),
        ("Dapr", "Kubernetes", Relation::Extends, 0.85),
        ("OpenKruise", "Kubernetes", Relation::Extends, 0.85),
        ("Kured", "Kubernetes", Relation::Extends, 0.80),
        ("Litmus", "Kubernetes", Relation::Extends, 0.80),
        ("Chaos Mesh", "Kubernetes", Relation::Extends, 0.80),
        ("KubeVela", "Kubernetes", Relation::Extends, 0.85),
        ("Backstage", "Kubernetes", Relation::IntegratesWith, 0.80),
        ("Kubeflow", "Kubernetes", Relation::Extends, 0.90),
        ("k3s", "Kubernetes", Relation::Extends, 0.90),
        ("OpenYurt", "Kubernetes", Relation::Extends, 0.85),
        // ── Observability stack ─────────────────────────────────────
        ("Prometheus", "Grafana", Relation::IntegratesWith, 0.95),
        ("Prometheus", "Thanos", Relation::IntegratesWith, 0.90),
        ("Prometheus", "Cortex", Relation::IntegratesWith, 0.85),
        ("Thanos", "Prometheus", Relation::Extends, 0.95),
        ("Cortex", "Prometheus", Relation::Extends, 0.90),
        ("OpenMetrics", "Prometheus", Relation::Extends, 0.90),
        (
            "OpenTelemetry",
            "Prometheus",
            Relation::IntegratesWith,
            0.85,
        ),
        ("OpenTelemetry", "Jaeger", Relation::IntegratesWith, 0.85),
        ("OpenTelemetry", "Grafana", Relation::IntegratesWith, 0.80),
        ("Jaeger", "OpenTelemetry", Relation::IntegratesWith, 0.90),
        ("Fluentd", "Prometheus", Relation::IntegratesWith, 0.70),
        ("Fluentd", "OpenTelemetry", Relation::IntegratesWith, 0.75),
        ("Pixie", "Kubernetes", Relation::IntegratesWith, 0.85),
        ("Pixie", "OpenTelemetry", Relation::IntegratesWith, 0.75),
        ("Keptn", "Prometheus", Relation::IntegratesWith, 0.80),
        ("Keptn", "Kubernetes", Relation::Extends, 0.85),
        ("Trickster", "Prometheus", Relation::IntegratesWith, 0.85),
        ("Perses", "Prometheus", Relation::IntegratesWith, 0.90),
        ("OpenCost", "Prometheus", Relation::IntegratesWith, 0.80),
        ("OpenCost", "Kubernetes", Relation::IntegratesWith, 0.85),
        (
            "Inspektor Gadget",
            "Kubernetes",
            Relation::IntegratesWith,
            0.85,
        ),
        // ── Service mesh ────────────────────────────────────────────
        ("Envoy", "Istio", Relation::ComponentOf, 0.95),
        ("Envoy", "Linkerd", Relation::IntegratesWith, 0.60),
        ("Linkerd", "Kubernetes", Relation::Extends, 0.95),
        ("Istio", "Kubernetes", Relation::Extends, 0.95),
        ("Kuma", "Envoy", Relation::IntegratesWith, 0.90),
        ("Kuma", "Kubernetes", Relation::Extends, 0.85),
        ("Cilium", "Kubernetes", Relation::Extends, 0.90),
        ("Cilium", "Envoy", Relation::IntegratesWith, 0.75),
        (
            "Network Service Mesh",
            "Kubernetes",
            Relation::Extends,
            0.80,
        ),
        ("Meshery", "Istio", Relation::IntegratesWith, 0.80),
        ("Meshery", "Linkerd", Relation::IntegratesWith, 0.80),
        ("Meshery", "Kuma", Relation::IntegratesWith, 0.75),
        ("Aeraki Mesh", "Istio", Relation::Extends, 0.80),
        // ── Networking & ingress ────────────────────────────────────
        (
            "Container Network Interface (CNI)",
            "Cilium",
            Relation::IntegratesWith,
            0.80,
        ),
        (
            "Container Network Interface (CNI)",
            "Antrea",
            Relation::IntegratesWith,
            0.80,
        ),
        ("Contour", "Envoy", Relation::IntegratesWith, 0.90),
        ("Contour", "Kubernetes", Relation::Extends, 0.85),
        ("Emissary-Ingress", "Envoy", Relation::IntegratesWith, 0.90),
        ("Emissary-Ingress", "Kubernetes", Relation::Extends, 0.85),
        ("Antrea", "Kubernetes", Relation::Extends, 0.85),
        ("Kube-OVN", "Kubernetes", Relation::Extends, 0.80),
        ("Submariner", "Kubernetes", Relation::Extends, 0.85),
        ("k8gb", "Kubernetes", Relation::Extends, 0.80),
        ("Kgateway", "Envoy", Relation::IntegratesWith, 0.85),
        ("Kgateway", "Kubernetes", Relation::Extends, 0.85),
        // ── Storage ─────────────────────────────────────────────────
        ("Rook", "Kubernetes", Relation::Extends, 0.85),
        ("OpenEBS", "Kubernetes", Relation::Extends, 0.85),
        ("CubeFS", "Kubernetes", Relation::IntegratesWith, 0.80),
        ("Piraeus Datastore", "Kubernetes", Relation::Extends, 0.80),
        ("Vitess", "Kubernetes", Relation::IntegratesWith, 0.80),
        ("TiKV", "Kubernetes", Relation::IntegratesWith, 0.75),
        // ── Security & policy ───────────────────────────────────────
        (
            "Open Policy Agent (OPA)",
            "Kubernetes",
            Relation::IntegratesWith,
            0.85,
        ),
        ("Falco", "Kubernetes", Relation::IntegratesWith, 0.85),
        ("SPIFFE", "SPIRE", Relation::IntegratesWith, 0.95),
        ("Kyverno", "Kubernetes", Relation::Extends, 0.90),
        ("Kubewarden", "Kubernetes", Relation::Extends, 0.85),
        ("KubeArmor", "Kubernetes", Relation::Extends, 0.85),
        ("Kubescape", "Kubernetes", Relation::IntegratesWith, 0.85),
        ("Keycloak", "Kubernetes", Relation::IntegratesWith, 0.75),
        ("Notary Project", "Harbor", Relation::IntegratesWith, 0.85),
        (
            "in-toto",
            "The Update Framework (TUF)",
            Relation::IntegratesWith,
            0.90,
        ),
        ("Falco", "OpenTelemetry", Relation::IntegratesWith, 0.70),
        ("cert-manager", "SPIFFE", Relation::IntegratesWith, 0.70),
        ("OpenFGA", "Keycloak", Relation::IntegratesWith, 0.70),
        ("Ratify", "Notary Project", Relation::IntegratesWith, 0.85),
        ("Copa", "Kubernetes", Relation::IntegratesWith, 0.75),
        // ── CI/CD & GitOps ──────────────────────────────────────────
        ("Argo", "Helm", Relation::IntegratesWith, 0.85),
        ("Argo", "Flux", Relation::IntegratesWith, 0.50),
        ("Flux", "Helm", Relation::IntegratesWith, 0.85),
        ("Shipwright", "Tekton", Relation::IntegratesWith, 0.85),
        ("Shipwright", "Buildpacks", Relation::IntegratesWith, 0.80),
        ("Buildpacks", "Kubernetes", Relation::IntegratesWith, 0.75),
        ("PipeCD", "Kubernetes", Relation::Extends, 0.80),
        ("Backstage", "Argo", Relation::IntegratesWith, 0.70),
        ("Backstage", "Tekton", Relation::IntegratesWith, 0.70),
        // ── Container registry & artifacts ──────────────────────────
        ("Harbor", "Kubernetes", Relation::IntegratesWith, 0.85),
        ("Distribution", "Harbor", Relation::ComponentOf, 0.85),
        ("ORAS", "Harbor", Relation::IntegratesWith, 0.80),
        ("Artifact Hub", "Helm", Relation::IntegratesWith, 0.80),
        ("Dragonfly", "Harbor", Relation::IntegratesWith, 0.80),
        ("Dragonfly", "Kubernetes", Relation::IntegratesWith, 0.75),
        // ── Serverless & events ─────────────────────────────────────
        ("Knative", "Istio", Relation::IntegratesWith, 0.80),
        ("Knative", "Contour", Relation::IntegratesWith, 0.75),
        ("CloudEvents", "Knative", Relation::IntegratesWith, 0.85),
        ("CloudEvents", "NATS", Relation::IntegratesWith, 0.70),
        ("NATS", "Kubernetes", Relation::IntegratesWith, 0.80),
        ("Strimzi", "Kubernetes", Relation::Extends, 0.85),
        ("Dapr", "NATS", Relation::IntegratesWith, 0.70),
        // ── Multi-cluster & edge ────────────────────────────────────
        ("Karmada", "Kubernetes", Relation::Extends, 0.90),
        (
            "Open Cluster Management",
            "Kubernetes",
            Relation::Extends,
            0.85,
        ),
        ("Clusternet", "Kubernetes", Relation::Extends, 0.80),
        ("KubeStellar", "Kubernetes", Relation::Extends, 0.80),
        // ── WASM runtime ────────────────────────────────────────────
        (
            "WasmEdge Runtime",
            "Kubernetes",
            Relation::IntegratesWith,
            0.70,
        ),
        ("wasmCloud", "Kubernetes", Relation::IntegratesWith, 0.70),
        ("wasmCloud", "NATS", Relation::IntegratesWith, 0.80),
        // ── Messaging & data ────────────────────────────────────────
        ("gRPC", "Envoy", Relation::IntegratesWith, 0.85),
        ("gRPC", "OpenTelemetry", Relation::IntegratesWith, 0.75),
        // ── Dev experience ──────────────────────────────────────────
        ("Telepresence", "Kubernetes", Relation::IntegratesWith, 0.85),
        ("Score", "Kubernetes", Relation::IntegratesWith, 0.75),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_atlas_data::models::{GitHubMetrics, Maturity, Project};

    fn make_project(
        name: &str,
        subcat: &str,
        maturity: Option<Maturity>,
        lang: Option<&str>,
    ) -> Project {
        Project {
            name: name.to_string(),
            description: Some(format!("{name} project")),
            homepage_url: None,
            repo_url: None,
            logo: None,
            crunchbase: None,
            category: "Test".into(),
            subcategory: subcat.into(),
            maturity,
            extra: Default::default(),
            github: lang.map(|l| GitHubMetrics {
                stars: 1000,
                forks: 100,
                open_issues: 10,
                contributors: 50,
                last_commit: None,
                license: None,
                language: Some(l.to_string()),
            }),
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        }
    }

    #[test]
    fn test_alternatives_inferred() {
        let projects = vec![
            make_project(
                "Prometheus",
                "Monitoring",
                Some(Maturity::Graduated),
                Some("Go"),
            ),
            make_project(
                "Thanos",
                "Monitoring",
                Some(Maturity::Incubating),
                Some("Go"),
            ),
            make_project("Cortex", "Monitoring", Some(Maturity::Sandbox), Some("Go")),
        ];

        let graph = ProjectGraph::build(&projects);

        let prom_edges = graph.get_edges("Prometheus");
        assert!(
            prom_edges.len() >= 2,
            "Prometheus should have edges to Thanos and Cortex"
        );

        let alternatives = graph.get_edges_by_type("Prometheus", &Relation::AlternativeTo);
        assert_eq!(alternatives.len(), 2);
    }

    #[test]
    fn test_curated_edges() {
        let projects = vec![
            make_project(
                "Kubernetes",
                "Scheduling & Orchestration",
                Some(Maturity::Graduated),
                Some("Go"),
            ),
            make_project(
                "Helm",
                "Application Definition",
                Some(Maturity::Graduated),
                Some("Go"),
            ),
            make_project(
                "etcd",
                "Key Value Store",
                Some(Maturity::Graduated),
                Some("Go"),
            ),
        ];

        let graph = ProjectGraph::build(&projects);

        let helm_edges = graph.get_edges("Helm");
        let extends_k8s: Vec<_> = helm_edges
            .iter()
            .filter(|e| e.to == "Kubernetes" && e.relation == Relation::Extends)
            .collect();
        assert_eq!(extends_k8s.len(), 1);

        let etcd_edges = graph.get_edges("etcd");
        let component_of: Vec<_> = etcd_edges
            .iter()
            .filter(|e| e.to == "Kubernetes" && e.relation == Relation::ComponentOf)
            .collect();
        assert_eq!(component_of.len(), 1);
    }

    #[test]
    fn test_find_path() {
        let projects = vec![
            make_project(
                "Prometheus",
                "Monitoring",
                Some(Maturity::Graduated),
                Some("Go"),
            ),
            make_project("Grafana", "Dashboards", None, Some("TypeScript")),
        ];

        let graph = ProjectGraph::build(&projects);
        let path = graph.find_path("Prometheus", "Grafana");
        assert!(path.is_some(), "Should find path Prometheus → Grafana");
        assert_eq!(path.unwrap().len(), 1);
    }

    #[test]
    fn test_stats() {
        let projects = vec![
            make_project("A", "Cat1", None, None),
            make_project("B", "Cat1", None, None),
            make_project("C", "Cat2", None, None),
        ];

        let graph = ProjectGraph::build(&projects);
        let stats = graph.stats();
        assert!(stats.total_edges >= 2); // A↔B alternatives
        assert!(stats.total_nodes >= 2);
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let projects = vec![
            make_project("Kubernetes", "Orch", Some(Maturity::Graduated), Some("Go")),
            make_project("Helm", "AppDef", Some(Maturity::Graduated), Some("Go")),
        ];

        let graph = ProjectGraph::build(&projects);
        // Helm has a curated Extends→Kubernetes edge; lookup with lowercase should work
        let edges = graph.get_edges("helm");
        assert!(
            !edges.is_empty(),
            "Case-insensitive lookup should find Helm edges"
        );
        assert_eq!(edges[0].to, "Kubernetes");
    }
}
