# Homebrew Formula

This directory contains the Homebrew formula for MCPAtlas.

## Using the formula

1. **Install from local formula (after a release):**
   ```bash
   brew install ./deploy/homebrew/mcp-atlas.rb
   ```
2. **Tap (when available):**  
   If we publish a tap (e.g. `mcp-atlas/tap`), you will be able to run:
   ```bash
   brew tap mcp-atlas/tap
   brew install mcp-atlas
   ```

## Automation

The formula is **automatically updated** when a new release is created:

1. Push to `main` with a version bump in `Cargo.toml` → `tag-on-main` creates a tag (e.g. `v0.1.0`)
2. Tag push triggers the **Release** workflow → builds binaries, creates GitHub Release
3. **Update Homebrew formula** job runs after the release → downloads tarballs, computes SHA256, updates `mcp-atlas.rb`, and pushes to `main`

No manual `url`/`sha256` updates are required for new releases.

## Manual update (if needed)

- Replace `v0.1.0` and `0.1.0` in the formula with the new version.
- Run `shasum -a 256 <tarball>` on each platform tarball and set the `sha256` values in the formula.
