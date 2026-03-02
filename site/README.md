# CNCF MCP  Landing site

Production landing page and **in-app docs** for the CNCF MCP server. Built with Vite, React, TypeScript, Tailwind CSS, and Framer Motion. Docs live at `/docs` (markdown in `src/content/docs/`). Deploy to GitHub Pages or Vercel.

## Prerequisites

- Node.js 20+
- npm

## Commands

```bash
# Install
npm ci

# Development
npm run dev

# Production build
npm run build

# Preview production build locally
npm run preview
```

For local preview with SPA fallback (e.g. for Lighthouse): `npx serve dist -s -l 3000`. From repo root, `./scripts/verify-release.sh` runs install and build for release verification.

## Deployment

### GitHub Pages

The repo uses GitHub Actions (`.github/workflows/pages.yml`): build the Vite site and upload `site/dist/` as the Pages artifact. Landing at `/`, docs at `/docs`. Enable in Settings → Pages → Source: GitHub Actions → **Pages** workflow.

### Vercel

A `vercel.json` in this directory configures the build for Vercel:

- **Build:** `npm run build`
- **Output:** `dist`
- **Framework:** Vite (auto-detected)
- **Rewrites:** All routes → `/index.html` (SPA)

Connect the repo in Vercel and set the **Root Directory** to `site`. Vercel will use `vercel.json` and deploy the built app. No extra config needed.

## Base path

For user/org Pages (`owner.github.io`), use default `base: '/'`. For project Pages (`owner.github.io/repo`), set at build time:

```bash
VITE_BASE_PATH=/repo/ npm run build
```

And set `VITE_DOCS_URL` to `/repo/docs/` if needed. The workflow can set these via `env` in the "Build site" step.

## Structure

- `src/app/App.tsx`  Root; lazy-loads landing sections.
- `src/components/ui/`  Button, Container, Section, Navbar, Footer, Typography, Badge.
- `src/components/landing/`  Hero, Problem, Solution, FeaturesGrid, ArchitectureSection, UseCases, RoadmapSection, CTA.
- `src/lib/constants.ts`  DOCS_URL, GITHUB_URL, NAV_LINKS, FEATURES, USE_CASES, ROADMAP_PHASES.
- `src/content/docs/*.md`  Doc content (introduction, getting-started, configuration, architecture, deployment, roadmap, contributing, tools-reference).
- `src/lib/docs.ts`  Loads doc markdown via `import.meta.glob`; `docSlugs`, `getDocContent`, `DOC_TITLES`.

No secrets; all config via `VITE_*` env or constants.
