# Deployment

## Docker

Use the official image:

```bash
docker run -p 3000:3000 ghcr.io/mcp-atlas/mcp-atlas:latest
```

For production, set `GITHUB_TOKEN` via secrets and consider `--rate-limit` and `--skip-github` depending on your startup and scaling needs.

## Kubernetes / Helm

A Helm chart is provided under `deploy/helm/mcp-atlas/`. Key values:

- `replicaCount` — Number of server replicas.
- `transport`, `port` — SSE transport and port.
- `cache.backend` — `file` or `redis` when Redis is enabled.
- `autoscaling.enabled` — HPA for CPU-based scaling.

See `deploy/helm/mcp-atlas/values.yaml` and the repository root [MCP_BLUEPRINT.md](https://github.com/mcp-atlas/mcp-atlas/blob/main/MCP_BLUEPRINT.md) for full options.

## GitHub Pages / Site

The project also has a Vite-based landing site in `site/` with in-app docs. Build with `cd site && npm ci && npm run build`. The Pages workflow deploys from `site/dist`.
