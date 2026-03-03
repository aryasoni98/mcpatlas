//! MCP tool definitions and input schemas.

use serde_json::{Value, json};

/// Common annotations for all tools — all are read-only queries.
pub fn read_only_annotations() -> Value {
    json!({
        "title": "CNCF Landscape Query",
        "readOnlyHint": true,
        "destructiveHint": false,
        "idempotentHint": true,
        "openWorldHint": false
    })
}

/// Core (built-in) tools list with input schemas.
pub fn core_tools_list(annot: &Value) -> Vec<Value> {
    let arr = json!([
        {
            "name": "search_projects",
            "description": "Search CNCF projects by keyword, category, or maturity level",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "category": { "type": "string", "description": "Filter by category" },
                    "maturity": { "type": "string", "enum": ["sandbox", "incubating", "graduated"] },
                    "min_stars": { "type": "number", "description": "Minimum GitHub stars filter" },
                    "language": { "type": "string", "description": "Filter by primary programming language (e.g., 'Go', 'Rust')" },
                    "limit": { "type": "number", "description": "Max results per page (default 10)" },
                    "offset": { "type": "number", "description": "Skip N results for pagination (default 0)" }
                },
                "required": ["query"]
            },
            "annotations": annot
        },
        {
            "name": "get_project",
            "description": "Get full details for a specific CNCF project by name",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Project name (e.g., 'prometheus')" }
                },
                "required": ["name"]
            },
            "annotations": annot
        },
        {
            "name": "compare_projects",
            "description": "Compare two or more CNCF projects side-by-side",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projects": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of project names to compare"
                    }
                },
                "required": ["projects"]
            },
            "annotations": annot
        },
        {
            "name": "list_categories",
            "description": "List all CNCF landscape categories and subcategories",
            "inputSchema": { "type": "object", "properties": {} },
            "annotations": annot
        },
        {
            "name": "get_stats",
            "description": "Get overall CNCF landscape statistics",
            "inputSchema": { "type": "object", "properties": {} },
            "annotations": annot
        },
        {
            "name": "find_alternatives",
            "description": "Find alternative CNCF projects in the same subcategory",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Project name to find alternatives for" },
                    "limit": { "type": "number", "description": "Max results (default 10)" }
                },
                "required": ["project"]
            },
            "annotations": annot
        },
        {
            "name": "get_health_score",
            "description": "Get project health metrics and computed health score",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Project name" }
                },
                "required": ["project"]
            },
            "annotations": annot
        },
        {
            "name": "suggest_stack",
            "description": "Suggest a cloud-native stack for a given use case. Recommends graduated/incubating CNCF projects for each architectural layer.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "use_case": { "type": "string", "description": "Description of the architecture needed (e.g., 'microservices with observability and service mesh')" },
                    "constraints": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Constraints (e.g., 'no Java', 'must support ARM')"
                    }
                },
                "required": ["use_case"]
            },
            "annotations": annot
        },
        {
            "name": "analyze_trends",
            "description": "Analyze adoption trends in a CNCF landscape category. Shows project counts, star distribution, maturity breakdown, language stats, and top projects.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "category": { "type": "string", "description": "Category name (e.g., 'Observability', 'Runtime', 'Orchestration')" }
                },
                "required": ["category"]
            },
            "annotations": annot
        },
        {
            "name": "get_relationships",
            "description": "Get all known relationships for a CNCF project (alternatives, integrations, components, extensions)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Project name (e.g., 'Kubernetes')" },
                    "relation": { "type": "string", "description": "Optional filter: 'alternative', 'integrates', 'component', 'extends', 'supersedes'" }
                },
                "required": ["project"]
            },
            "annotations": annot
        },
        {
            "name": "find_path",
            "description": "Find the shortest relationship path between two CNCF projects in the knowledge graph",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from": { "type": "string", "description": "Source project name" },
                    "to": { "type": "string", "description": "Target project name" }
                },
                "required": ["from", "to"]
            },
            "annotations": annot
        },
        {
            "name": "get_graph_stats",
            "description": "Get knowledge graph statistics — total nodes, edges, and relationship type distribution",
            "inputSchema": { "type": "object", "properties": {} },
            "annotations": annot
        },
        {
            "name": "get_good_first_issues",
            "description": "List CNCF projects that are good candidates for contributors (have repo URLs). Optionally filter by language or category. Configure GITHUB_TOKEN to fetch open issues with 'good first issue' label.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "language": { "type": "string", "description": "Filter by primary language (e.g., Go, Rust)" },
                    "category": { "type": "string", "description": "Filter by category or subcategory" },
                    "limit": { "type": "number", "description": "Max projects to return (default 20)" }
                }
            },
            "annotations": annot
        },
        {
            "name": "get_migration_path",
            "description": "Get a structured migration guide from one CNCF project to another. Returns key differences, relationship path, and high-level migration steps.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from_project": { "type": "string", "description": "Source project name" },
                    "to_project": { "type": "string", "description": "Target project name" },
                    "from": { "type": "string", "description": "Alias for from_project" },
                    "to": { "type": "string", "description": "Alias for to_project" }
                },
                "required": []
            },
            "annotations": annot
        },
        {
            "name": "get_issue_context",
            "description": "Get structured context for a GitHub issue to enable AI-assisted resolution. Returns a compact brief with title, summary, labels, suggested files, and CNCF project metadata when available. Requires GITHUB_TOKEN.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo": { "type": "string", "description": "GitHub repository — 'owner/repo' format or full URL (e.g. 'kubernetes/website' or 'https://github.com/kubernetes/website')" },
                    "issue": { "type": "number", "description": "Issue number (e.g. 54739)" }
                },
                "required": ["repo", "issue"]
            },
            "annotations": annot
        }
    ]);
    arr.as_array().cloned().unwrap_or_default()
}
