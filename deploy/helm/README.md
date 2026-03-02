# Helm chart for CNCF MCP Server

Deploy the CNCF MCP Server on Kubernetes (Blueprint §4b).

## Prerequisites

- Kubernetes 1.21+
- Helm 3.x (optional; can use `kubectl apply` with rendered manifests)

## Install

From the repo root:

```bash
# Install with default values (single replica, no ingress)
helm install cncf-mcp ./deploy/helm/cncf-mcp

# Or with custom image and replicas
helm install cncf-mcp ./deploy/helm/cncf-mcp \
  --set image.repository=ghcr.io/your-org/cncf-mcp-server \
  --set image.tag=v0.1.0 \
  --set replicaCount=2
```

## Configure

| Value | Default | Description |
|-------|---------|-------------|
| `replicaCount` | 1 | Number of server replicas |
| `image.repository` | ghcr.io/cncf-mcp/server | Container image |
| `image.tag` | latest | Image tag |
| `transport` | sse | MCP transport (sse) |
| `port` | 3000 | HTTP port |
| `skipGithub` | true | Skip GitHub enrichment at startup |
| `github.enabled` | false | Use GITHUB_TOKEN from a secret |
| `ingress.enabled` | false | Create Ingress for external access |
| `autoscaling.enabled` | false | Enable HPA |
| `pdb.enabled` | false | Enable PodDisruptionBudget |

## Examples

**Enable Ingress (e.g. for Cursor/IDE):**

```bash
helm upgrade --install cncf-mcp ./deploy/helm/cncf-mcp \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=cncf-mcp.example.com
```

**GitHub token from secret:**

```bash
kubectl create secret generic cncf-mcp-github --from-literal=token=ghp_xxx
helm upgrade --install cncf-mcp ./deploy/helm/cncf-mcp \
  --set github.enabled=true \
  --set github.secretName=cncf-mcp-github \
  --set skipGithub=false
```

**Enable HPA and PDB:**

```bash
helm upgrade --install cncf-mcp ./deploy/helm/cncf-mcp \
  --set autoscaling.enabled=true \
  --set autoscaling.minReplicas=2 \
  --set autoscaling.maxReplicas=10 \
  --set pdb.enabled=true
```

## MCP endpoint

- In-cluster: `http://<service-name>.<namespace>.svc.cluster.local:3000/sse`
- With ingress: `https://<ingress-host>/sse`
- Health: `GET /health`

## Values schema and security

The chart includes `values.schema.json` for Helm 3 value validation. Install with schema check:

```bash
helm install cncf-mcp ./deploy/helm/cncf-mcp --validate
```

**Security defaults:** The deployment sets `runAsNonRoot: true`, `allowPrivilegeEscalation: false`, drops all capabilities, and uses `RuntimeDefault` seccomp. The container image runs as a non-root user; the cache volume at `/app/cache` is writable.

**Resource tuning:** Default requests are 100m CPU / 256Mi memory; limits 500m CPU / 512Mi memory. For production with 2.4k+ projects, consider 256Mi–512Mi requests and 512Mi limits.

## Lint / template

```bash
helm lint deploy/helm/cncf-mcp
helm template cncf-mcp deploy/helm/cncf-mcp
```
