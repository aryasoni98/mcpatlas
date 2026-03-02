---
name: devops
description: CI/CD, Docker, Helm, Kubernetes, and observability for CNCF MCP. Use when editing workflows, Dockerfile, Helm charts, or adding metrics/traces.
---

# DevOps — CNCF MCP

Apply BluePrint §3–4 (phases, Helm, scaling) and DEEP_PLAN §10.

## CI (GitHub Actions)

- **On PR/push**: Lint (rustfmt, clippy), test (cargo test --workspace), optional bench. Fail fast.
- **Release workflow**: Trigger on tag or manual. Build multi-arch (x86_64/aarch64, linux/darwin). Build and push container (multi-arch). Sign (cosign). Generate and attach SBOM. Create GitHub Release with assets. Optional: update Homebrew tap.
- **Scheduled**: Data sync (e.g. landscape refresh) as cron or workflow_dispatch; use cache and secrets correctly.
- Use OIDC where possible; avoid long-lived secrets. Pin actions by full SHA.

## Docker

- Multi-stage: build stage (cargo build --release), minimal runtime stage (e.g. Chainguard or distroless). Copy only binary and needed assets.
- No secrets in image. Env vars for config; document required (GITHUB_TOKEN, cache dir, backend URLs).
- Labels: org.opencontainers.* for version, source, revision. Expose port (e.g. 3000 for SSE).
- docker-compose for local/team: cncf-mcp, optional Redis, optional Qdrant/Meilisearch; reverse proxy if needed.

## Helm

- Chart under deploy/helm/cncf-mcp. Templates: deployment, service, ingress, configmap, secret (template only), HPA, PDB, serviceaccount. Values: replicaCount, image, transport, port, rateLimit, cache backend/ttl, redis url, autoscaling min/max/target.
- Default values work for Tier 2 (single namespace, optional Redis). Document overrides for Tier 3 (external DBs, secrets from vault).
- Do not store secrets in values; use existingSecret or external-secrets.

## Kubernetes and operator

- Deployment: resource requests/limits (CPU, memory). Liveness/readiness probes on health endpoint. PodDisruptionBudget for HA.
- CronJob for data pipeline: same image, different command/args; use cache and secrets. Backoff and history limits.
- Operator (future): CRD CncfMcpServer; reconcile Deployment, Service, Ingress, HPA, CronJob, plugin config; status subresource. Go + controller-runtime; separate repo or deploy/operator.

## Observability

- **Metrics**: Prometheus /metrics endpoint. Counters for requests, tool calls, errors; gauges for in-flight, cache size; histograms for latency.
- **Logs**: Structured (e.g. JSON); tracing fields (trace_id, span_id) when OTLP enabled. Levels via RUST_LOG or logging/setLevel (wire to tracing filter).
- **Traces**: OpenTelemetry for tool calls and outbound HTTP; sample in production to limit volume.
- **Health**: HTTP /health or equivalent for readiness; include dependency checks (cache, optional DB) when applicable.

When changing CI or deploy: ensure local dev path (cargo run, --skip-github) still works and release path is reproducible.
