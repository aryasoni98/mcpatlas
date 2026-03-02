#!/usr/bin/env bash
# Release verification script for CNCF MCP site (landing + in-app docs).
# Run from repo root. See docs/RELEASE_EXECUTION_REPORT.md §10.
set -e

echo "=== 1. Install site ==="
cd site && npm ci && cd ..

echo "=== 2. Build site ==="
cd site && npm run build && cd ..

echo "=== 3. Serve (optional). Run: npx serve site/dist -s -l 3000 ==="
echo "    Then open http://localhost:3000 (landing) and http://localhost:3000/docs (in-app docs)."
echo "=== 4. Lighthouse: open DevTools → Lighthouse, run report, target 95+ all categories. ==="
echo "=== 5. Verify Docs link in nav points to /docs. ==="
echo "=== 6. Verify 404: request /nonexistent; with -s flag serve returns index.html for SPA. ==="
echo ""
echo "Verification build complete. Site built in site/dist/."
