use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

use mcp_atlas_data::models::Maturity;

#[derive(Parser)]
#[command(name = "mcp-atlas-cli", about = "CLI companion for MCPAtlas")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// GitHub personal access token for API enrichment.
    #[arg(long, env = "GITHUB_TOKEN", global = true)]
    github_token: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show server version info.
    Version,
    /// Trigger a data sync from the CNCF landscape.
    Sync {
        /// Path to save the landscape data (default: ./data/landscape.yml).
        #[arg(long, default_value = "data/landscape.yml")]
        output: String,
        /// Skip GitHub API enrichment.
        #[arg(long)]
        skip_github: bool,
    },
    /// Search CNCF projects by keyword.
    Search {
        /// Search query.
        query: String,
        /// Filter by maturity level.
        #[arg(long)]
        maturity: Option<String>,
        /// Max number of results.
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Path to landscape.yml file.
        #[arg(long, default_value = "data/landscape.yml")]
        data: String,
    },
    /// Inspect a specific CNCF project.
    Inspect {
        /// Project name (e.g., "Kubernetes", "Prometheus").
        name: String,
        /// Path to landscape.yml file.
        #[arg(long, default_value = "data/landscape.yml")]
        data: String,
    },
    /// Show landscape statistics.
    Stats {
        /// Path to landscape.yml file.
        #[arg(long, default_value = "data/landscape.yml")]
        data: String,
    },
    /// Show project relationships from the knowledge graph.
    Graph {
        /// Project name.
        name: String,
        /// Path to landscape.yml file.
        #[arg(long, default_value = "data/landscape.yml")]
        data: String,
    },
    /// List installed plugins.
    Plugins,
    /// Validate a local landscape.yml file.
    Validate {
        /// Path to the landscape YAML file.
        path: String,
    },
    /// Start the MCP server (shortcut — delegates to the main binary).
    Serve {
        /// Transport: stdio or sse.
        #[arg(long, default_value = "stdio")]
        transport: String,
        /// Port for SSE transport.
        #[arg(long, default_value = "3000")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            println!("mcp-atlas-cli v{}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Sync {
            output,
            skip_github,
        } => {
            info!("Fetching CNCF landscape data...");
            let yaml = mcp_atlas_data::landscape::fetch_landscape_yaml().await?;

            let output_path = std::path::Path::new(&output);
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(output_path, &yaml)?;
            info!("Saved landscape YAML to {}", output_path.display());

            let landscape = mcp_atlas_data::landscape::parse_landscape_yaml(&yaml)?;
            let projects = mcp_atlas_data::landscape::flatten_projects(&landscape);
            println!("Synced {} projects from CNCF landscape", projects.len());

            if !skip_github {
                info!("Enriching with GitHub metrics...");
                let config = mcp_atlas_data::pipeline::PipelineConfig {
                    github_token: cli.github_token,
                    landscape_file: Some(output_path.into()),
                    github_concurrency: 5,
                    ..Default::default()
                };
                let enriched = mcp_atlas_data::pipeline::run_pipeline(&config).await?;
                let with_gh = enriched.iter().filter(|p| p.github.is_some()).count();
                println!(
                    "Enriched {with_gh}/{} projects with GitHub metrics",
                    enriched.len()
                );
            }

            println!("Sync complete.");
        }
        Commands::Search {
            query,
            maturity,
            limit,
            data,
        } => {
            let projects = load_projects(&data)?;
            let index = mcp_atlas_search::SearchIndex::build(&projects)?;
            let results = index.search(&query, limit, None, None)?;

            // Post-filter by maturity if specified
            let results: Vec<_> = if let Some(ref m) = maturity {
                let target = parse_maturity(m);
                results
                    .into_iter()
                    .filter(|p| p.maturity == target)
                    .collect()
            } else {
                results
            };

            if results.is_empty() {
                println!("No projects found for '{query}'");
            } else {
                println!("Found {} project(s) for '{query}':\n", results.len());
                for p in &results {
                    let maturity_str = p
                        .maturity
                        .as_ref()
                        .map(|m| format!("{m:?}"))
                        .unwrap_or_else(|| "-".into());
                    let stars = p
                        .stars
                        .map(|s| format!("{s}"))
                        .unwrap_or_else(|| "-".into());
                    println!(
                        "  {:<25} {:<12} {:>8} stars  {}",
                        p.name,
                        maturity_str,
                        stars,
                        p.description.as_deref().unwrap_or("")
                    );
                }
            }
        }
        Commands::Inspect { name, data } => {
            let projects = load_projects(&data)?;
            let lower = name.to_lowercase();
            let project = projects.iter().find(|p| p.name.to_lowercase() == lower);

            match project {
                Some(p) => {
                    println!("Project: {}", p.name);
                    println!("Category: {} / {}", p.category, p.subcategory);
                    if let Some(m) = &p.maturity {
                        println!("Maturity: {m:?}");
                    }
                    if let Some(desc) = &p.description {
                        println!("Description: {desc}");
                    }
                    if let Some(url) = &p.homepage_url {
                        println!("Homepage: {url}");
                    }
                    if let Some(url) = &p.repo_url {
                        println!("Repository: {url}");
                    }
                    if let Some(gh) = &p.github {
                        println!("\nGitHub Metrics:");
                        println!("  Stars: {}", gh.stars);
                        println!("  Forks: {}", gh.forks);
                        println!("  Open Issues: {}", gh.open_issues);
                        println!("  Contributors: {}", gh.contributors);
                        if let Some(lang) = &gh.language {
                            println!("  Language: {lang}");
                        }
                        if let Some(lic) = &gh.license {
                            println!("  License: {lic}");
                        }
                    }
                }
                None => {
                    println!("Project '{name}' not found.");
                    // Suggest closest matches
                    let suggestions: Vec<_> = projects
                        .iter()
                        .filter(|p| p.name.to_lowercase().contains(&lower))
                        .take(5)
                        .map(|p| p.name.as_str())
                        .collect();
                    if !suggestions.is_empty() {
                        println!("Did you mean: {}", suggestions.join(", "));
                    }
                }
            }
        }
        Commands::Stats { data } => {
            let projects = load_projects(&data)?;

            let graduated = projects
                .iter()
                .filter(|p| p.maturity == Some(Maturity::Graduated))
                .count();
            let incubating = projects
                .iter()
                .filter(|p| p.maturity == Some(Maturity::Incubating))
                .count();
            let sandbox = projects
                .iter()
                .filter(|p| p.maturity == Some(Maturity::Sandbox))
                .count();
            let with_github = projects.iter().filter(|p| p.github.is_some()).count();

            let mut categories: std::collections::HashSet<&str> = std::collections::HashSet::new();
            let mut subcategories: std::collections::HashSet<(&str, &str)> =
                std::collections::HashSet::new();
            for p in &projects {
                if !p.category.is_empty() {
                    categories.insert(&p.category);
                    subcategories.insert((&p.category, &p.subcategory));
                }
            }

            let total_stars: u64 = projects
                .iter()
                .filter_map(|p| p.github.as_ref().map(|g| g.stars))
                .sum();

            println!("CNCF Landscape Statistics:");
            println!("  Total Projects: {}", projects.len());
            println!("  Graduated: {graduated}");
            println!("  Incubating: {incubating}");
            println!("  Sandbox: {sandbox}");
            println!("  Categories: {}", categories.len());
            println!("  Subcategories: {}", subcategories.len());
            println!("  With GitHub data: {with_github}");
            println!("  Total Stars: {total_stars}");
        }
        Commands::Graph { name, data } => {
            let projects = load_projects(&data)?;
            let graph = mcp_atlas_graph::engine::ProjectGraph::build(&projects);

            let edges = graph.get_edges(&name);
            if edges.is_empty() {
                println!("No relationships found for '{name}'");
            } else {
                println!("Relationships for {}:\n", name);
                for edge in edges {
                    println!(
                        "  --[{:?}]--> {} (confidence: {:.0}%)",
                        edge.relation,
                        edge.to,
                        edge.confidence * 100.0
                    );
                }
            }
        }
        Commands::Plugins => {
            println!("Installed plugins:");
            println!("  (none — plugin system coming in Phase 3)");
        }
        Commands::Validate { path } => {
            info!("Validating {}...", path);
            let file_path = std::path::Path::new(&path);
            let (landscape, projects) =
                mcp_atlas_data::landscape::load_landscape_from_file(file_path)?;
            println!("Valid landscape file:");
            println!("  Categories: {}", landscape.landscape.len());
            println!("  Projects: {}", projects.len());

            let missing_desc = projects.iter().filter(|p| p.description.is_none()).count();
            let missing_repo = projects.iter().filter(|p| p.repo_url.is_none()).count();
            if missing_desc > 0 {
                println!("  Warning: {missing_desc} projects missing description");
            }
            if missing_repo > 0 {
                println!("  Info: {missing_repo} projects without repo_url");
            }
        }
        Commands::Serve { transport, port } => {
            println!("Starting MCP server (transport={transport}, port={port})...");
            println!("Hint: Run the main binary directly for full server functionality:");
            println!("  mcp-atlas --transport {transport} --port {port}");
        }
    }

    Ok(())
}

/// Load projects from a local landscape.yml file.
fn load_projects(data_path: &str) -> Result<Vec<mcp_atlas_data::models::Project>> {
    let path = std::path::Path::new(data_path);
    if !path.exists() {
        anyhow::bail!(
            "Data file not found: {data_path}\nRun `mcp-atlas-cli sync` first to fetch the landscape data."
        );
    }
    let (_landscape, projects) = mcp_atlas_data::landscape::load_landscape_from_file(path)?;
    Ok(projects)
}

/// Parse a maturity string into the enum.
fn parse_maturity(s: &str) -> Option<Maturity> {
    match s.to_lowercase().as_str() {
        "graduated" => Some(Maturity::Graduated),
        "incubating" => Some(Maturity::Incubating),
        "sandbox" => Some(Maturity::Sandbox),
        _ => None,
    }
}
