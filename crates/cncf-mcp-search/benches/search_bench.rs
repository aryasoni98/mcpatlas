use criterion::{Criterion, criterion_group, criterion_main};

use cncf_mcp_data::models::{GitHubMetrics, Maturity, Project};
use cncf_mcp_search::SearchIndex;

fn generate_projects(n: usize) -> Vec<Project> {
    let categories = [
        "Observability",
        "Orchestration",
        "Runtime",
        "Provisioning",
        "Security",
    ];
    let subcats = [
        "Monitoring",
        "Logging",
        "Tracing",
        "Service Mesh",
        "Container Runtime",
    ];
    let maturities = [
        Some(Maturity::Graduated),
        Some(Maturity::Incubating),
        Some(Maturity::Sandbox),
        None,
    ];
    let languages = ["Go", "Rust", "C++", "Java", "Python", "TypeScript"];

    (0..n)
        .map(|i| Project {
            name: format!("project-{i}"),
            description: Some(format!(
                "A cloud-native tool for {} in the {} space",
                subcats[i % subcats.len()],
                categories[i % categories.len()]
            )),
            homepage_url: Some(format!("https://project-{i}.example.com")),
            repo_url: Some(format!("https://github.com/org/project-{i}")),
            logo: None,
            crunchbase: None,
            category: categories[i % categories.len()].into(),
            subcategory: subcats[i % subcats.len()].into(),
            maturity: maturities[i % maturities.len()].clone(),
            extra: Default::default(),
            github: Some(GitHubMetrics {
                stars: (i as u64 + 1) * 100,
                forks: (i as u64 + 1) * 20,
                open_issues: (i as u64) * 5,
                contributors: (i as u64 + 1) * 10,
                last_commit: None,
                license: Some("Apache-2.0".into()),
                language: Some(languages[i % languages.len()].into()),
            }),
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        })
        .collect()
}

fn bench_index_build(c: &mut Criterion) {
    let projects = generate_projects(2000);

    c.bench_function("build_index_2000", |b| {
        b.iter(|| {
            SearchIndex::build(&projects).unwrap();
        })
    });
}

fn bench_search(c: &mut Criterion) {
    let projects = generate_projects(2000);
    let index = SearchIndex::build(&projects).unwrap();

    c.bench_function("search_simple_query", |b| {
        b.iter(|| {
            index.search("monitoring cloud-native", 10, None, None).unwrap();
        })
    });

    c.bench_function("search_single_word", |b| {
        b.iter(|| {
            index.search("container", 10, None, None).unwrap();
        })
    });

    c.bench_function("search_no_results", |b| {
        b.iter(|| {
            index.search("zzzznonexistent", 10, None, None).unwrap();
        })
    });

    c.bench_function("search_large_limit", |b| {
        b.iter(|| {
            index.search("project", 100, None, None).unwrap();
        })
    });
}

criterion_group!(benches, bench_index_build, bench_search);
criterion_main!(benches);
