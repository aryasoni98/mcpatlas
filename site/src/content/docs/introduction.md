# CNCF MCP Server

Query 2,400+ cloud-native projects from any AI assistant  search, compare, and explore the CNCF Landscape via the Model Context Protocol.

## What it does

- **Tools**  Full-text search, project lookup, comparison, alternatives, health scores, stack suggestions, trends, and knowledge-graph relationships (e.g. `get_relationships`, `find_path`, `get_migration_path`).
- **Resources**  URI-based access to landscape overview, categories, and project details (`cncf://` scheme).
- **Prompts**  Pre-built prompts for tool evaluation, migration planning, stack review, and contributor onboarding.
- **Transports**  STDIO (local), HTTP/SSE, and Streamable HTTP (MCP 2025-03-26).

## Who it is for

- **Developers**  Find the right CNCF project for a use case, compare options, and get health and trend data.
- **AI agents**  Structured, real-time access to landscape data via MCP.
- **Operators**  Run the server locally, in Docker, or in Kubernetes; optional GitHub enrichment and caching.

## Documentation

| Topic | Description |
|-------|-------------|
| [Getting started](/docs/getting-started) | Install, run locally, connect a client. |
| [Project structure](/docs/project-structure) | Repo layout, key files, and where docs live. |
| [Configuration](/docs/configuration) | Transport, cache, GitHub token, rate limits. |
| [Architecture](/docs/architecture) | Components, storage, and data pipeline. |
| [Deployment](/docs/deployment) | Docker, Compose, Helm, and Kubernetes. |
| [Roadmap](/docs/roadmap) | Phases, milestones, and success metrics. |
| [Contributing](/docs/contributing) | Build, test, and submit changes. |
