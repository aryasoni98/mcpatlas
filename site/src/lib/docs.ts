const modules = import.meta.glob<string>("../content/docs/*.md", {
  query: "?raw",
  import: "default",
});

const base = "../content/docs/";
const allSlugs = Object.keys(modules)
  .filter((p) => p.startsWith(base) && p.endsWith(".md"))
  .map((p) => p.slice(base.length, -3));
export const docSlugs = [...new Set(allSlugs)].sort();

const slugToKey: Record<string, string> = {};
Object.keys(modules).forEach((key) => {
  if (key.startsWith(base) && key.endsWith(".md")) {
    const slug = key.slice(base.length, -3);
    slugToKey[slug] = key;
  }
});

export async function getDocContent(slug: string): Promise<string | null> {
  const key = slugToKey[slug];
  if (!key) return null;
  const loader = modules[key];
  if (typeof loader !== "function") return null;
  return (await loader()) as string;
}

export const DOC_TITLES: Record<string, string> = {
  introduction: "Introduction",
  "getting-started": "Getting started",
  "how-to-use": "How to use",
  "use-cases": "Use cases & setup",
  "project-structure": "Project structure",
  configuration: "Configuration",
  "tools-reference": "Tools reference",
  "resources-reference": "Resources reference",
  prompts: "Prompts",
  architecture: "Architecture",
  deployment: "Deployment",
  workflow: "Workflow (CI/CD & release)",
  "plugin-development": "Plugin development",
  "vercel-web-analytics": "Vercel Web Analytics",
  roadmap: "Roadmap",
  contributing: "Contributing",
};

export type DocNavItem = { slug: string; label: string; anchor?: string };

export type DocNavGroup = { title: string; items: DocNavItem[] };

/** Sidebar nav in display order with groups. Anchors are for in-page sections (e.g. introduction). */
export const DOC_NAV: DocNavGroup[] = [
  {
    title: "Introduction",
    items: [
      { slug: "introduction", label: "What it does", anchor: "what-it-does" },
      { slug: "introduction", label: "Who it is for", anchor: "who-it-is-for" },
      { slug: "introduction", label: "Documentation", anchor: "documentation" },
    ],
  },
  {
    title: "Documentation",
    items: [
      { slug: "getting-started", label: "Getting started" },
      { slug: "how-to-use", label: "How to use" },
      { slug: "use-cases", label: "Use cases & setup" },
      { slug: "project-structure", label: "Project structure" },
      { slug: "configuration", label: "Configuration" },
      { slug: "tools-reference", label: "Tools reference" },
      { slug: "resources-reference", label: "Resources reference" },
      { slug: "prompts", label: "Prompts" },
    ],
  },
  {
    title: "Architecture & operations",
    items: [
      { slug: "architecture", label: "Architecture" },
      { slug: "deployment", label: "Deployment" },
      { slug: "workflow", label: "Workflow (CI/CD & release)" },
      { slug: "vercel-web-analytics", label: "Vercel Web Analytics" },
    ],
  },
  {
    title: "Extending",
    items: [{ slug: "plugin-development", label: "Plugin development" }],
  },
  {
    title: "Roadmap & community",
    items: [
      { slug: "roadmap", label: "Roadmap" },
      { slug: "contributing", label: "Contributing" },
    ],
  },
];
