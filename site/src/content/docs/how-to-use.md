# How to use

Typical flows for using the CNCF MCP server with your AI assistant.

## Search and compare

1. **Search**  Use `search_projects` with a keyword or category to find relevant projects.
2. **Details**  Use `get_project` with a project name for description, maturity, and links.
3. **Compare**  Use `compare_projects` with multiple project names for a side-by-side table.

## Stack and migration

- **Suggest a stack**  Use `suggest_stack` with a use case (e.g. "Kubernetes observability") to get recommended projects.
- **Migration path**  Use `get_migration_path` with source and target project names for a migration guide.
- **Alternatives**  Use `find_alternatives` to see other projects in the same subcategory.

## Graph and trends

- **Relationships**  Use `get_relationships` for knowledge-graph edges; use `find_path` for the shortest path between two projects.
- **Trends**  Use `analyze_trends` for adoption metrics in a category.

## Next steps

- [Use cases & setup](/docs/use-cases)  Cursor, Claude, Docker, and Kubernetes.
- [Tools reference](/docs/tools-reference)  All tools and parameters.
