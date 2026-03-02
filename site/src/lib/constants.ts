const basePath = import.meta.env.VITE_BASE_PATH ?? "";
export const DOCS_URL = `${basePath}docs`;
/** Full user guide (mdBook), deployed at /book when built via Pages workflow. */
export const BOOK_URL = `${basePath}book`;
export const GITHUB_URL = "https://github.com/cncf-mcp/server";
export const CONTRIBUTE_URL = `${GITHUB_URL}/blob/main/CONTRIBUTING.md`;

export interface NavLink {
  label: string;
  href: string;
  external?: boolean;
}

export const NAV_LINKS: NavLink[] = [
  { label: "Home", href: basePath || "/" },
  { label: "Docs", href: DOCS_URL },
  { label: "Book", href: BOOK_URL },
  { label: "Roadmap", href: `${basePath || "/"}#roadmap` },
  { label: "GitHub", href: GITHUB_URL, external: true },
  { label: "Contribute", href: CONTRIBUTE_URL, external: true },
];

export interface FeatureItem {
  title: string;
  description: string;
  /** Optional mono label (tool names, transport). */
  mono?: string;
}

/** System fact strip for Problem section (spec: one data line). */
export const SYSTEM_FACT_STRIP =
  "2,400+ projects · 11 categories · 4 maturity levels";

export const FEATURES: FeatureItem[] = [
  {
    title: "14 MCP tools",
    description:
      "search_projects, get_project, compare_projects, suggest_stack, get_relationships, get_migration_path, and more.",
    mono: "search_projects, get_project, compare_projects, ...",
  },
  {
    title: "Resources & prompts",
    description:
      "cncf://landscape/overview, cncf://projects/{name}; evaluate_tool, plan_migration, review_stack, onboard_contributor.",
    mono: "cncf:// · 4 prompts",
  },
  {
    title: "STDIO, SSE, HTTP",
    description:
      "Content-Length framing, SSE, and Streamable HTTP (MCP 2025-03-26) with session support.",
    mono: "STDIO | SSE | Streamable HTTP",
  },
  {
    title: "Knowledge graph",
    description:
      "Auto-inferred relationships plus curated CNCF edges; get_relationships, find_path.",
    mono: "get_relationships, find_path",
  },
  {
    title: "Hybrid search",
    description:
      "Full-text (Tantivy) and optional vector (Qdrant) with RRF merge.",
    mono: "Tantivy + Qdrant",
  },
  {
    title: "2,400+ projects",
    description:
      "CNCF Landscape data with maturity, categories, GitHub enrichment, optional Artifact Hub.",
    mono: "landscape.yml → index + graph",
  },
];

export interface UseCaseItem {
  title: string;
  description: string;
  /** Tools used (spec: tool-driven use cases). */
  tools: string;
}

/** Stack list for Solution section (spec: counts and protocol names). */
export const STACK_LIST =
  "14 tools · 4 prompts · cncf:// resources · STDIO · SSE · Streamable HTTP";

export const USE_CASES: UseCaseItem[] = [
  {
    title: "Choose a service mesh",
    description: "Compare Istio, Linkerd, Cilium.",
    tools: "suggest_stack, compare_projects",
  },
  {
    title: "Compare Kubernetes distros",
    description: "k3s, k0s, OpenShift, EKS.",
    tools: "get_project, compare_projects",
  },
  {
    title: "Find good first issues",
    description: "Filter by language or category.",
    tools: "get_good_first_issues",
  },
  {
    title: "Plan a migration",
    description: "Migration guide between two projects.",
    tools: "get_migration_path, plan_migration",
  },
];

export interface RoadmapPhase {
  name: string;
  goal: string;
  done: string[];
  upcoming: string[];
}

export const ROADMAP_PHASES: RoadmapPhase[] = [
  {
    name: "Phase 1: Foundation",
    goal: "Installable, usable MCP server with core data and search.",
    done: ["Core MCP server (STDIO, SSE, Streamable HTTP)", "14 tools, resources, prompts", "Docker, mdBook, CI", "Homebrew template"],
    upcoming: [],
  },
  {
    name: "Phase 2: Intelligence",
    goal: "Semantic search, health scoring, richer data, CNCF Sandbox.",
    done: ["Storage traits (Graph, Vector, Cache)", "SurrealDB, Qdrant, hybrid search", "Artifact Hub, LLM summaries"],
    upcoming: ["Apply for CNCF Sandbox"],
  },
  {
    name: "Phase 3: Extensibility",
    goal: "Plugin ecosystem, operator, contributor growth.",
    done: ["Dynamic plugin tools", "get_good_first_issues, get_migration_path", "PULL_REQUEST_TEMPLATE, CODEOWNERS"],
    upcoming: ["WASM plugin host wiring", "Built-in plugins", "Kubernetes operator"],
  },
  {
    name: "Phase 4: Scale & maturity",
    goal: "Production-grade, horizontal scaling, security hardening.",
    done: ["Helm chart", "Audit logging", "Pod securityContext"],
    upcoming: ["Redis shared state", "RBAC", "SLSA Level 3, cosign", "Persistent Tantivy index"],
  },
];

/** Success metrics targets (from ROADMAP.md). */
export interface SuccessMetricRow {
  metric: string;
  "3mo": string;
  "6mo": string;
  "12mo": string;
}

export const SUCCESS_METRICS: SuccessMetricRow[] = [
  { metric: "GitHub stars", "3mo": "500", "6mo": "2,000", "12mo": "10,000" },
  { metric: "Monthly downloads", "3mo": "1K", "6mo": "10K", "12mo": "100K" },
  { metric: "Contributors", "3mo": "5", "6mo": "20", "12mo": "50+" },
  { metric: "MCP tools", "3mo": "12", "6mo": "14+", "12mo": "20+" },
  { metric: "Plugins", "3mo": "0", "6mo": "3", "12mo": "15+" },
  { metric: "CNCF status", "3mo": "", "6mo": "Sandbox", "12mo": "Incubating prep" },
];
