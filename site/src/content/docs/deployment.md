# Deployment

## Docker

```bash
docker run -p 3000:3000 ghcr.io/cncf-mcp/server:latest
```

Use a volume for cache persistence:

```bash
docker run -p 3000:3000 -v cncf-mcp-cache:/app/cache ghcr.io/cncf-mcp/server:latest
```

## Docker Compose

See `deploy/docker/` in the repo for a Compose file with health checks and cache volume.

## Kubernetes / Helm

```bash
helm install cncf-mcp deploy/helm/cncf-mcp
```

Chart includes Deployment, Service, optional Ingress, HPA, and PDB. See `deploy/helm/README.md` for values and security context.

## Health

- **HTTP:** `GET /health` returns 200 and project count when ready.
- **Readiness:** Server is ready when landscape and search index are loaded.
