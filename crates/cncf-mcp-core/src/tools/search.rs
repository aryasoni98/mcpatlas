use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Result;
use serde_json::{Value, json};

use cncf_mcp_data::models::{Project, ProjectSummary};

use crate::server::AppState;

const RRF_K: u32 = 60;

/// Reciprocal Rank Fusion: merge two ranked lists by name, score = sum 1/(k+rank).
fn reciprocal_rank_fusion(
    bm25: &[ProjectSummary],
    vector: &[(String, f64)],
    k: u32,
) -> Vec<String> {
    let mut scores: HashMap<String, f64> = HashMap::new();
    for (rank_1based, p) in bm25.iter().enumerate() {
        let r = (rank_1based + 1) as u32;
        *scores.entry(p.name.clone()).or_insert(0.0) += 1.0 / (k + r) as f64;
    }
    for (rank_1based, (name, _)) in vector.iter().enumerate() {
        let r = (rank_1based + 1) as u32;
        *scores.entry(name.clone()).or_insert(0.0) += 1.0 / (k + r) as f64;
    }
    let mut order: Vec<(String, f64)> = scores.into_iter().collect();
    order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    order.into_iter().map(|(name, _)| name).collect()
}

fn project_summary_by_name(projects: &[Project], name: &str) -> Option<ProjectSummary> {
    projects
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(name))
        .map(ProjectSummary::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(name: &str) -> ProjectSummary {
        ProjectSummary {
            name: name.into(),
            description: None,
            category: String::new(),
            subcategory: String::new(),
            maturity: None,
            stars: None,
            language: None,
            homepage_url: None,
            repo_url: None,
        }
    }

    #[test]
    fn rrf_merges_two_lists_by_rank() {
        let bm25 = vec![summary("A"), summary("B"), summary("C")];
        let vector = vec![
            ("C".to_string(), 0.9),
            ("A".to_string(), 0.8),
            ("B".to_string(), 0.7),
        ];
        let names = reciprocal_rank_fusion(&bm25, &vector, 60);
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"A".to_string()));
        assert!(names.contains(&"B".to_string()));
        assert!(names.contains(&"C".to_string()));
    }
}

/// Handle `search_projects` tool call.
///
/// Supports pagination via `offset` and `limit`. Uses hybrid (BM25 + vector) search when
/// embedding provider and vector backend are configured.
pub async fn handle_search_projects(state: &Arc<AppState>, args: &Value) -> Result<Value> {
    let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
    let limit = args.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;
    let offset = args.get("offset").and_then(|o| o.as_u64()).unwrap_or(0) as usize;
    let category_filter = args.get("category").and_then(|c| c.as_str());
    let maturity_filter = args.get("maturity").and_then(|m| m.as_str());
    let min_stars = args.get("min_stars").and_then(|s| s.as_u64());
    let language_filter = args.get("language").and_then(|l| l.as_str());

    let fetch_count = (offset + limit) * 3;
    let mut results = if let (Some(provider), Some(backend)) = (
        state.embedding_provider.as_ref(),
        state.vector_backend.as_ref(),
    ) {
        let bm25 = state
            .search_index
            .search(query, fetch_count * 2, min_stars, language_filter)?;
        match provider.embed(query).await {
            Ok(embedding) => {
                let vector_results = backend
                    .search(embedding.as_slice(), fetch_count * 2)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::warn!("Vector search failed, using BM25 only: {}", e);
                        Vec::new()
                    });
                if vector_results.is_empty() {
                    bm25
                } else {
                    let names = reciprocal_rank_fusion(&bm25, &vector_results, RRF_K);
                    names
                        .into_iter()
                        .filter_map(|name| project_summary_by_name(&state.projects, &name))
                        .take(fetch_count)
                        .collect()
                }
            }
            Err(e) => {
                tracing::warn!("Embedding failed, falling back to BM25 only: {}", e);
                bm25
            }
        }
    } else {
        state
            .search_index
            .search(query, fetch_count, min_stars, language_filter)?
    };

    // Apply post-search filters (category, maturity; min_stars/language also re-applied for hybrid path)
    if let Some(cat) = category_filter {
        let cat_lower = cat.to_lowercase();
        results.retain(|p| p.category.to_lowercase().contains(&cat_lower));
    }
    if let Some(mat) = maturity_filter {
        let mat_lower = mat.to_lowercase();
        results.retain(|p| {
            p.maturity
                .as_ref()
                .map(|m| format!("{m:?}").to_lowercase() == mat_lower)
                .unwrap_or(false)
        });
    }
    if let Some(min) = min_stars {
        results.retain(|p| p.stars.map(|s| s >= min).unwrap_or(false));
    }
    if let Some(lang) = language_filter {
        let lang_lower = lang.to_lowercase();
        results.retain(|p| {
            p.language
                .as_ref()
                .map(|l| l.to_lowercase() == lang_lower)
                .unwrap_or(false)
        });
    }

    let total_matching = results.len();
    let has_more = total_matching > offset + limit;

    // Apply offset and limit
    if offset < results.len() {
        results = results[offset..].to_vec();
    } else {
        results.clear();
    }
    results.truncate(limit);

    let page_info = if offset > 0 || has_more {
        format!(
            " (showing {}-{} of {}{})",
            offset + 1,
            offset + results.len(),
            total_matching,
            if has_more { "+" } else { "" }
        )
    } else {
        String::new()
    };

    let content = format!(
        "Found {} projects matching \"{}\"{page_info}:\n\n{}",
        results.len(),
        query,
        results
            .iter()
            .enumerate()
            .map(|(i, p)| format_project_summary(offset + i + 1, p))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(json!({
        "content": [{ "type": "text", "text": content }],
        "_meta": {
            "total": total_matching,
            "offset": offset,
            "limit": limit,
            "hasMore": has_more
        }
    }))
}

/// Handle `get_project` tool call.
pub fn handle_get_project(state: &Arc<AppState>, args: &Value) -> Result<Value> {
    let name = args
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;

    let name_lower = name.to_lowercase();
    let project = state
        .projects
        .iter()
        .find(|p| p.name.to_lowercase() == name_lower);

    match project {
        Some(p) => {
            let detail = serde_json::to_string_pretty(p)?;
            Ok(json!({
                "content": [{ "type": "text", "text": detail }]
            }))
        }
        None => {
            // Suggest similar names
            let suggestions: Vec<_> = state
                .projects
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&name_lower))
                .take(5)
                .map(|p| p.name.as_str())
                .collect();

            let msg = if suggestions.is_empty() {
                format!("Project \"{name}\" not found in the CNCF landscape.")
            } else {
                format!(
                    "Project \"{name}\" not found. Did you mean: {}?",
                    suggestions.join(", ")
                )
            };

            Ok(json!({
                "content": [{ "type": "text", "text": msg }]
            }))
        }
    }
}

/// Handle `compare_projects` tool call.
pub fn handle_compare_projects(state: &Arc<AppState>, args: &Value) -> Result<Value> {
    let project_names: Vec<String> = args
        .get("projects")
        .and_then(|p| serde_json::from_value(p.clone()).ok())
        .unwrap_or_default();

    if project_names.len() < 2 {
        anyhow::bail!("At least 2 project names are required for comparison");
    }

    let mut found = Vec::new();
    let mut not_found = Vec::new();

    for name in &project_names {
        let name_lower = name.to_lowercase();
        match state
            .projects
            .iter()
            .find(|p| p.name.to_lowercase() == name_lower)
        {
            Some(p) => found.push(p),
            None => not_found.push(name.as_str()),
        }
    }

    let mut output = String::from("## CNCF Project Comparison\n\n");

    if !not_found.is_empty() {
        output.push_str(&format!("**Not found:** {}\n\n", not_found.join(", ")));
    }

    if found.len() >= 2 {
        // Build comparison table
        output.push_str("| Attribute | ");
        output.push_str(
            &found
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(" | "),
        );
        output.push_str(" |\n|---|");
        output.push_str(&found.iter().map(|_| "---").collect::<Vec<_>>().join("|"));
        output.push_str("|\n");

        // Category
        output.push_str("| Category | ");
        output.push_str(
            &found
                .iter()
                .map(|p| p.category.as_str())
                .collect::<Vec<_>>()
                .join(" | "),
        );
        output.push_str(" |\n");

        // Maturity
        output.push_str("| Maturity | ");
        output.push_str(
            &found
                .iter()
                .map(|p| {
                    p.maturity
                        .as_ref()
                        .map(|m| format!("{m:?}"))
                        .unwrap_or_else(|| "N/A".into())
                })
                .collect::<Vec<_>>()
                .join(" | "),
        );
        output.push_str(" |\n");

        // Stars
        output.push_str("| GitHub Stars | ");
        output.push_str(
            &found
                .iter()
                .map(|p| {
                    p.github
                        .as_ref()
                        .map(|g| g.stars.to_string())
                        .unwrap_or_else(|| "N/A".into())
                })
                .collect::<Vec<_>>()
                .join(" | "),
        );
        output.push_str(" |\n");

        // Language
        output.push_str("| Language | ");
        output.push_str(
            &found
                .iter()
                .map(|p| {
                    p.github
                        .as_ref()
                        .and_then(|g| g.language.as_deref())
                        .unwrap_or("N/A")
                })
                .collect::<Vec<_>>()
                .join(" | "),
        );
        output.push_str(" |\n");

        // License
        output.push_str("| License | ");
        output.push_str(
            &found
                .iter()
                .map(|p| {
                    p.github
                        .as_ref()
                        .and_then(|g| g.license.as_deref())
                        .unwrap_or("N/A")
                })
                .collect::<Vec<_>>()
                .join(" | "),
        );
        output.push_str(" |\n");
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Handle `list_categories` tool call.
pub fn handle_list_categories(state: &Arc<AppState>) -> Result<Value> {
    let mut categories: HashMap<String, HashSet<String>> = HashMap::new();
    for p in &state.projects {
        categories
            .entry(p.category.clone())
            .or_default()
            .insert(p.subcategory.clone());
    }

    let mut output = String::from("## CNCF Landscape Categories\n\n");
    let mut sorted_cats: Vec<_> = categories.into_iter().collect();
    sorted_cats.sort_by(|a, b| a.0.cmp(&b.0));

    for (cat, subcats) in &sorted_cats {
        output.push_str(&format!("### {cat}\n"));
        let mut sorted_subs: Vec<_> = subcats.iter().collect();
        sorted_subs.sort();
        for sub in sorted_subs {
            let count = state
                .projects
                .iter()
                .filter(|p| p.category == *cat && p.subcategory == *sub)
                .count();
            output.push_str(&format!("- {sub} ({count} projects)\n"));
        }
        output.push('\n');
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Handle `get_stats` tool call.
pub fn handle_get_stats(state: &Arc<AppState>) -> Result<Value> {
    let total = state.projects.len();
    let with_github = state.projects.iter().filter(|p| p.github.is_some()).count();
    let graduated = state
        .projects
        .iter()
        .filter(|p| matches!(p.maturity, Some(cncf_mcp_data::models::Maturity::Graduated)))
        .count();
    let incubating = state
        .projects
        .iter()
        .filter(|p| {
            matches!(
                p.maturity,
                Some(cncf_mcp_data::models::Maturity::Incubating)
            )
        })
        .count();
    let sandbox = state
        .projects
        .iter()
        .filter(|p| matches!(p.maturity, Some(cncf_mcp_data::models::Maturity::Sandbox)))
        .count();

    let categories: HashSet<_> = state.projects.iter().map(|p| &p.category).collect();

    let output = format!(
        "## CNCF Landscape Statistics\n\n\
         - **Total projects/products:** {total}\n\
         - **Categories:** {}\n\
         - **With GitHub data:** {with_github}\n\
         - **Graduated:** {graduated}\n\
         - **Incubating:** {incubating}\n\
         - **Sandbox:** {sandbox}\n",
        categories.len()
    );

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Handle `find_alternatives` tool call.
/// Finds projects in the same subcategory as the given project.
pub fn handle_find_alternatives(state: &Arc<AppState>, args: &Value) -> Result<Value> {
    let name = args
        .get("project")
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project"))?;

    let name_lower = name.to_lowercase();
    let base_project = state
        .projects
        .iter()
        .find(|p| p.name.to_lowercase() == name_lower);

    let Some(base) = base_project else {
        return Ok(json!({
            "content": [{ "type": "text", "text": format!("Project \"{name}\" not found in the CNCF landscape.") }]
        }));
    };

    let mut alternatives: Vec<_> = state
        .projects
        .iter()
        .filter(|p| {
            p.name.to_lowercase() != name_lower
                && p.subcategory == base.subcategory
                && p.category == base.category
        })
        .collect();

    // Sort by stars (descending) if GitHub data is available
    alternatives.sort_by(|a, b| {
        let stars_a = a.github.as_ref().map(|g| g.stars).unwrap_or(0);
        let stars_b = b.github.as_ref().map(|g| g.stars).unwrap_or(0);
        stars_b.cmp(&stars_a)
    });

    let limit = args.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;
    alternatives.truncate(limit);

    let mut output = format!(
        "## Alternatives to {}\n\nCategory: {} > {}\n\n",
        base.name, base.category, base.subcategory
    );

    if alternatives.is_empty() {
        output.push_str("No alternatives found in the same subcategory.");
    } else {
        for (i, p) in alternatives.iter().enumerate() {
            let summary = ProjectSummary::from(*p);
            output.push_str(&format_project_summary(i + 1, &summary));
            output.push('\n');
        }
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Handle `get_health_score` tool call.
/// Computes a basic health score from available GitHub metrics.
pub fn handle_get_health_score(state: &Arc<AppState>, args: &Value) -> Result<Value> {
    let name = args
        .get("project")
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project"))?;

    let name_lower = name.to_lowercase();
    let project = state
        .projects
        .iter()
        .find(|p| p.name.to_lowercase() == name_lower);

    let Some(p) = project else {
        return Ok(json!({
            "content": [{ "type": "text", "text": format!("Project \"{name}\" not found.") }]
        }));
    };

    let mut output = format!("## Health Report: {}\n\n", p.name);

    output.push_str(&format!(
        "- **Category:** {} > {}\n",
        p.category, p.subcategory
    ));
    output.push_str(&format!(
        "- **Maturity:** {}\n",
        p.maturity
            .as_ref()
            .map(|m| format!("{m:?}"))
            .unwrap_or_else(|| "N/A".into())
    ));

    if let Some(gh) = &p.github {
        output.push_str(&format!("- **Stars:** {}\n", gh.stars));
        output.push_str(&format!("- **Forks:** {}\n", gh.forks));
        output.push_str(&format!("- **Open Issues:** {}\n", gh.open_issues));
        if let Some(lang) = &gh.language {
            output.push_str(&format!("- **Language:** {lang}\n"));
        }
        if let Some(license) = &gh.license {
            output.push_str(&format!("- **License:** {license}\n"));
        }
        if let Some(last) = &gh.last_commit {
            output.push_str(&format!("- **Last Push:** {last}\n"));
        }

        // Simple health score: weighted combination of available signals
        let star_score = (gh.stars as f64).ln().min(10.0) / 10.0; // log scale, max ~1.0
        let fork_ratio = if gh.stars > 0 {
            (gh.forks as f64 / gh.stars as f64).min(1.0)
        } else {
            0.0
        };
        let issue_ratio = if gh.stars > 0 {
            1.0 - (gh.open_issues as f64 / gh.stars as f64).min(1.0)
        } else {
            0.5
        };

        let overall = (star_score * 0.4 + fork_ratio * 0.2 + issue_ratio * 0.4) * 100.0;
        output.push_str(&format!("\n**Health Score:** {overall:.0}/100\n"));
        output.push_str("_(Based on stars, fork ratio, and issue ratio. Higher = healthier.)_\n");
    } else {
        output.push_str("\n_No GitHub metrics available for this project._\n");
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Handle `get_good_first_issues` — list CNCF projects that are good candidates for
/// contributors (with repo URLs). Optionally filter by language/category.
/// Actual issue fetch from GitHub requires GITHUB_TOKEN; this returns project list and repo links.
pub fn handle_get_good_first_issues(state: &Arc<AppState>, args: &Value) -> Result<Value> {
    let language_filter = args.get("language").and_then(|l| l.as_str()).map(|s| s.to_lowercase());
    let category_filter = args.get("category").and_then(|c| c.as_str()).map(|s| s.to_lowercase());
    let limit = args
        .get("limit")
        .and_then(|l| l.as_u64())
        .unwrap_or(20) as usize;

    let mut candidates: Vec<_> = state
        .projects
        .iter()
        .filter(|p| p.repo_url.as_ref().is_some())
        .filter(|p| {
            language_filter.as_ref().is_none_or(|lang| {
                p.github
                    .as_ref()
                    .and_then(|g| g.language.as_ref())
                    .map(|l| l.to_lowercase() == *lang)
                    .unwrap_or(false)
            })
        })
        .filter(|p| {
            category_filter.as_ref().is_none_or(|cat| {
                p.category.to_lowercase().contains(cat) || p.subcategory.to_lowercase().contains(cat)
            })
        })
        .collect();

    candidates.sort_by(|a, b| {
        let stars_a = a.github.as_ref().map(|g| g.stars).unwrap_or(0);
        let stars_b = b.github.as_ref().map(|g| g.stars).unwrap_or(0);
        stars_b.cmp(&stars_a)
    });
    candidates.truncate(limit);

    let mut lines = vec![
        "## Good First Issue Candidates (CNCF Projects)\n".to_string(),
        "Projects below have public repos and match your filters. To fetch open issues with the "
            .to_string(),
        "`good first issue` label, configure GITHUB_TOKEN and use the GitHub API or web UI.\n"
            .to_string(),
    ];

    for (i, p) in candidates.iter().enumerate() {
        let repo = p.repo_url.as_deref().unwrap_or("");
        let lang = p
            .github
            .as_ref()
            .and_then(|g| g.language.as_deref())
            .unwrap_or("—");
        lines.push(format!(
            "{}. **{}** — {} | {} | [repo]({})",
            i + 1,
            p.name,
            p.subcategory,
            lang,
            repo
        ));
    }

    if candidates.is_empty() {
        lines.push("\nNo projects matched the filters. Try broadening language or category.".to_string());
    }

    Ok(json!({
        "content": [{ "type": "text", "text": lines.join("\n") }],
        "_meta": { "count": candidates.len() }
    }))
}

fn format_project_summary(index: usize, p: &ProjectSummary) -> String {
    let maturity = p
        .maturity
        .as_ref()
        .map(|m| format!(" [{m:?}]"))
        .unwrap_or_default();
    let stars = p.stars.map(|s| format!(" ⭐{s}")).unwrap_or_default();

    format!(
        "{index}. **{}**{maturity}{stars}\n   {}\n   Category: {} > {}",
        p.name,
        p.description.as_deref().unwrap_or("No description"),
        p.category,
        p.subcategory,
    )
}
