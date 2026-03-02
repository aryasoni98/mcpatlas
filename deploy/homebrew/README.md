# Homebrew Formula

This directory contains the Homebrew formula for CNCF MCP Server.

## Using the formula

1. **After a release:** Download the tarball for your platform from [Releases](https://github.com/mcp-atlas/server/releases) (e.g. `mcp-atlas-0.1.0-aarch64-apple-darwin.tar.gz`).
2. **Install from local formula:**  
   Update the `url` and `sha256` in `mcp-atlas.rb` to match the release, then:
   ```bash
   brew install ./deploy/homebrew/mcp-atlas.rb
   ```
3. **Tap (when available):**  
   If we publish a tap (e.g. `mcp-atlas/tap`), you will be able to run:
   ```bash
   brew tap mcp-atlas/tap
   brew install mcp-atlas
   ```

## Updating the formula for a new release

- Replace `v0.1.0` and `0.1.0` in the formula with the new version.
- Run `shasum -a 256 <tarball>` on each platform tarball and set the `sha256` values in the formula.
