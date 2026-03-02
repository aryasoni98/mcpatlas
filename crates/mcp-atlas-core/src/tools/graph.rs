use std::sync::Arc;

use anyhow::Result;
use serde_json::{Value, json};

use mcp_atlas_graph::schema::Relation;

use crate::server::AppState;

/// Handle `get_relationships` — return all relationships for a given project.
pub async fn handle_get_relationships(state: &Arc<AppState>, arguments: &Value) -> Result<Value> {
    let project = arguments
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project"))?;

    let relation_filter = arguments.get("relation").and_then(|v| v.as_str());

    let edges = state.graph.get_edges(project).await?;

    if edges.is_empty() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("No relationships found for project '{project}'. It may not exist in the landscape or have no known relationships.")
            }]
        }));
    }

    // Optionally filter by relation type
    let filtered: Vec<_> = if let Some(rel) = relation_filter {
        let target_rel = parse_relation(rel);
        edges
            .iter()
            .filter(|e| Some(&e.relation) == target_rel.as_ref())
            .collect()
    } else {
        edges.iter().collect()
    };

    let mut lines = vec![format!("## Relationships for {project}\n")];
    lines.push(format!("Found {} relationship(s):\n", filtered.len()));

    // Group by relation type
    let mut by_type: std::collections::HashMap<String, Vec<(&str, f64)>> =
        std::collections::HashMap::new();
    for e in &filtered {
        let key = format!("{:?}", e.relation);
        by_type.entry(key).or_default().push((&e.to, e.confidence));
    }

    for (rel_type, projects) in &by_type {
        lines.push(format!("### {rel_type}"));
        for (name, conf) in projects {
            lines.push(format!("- **{name}** (confidence: {:.0}%)", conf * 100.0));
        }
        lines.push(String::new());
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": lines.join("\n")
        }]
    }))
}

/// Handle `find_path` — find the shortest relationship path between two projects.
pub async fn handle_find_path(state: &Arc<AppState>, arguments: &Value) -> Result<Value> {
    let from = arguments
        .get("from")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: from"))?;
    let to = arguments
        .get("to")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: to"))?;

    let path = state.graph.find_path(from, to, 5).await?;
    match path {
        Some(p) => {
            let mut lines = vec![format!("## Path: {from} → {to}\n")];
            lines.push(format!("{} hop(s):\n", p.len()));

            for (i, edge) in p.iter().enumerate() {
                lines.push(format!(
                    "{}. **{}** —[{:?}]→ **{}** (confidence: {:.0}%)",
                    i + 1,
                    edge.from,
                    edge.relation,
                    edge.to,
                    edge.confidence * 100.0
                ));
            }

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": lines.join("\n")
                }]
            }))
        }
        None => Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("No path found between '{from}' and '{to}' within 5 hops.")
            }]
        })),
    }
}

/// Handle `get_graph_stats` — return knowledge graph statistics.
pub async fn handle_get_graph_stats(state: &Arc<AppState>) -> Result<Value> {
    let stats = state.graph.stats().await?;

    let mut lines = vec!["## Knowledge Graph Statistics\n".to_string()];
    lines.push(format!("- **Nodes:** {}", stats.total_nodes));
    lines.push(format!("- **Edges:** {}", stats.total_edges));
    lines.push(String::new());
    lines.push("### Edge Types".to_string());

    let mut sorted: Vec<_> = stats.relation_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (rel, count) in sorted {
        lines.push(format!("- **{rel}:** {count}"));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": lines.join("\n")
        }]
    }))
}

/// Parse a relation string into a Relation enum.
fn parse_relation(s: &str) -> Option<Relation> {
    match s.to_lowercase().as_str() {
        "alternativeto" | "alternative" | "alternatives" => Some(Relation::AlternativeTo),
        "integrateswith" | "integrates" | "integration" => Some(Relation::IntegratesWith),
        "componentof" | "component" => Some(Relation::ComponentOf),
        "extends" | "extension" => Some(Relation::Extends),
        "supersedes" | "supersede" => Some(Relation::Supersedes),
        _ => None,
    }
}
