use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde_json::{Value, json};

use cncf_mcp_data::models::ProjectSummary;

use crate::server::AppState;

/// Handle `suggest_stack` tool call.
/// Recommends a cloud-native stack based on a use case description.
/// Matches use case keywords against categories/subcategories and picks
/// the highest-starred project from each relevant area.
pub fn handle_suggest_stack(state: &Arc<AppState>, args: &Value) -> Result<Value> {
    let use_case = args
        .get("use_case")
        .and_then(|u| u.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: use_case"))?;

    let constraints: Vec<String> = args
        .get("constraints")
        .and_then(|c| serde_json::from_value(c.clone()).ok())
        .unwrap_or_default();

    let use_case_lower = use_case.to_lowercase();

    // Map common keywords to real CNCF landscape subcategories.
    // The subcategory matching uses `contains` so partial names work.
    let keyword_map: Vec<(&str, &[&str])> = vec![
        ("monitor", &["Observability"]),
        ("observ", &["Observability"]),
        ("log", &["Observability"]),
        ("trace", &["Observability"]),
        ("mesh", &["Service Mesh"]),
        ("proxy", &["Service Proxy"]),
        ("gateway", &["API Gateway"]),
        ("api", &["API Gateway"]),
        ("container", &["Container Runtime", "Container Registry"]),
        ("registry", &["Container Registry"]),
        ("runtime", &["Container Runtime"]),
        ("orchestrat", &["Scheduling & Orchestration"]),
        ("schedul", &["Scheduling & Orchestration"]),
        (
            "kubernetes",
            &[
                "Scheduling & Orchestration",
                "Certified Kubernetes - Distribution",
            ],
        ),
        ("k8s", &["Scheduling & Orchestration"]),
        ("storage", &["Cloud Native Storage"]),
        ("network", &["Cloud Native Network"]),
        ("security", &["Security & Compliance", "Key Management"]),
        ("secret", &["Key Management"]),
        ("key", &["Key Management"]),
        ("ci", &["Continuous Integration & Delivery"]),
        ("cd", &["Continuous Integration & Delivery"]),
        ("deploy", &["Continuous Integration & Delivery"]),
        ("serverless", &["Installable Platform", "Hosted Platform"]),
        ("function", &["Installable Platform"]),
        ("database", &["Database"]),
        ("stream", &["Streaming & Messaging"]),
        ("messag", &["Streaming & Messaging"]),
        ("queue", &["Streaming & Messaging"]),
        ("wasm", &["Container Runtime"]),
        ("automat", &["Automation & Configuration"]),
        ("config", &["Automation & Configuration"]),
        ("service discover", &["Coordination & Service Discovery"]),
        ("dns", &["Coordination & Service Discovery"]),
        ("chaos", &["Chaos Engineering"]),
        ("feature flag", &["Feature Flagging"]),
        ("paas", &["PaaS/Container Service"]),
        ("rpc", &["Remote Procedure Call"]),
        ("grpc", &["Remote Procedure Call"]),
    ];

    // Find matching subcategories
    let mut matched_subcats: Vec<&str> = Vec::new();
    for (keyword, subcats) in &keyword_map {
        if use_case_lower.contains(keyword) {
            for subcat in *subcats {
                if !matched_subcats.contains(subcat) {
                    matched_subcats.push(subcat);
                }
            }
        }
    }

    // If no keywords matched, suggest broad defaults
    if matched_subcats.is_empty() {
        matched_subcats = vec![
            "Scheduling & Orchestration",
            "Observability",
            "Service Mesh",
            "Container Runtime",
        ];
    }

    // For each subcategory, pick the best project (highest stars among graduated/incubating)
    let mut stack: Vec<(String, ProjectSummary)> = Vec::new();

    for subcat in &matched_subcats {
        let mut candidates: Vec<_> = state
            .projects
            .iter()
            .filter(|p| {
                let subcat_match = p
                    .subcategory
                    .to_lowercase()
                    .contains(&subcat.to_lowercase());
                let constraint_ok = constraints.iter().all(|c| {
                    let c_lower = c.to_lowercase();
                    if let Some(excluded) = c_lower.strip_prefix("no ") {
                        let lang = p
                            .github
                            .as_ref()
                            .and_then(|g| g.language.as_deref())
                            .unwrap_or("");
                        !lang.to_lowercase().contains(excluded)
                    } else {
                        true
                    }
                });
                subcat_match && constraint_ok
            })
            .collect();

        // Sort: graduated first, then by stars
        candidates.sort_by(|a, b| {
            let maturity_ord = |p: &cncf_mcp_data::models::Project| match &p.maturity {
                Some(cncf_mcp_data::models::Maturity::Graduated) => 0,
                Some(cncf_mcp_data::models::Maturity::Incubating) => 1,
                Some(cncf_mcp_data::models::Maturity::Sandbox) => 2,
                _ => 3,
            };
            let ord = maturity_ord(a).cmp(&maturity_ord(b));
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
            let stars_a = a.github.as_ref().map(|g| g.stars).unwrap_or(0);
            let stars_b = b.github.as_ref().map(|g| g.stars).unwrap_or(0);
            stars_b.cmp(&stars_a)
        });

        if let Some(best) = candidates.first() {
            stack.push((subcat.to_string(), ProjectSummary::from(*best)));
        }
    }

    // Format output
    let mut output = format!("## Suggested Cloud-Native Stack\n\n**Use case:** {use_case}\n\n");

    if !constraints.is_empty() {
        output.push_str(&format!("**Constraints:** {}\n\n", constraints.join(", ")));
    }

    if stack.is_empty() {
        output.push_str("No matching projects found for this use case.\n");
    } else {
        output.push_str("| Layer | Recommended Project | Maturity | Stars |\n");
        output.push_str("|-------|-------------------|----------|-------|\n");
        for (subcat, project) in &stack {
            let maturity = project
                .maturity
                .as_ref()
                .map(|m| format!("{m:?}"))
                .unwrap_or_else(|| "N/A".into());
            let stars = project
                .stars
                .map(|s| s.to_string())
                .unwrap_or_else(|| "N/A".into());
            output.push_str(&format!(
                "| {} | **{}** | {} | {} |\n",
                subcat, project.name, maturity, stars
            ));
        }

        output.push_str("\n### Project Details\n\n");
        for (subcat, project) in &stack {
            output.push_str(&format!(
                "- **{}** ({}): {}\n",
                project.name,
                subcat,
                project.description.as_deref().unwrap_or("No description"),
            ));
            if let Some(url) = &project.homepage_url {
                output.push_str(&format!("  Homepage: {url}\n"));
            }
        }
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Handle `analyze_trends` tool call.
/// Provides adoption metrics and trends for projects in a category.
pub fn handle_analyze_trends(state: &Arc<AppState>, args: &Value) -> Result<Value> {
    let category = args
        .get("category")
        .and_then(|c| c.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: category"))?;

    let cat_lower = category.to_lowercase();

    // Find all projects in this category
    let projects_in_cat: Vec<_> = state
        .projects
        .iter()
        .filter(|p| p.category.to_lowercase().contains(&cat_lower))
        .collect();

    if projects_in_cat.is_empty() {
        return Ok(json!({
            "content": [{ "type": "text", "text": format!("No projects found in category matching \"{category}\".") }]
        }));
    }

    // Compute category-level stats
    let total = projects_in_cat.len();
    let with_github = projects_in_cat
        .iter()
        .filter(|p| p.github.is_some())
        .count();

    let total_stars: u64 = projects_in_cat
        .iter()
        .filter_map(|p| p.github.as_ref().map(|g| g.stars))
        .sum();

    let avg_stars = if with_github > 0 {
        total_stars / with_github as u64
    } else {
        0
    };

    let graduated = projects_in_cat
        .iter()
        .filter(|p| matches!(p.maturity, Some(cncf_mcp_data::models::Maturity::Graduated)))
        .count();
    let incubating = projects_in_cat
        .iter()
        .filter(|p| {
            matches!(
                p.maturity,
                Some(cncf_mcp_data::models::Maturity::Incubating)
            )
        })
        .count();
    let sandbox = projects_in_cat
        .iter()
        .filter(|p| matches!(p.maturity, Some(cncf_mcp_data::models::Maturity::Sandbox)))
        .count();

    // Subcategory breakdown
    let mut subcats: HashMap<&str, Vec<_>> = HashMap::new();
    for p in &projects_in_cat {
        subcats.entry(p.subcategory.as_str()).or_default().push(p);
    }

    // Top projects by stars
    let mut top_by_stars: Vec<_> = projects_in_cat
        .iter()
        .filter(|p| p.github.is_some())
        .collect();
    top_by_stars.sort_by(|a, b| {
        let sa = a.github.as_ref().map(|g| g.stars).unwrap_or(0);
        let sb = b.github.as_ref().map(|g| g.stars).unwrap_or(0);
        sb.cmp(&sa)
    });

    // Language distribution
    let mut languages: HashMap<&str, usize> = HashMap::new();
    for p in &projects_in_cat {
        if let Some(gh) = &p.github
            && let Some(lang) = &gh.language
        {
            *languages.entry(lang.as_str()).or_default() += 1;
        }
    }
    let mut lang_sorted: Vec<_> = languages.into_iter().collect();
    lang_sorted.sort_by(|a, b| b.1.cmp(&a.1));

    // Build output
    let actual_category = projects_in_cat
        .first()
        .map(|p| p.category.as_str())
        .unwrap_or(category);

    let mut output = format!("## Trend Analysis: {actual_category}\n\n");
    output.push_str("### Overview\n\n");
    output.push_str(&format!("- **Total projects:** {total}\n"));
    output.push_str(&format!("- **Graduated:** {graduated}\n"));
    output.push_str(&format!("- **Incubating:** {incubating}\n"));
    output.push_str(&format!("- **Sandbox:** {sandbox}\n"));
    output.push_str(&format!("- **Total GitHub stars:** {total_stars}\n"));
    output.push_str(&format!("- **Average stars:** {avg_stars}\n\n"));

    // Subcategory breakdown
    output.push_str("### Subcategory Breakdown\n\n");
    let mut subcat_list: Vec<_> = subcats.iter().collect();
    subcat_list.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    for (subcat, projects) in &subcat_list {
        let sub_stars: u64 = projects
            .iter()
            .filter_map(|p| p.github.as_ref().map(|g| g.stars))
            .sum();
        output.push_str(&format!(
            "- **{}**: {} projects ({} total stars)\n",
            subcat,
            projects.len(),
            sub_stars
        ));
    }

    // Top 5 projects
    output.push_str("\n### Top Projects by Stars\n\n");
    for (i, p) in top_by_stars.iter().take(5).enumerate() {
        let stars = p.github.as_ref().map(|g| g.stars).unwrap_or(0);
        let maturity = p
            .maturity
            .as_ref()
            .map(|m| format!("[{m:?}]"))
            .unwrap_or_default();
        output.push_str(&format!(
            "{}. **{}** {maturity} — {} stars\n",
            i + 1,
            p.name,
            stars
        ));
    }

    // Language distribution
    if !lang_sorted.is_empty() {
        output.push_str("\n### Language Distribution\n\n");
        for (lang, count) in lang_sorted.iter().take(8) {
            output.push_str(&format!("- {lang}: {count} projects\n"));
        }
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Handle `get_migration_path` — structured migration guide from one CNCF project to another.
/// Uses the knowledge graph path and project comparison (category, language, maturity).
pub async fn handle_get_migration_path(state: &Arc<AppState>, args: &Value) -> Result<Value> {
    let from_name = args
        .get("from_project")
        .or_else(|| args.get("from"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: from_project (or from)"))?;
    let to_name = args
        .get("to_project")
        .or_else(|| args.get("to"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: to_project (or to)"))?;

    let from_lower = from_name.to_lowercase();
    let to_lower = to_name.to_lowercase();
    let from_proj = state
        .projects
        .iter()
        .find(|p| p.name.to_lowercase() == from_lower);
    let to_proj = state
        .projects
        .iter()
        .find(|p| p.name.to_lowercase() == to_lower);

    let (Some(from_p), Some(to_p)) = (from_proj, to_proj) else {
        let missing = if from_proj.is_none() && to_proj.is_none() {
            format!("Both \"{from_name}\" and \"{to_name}\" not found.")
        } else if from_proj.is_none() {
            format!("\"{from_name}\" not found.")
        } else {
            format!("\"{to_name}\" not found.")
        };
        return Ok(json!({
            "content": [{ "type": "text", "text": format!("Migration path: {missing}") }]
        }));
    };

    let path = state.graph.find_path(from_name, to_name, 5).await?;

    let mut lines = vec![
        format!("## Migration Guide: {from_name} → {to_name}\n"),
        "### Key Differences\n".to_string(),
    ];

    lines.push(format!(
        "| Attribute | **{}** | **{}** |",
        from_p.name, to_p.name
    ));
    lines.push("|-----------|--------|--------|".to_string());
    lines.push(format!(
        "| Category | {} > {} | {} > {} |",
        from_p.category,
        from_p.subcategory,
        to_p.category,
        to_p.subcategory
    ));
    lines.push(format!(
        "| Maturity | {} | {} |",
        from_p
            .maturity
            .as_ref()
            .map(|m| format!("{m:?}"))
            .unwrap_or_else(|| "N/A".into()),
        to_p
            .maturity
            .as_ref()
            .map(|m| format!("{m:?}"))
            .unwrap_or_else(|| "N/A".into())
    ));
    let lang_a = from_p
        .github
        .as_ref()
        .and_then(|g| g.language.as_deref())
        .unwrap_or("—");
    let lang_b = to_p
        .github
        .as_ref()
        .and_then(|g| g.language.as_deref())
        .unwrap_or("—");
    lines.push(format!("| Language | {lang_a} | {lang_b} |"));
    lines.push(String::new());

    if let Some(edges) = path {
        lines.push("### Relationship Path\n".to_string());
        for (i, e) in edges.iter().enumerate() {
            lines.push(format!(
                "{}. **{}** —[{:?}]→ **{}**",
                i + 1,
                e.from,
                e.relation,
                e.to
            ));
        }
        lines.push(String::new());
        lines.push("### Migration Steps (high level)\n".to_string());
        lines.push("1. Review documentation and APIs of the target project.".to_string());
        lines.push("2. Identify feature parity and gaps between source and target.".to_string());
        lines.push("3. Plan a phased rollout or parallel run if possible.".to_string());
        lines.push("4. Update dependencies, config, and integration points.".to_string());
        lines.push("5. Run tests and validate before full cutover.".to_string());
    } else {
        lines.push("_No direct relationship path in the knowledge graph; projects may be in different domains._".to_string());
        lines.push("Consider comparing documentation and use cases manually.".to_string());
    }

    lines.push(String::new());
    if let Some(url) = &to_p.homepage_url {
        lines.push(format!("**Target:** [{}]({url})", to_p.name));
    }
    if let Some(url) = &to_p.repo_url {
        lines.push(format!("**Repo:** {url}"));
    }

    Ok(json!({
        "content": [{ "type": "text", "text": lines.join("\n") }]
    }))
}
