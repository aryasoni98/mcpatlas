# Tools Reference

The server exposes 14+ MCP tools. Key ones:

| Tool | Description |
|------|-------------|
| `search_projects` | Full-text search across 2,400+ projects (keyword, category, maturity). |
| `get_project` | Detailed project info: description, GitHub metrics, maturity, links. |
| `compare_projects` | Side-by-side comparison table for multiple projects. |
| `list_categories` | All landscape categories and subcategories. |
| `get_stats` | Landscape-wide statistics. |
| `find_alternatives` | Projects in the same subcategory. |
| `get_health_score` | Health score from GitHub metrics. |
| `suggest_stack` | Recommend a cloud-native stack for a use case. |
| `analyze_trends` | Adoption metrics and trends per category. |
| `get_relationships` | Knowledge graph edges for a project. |
| `find_path` | Shortest relationship path between two projects. |
| `get_graph_stats` | Knowledge graph statistics. |
| `get_good_first_issues` | Projects good for contributors (filter by language/category). |
| `get_migration_path` | Migration guide from one project to another. |

Tool inputs and outputs follow the MCP tool schema. Use your client’s tool introspection or see the server source in `crates/cncf-mcp-core/src/tools/`.
