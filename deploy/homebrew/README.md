# Homebrew Formula

This directory contains the Homebrew formula for CNCF MCP Server.

## Using the formula

1. **After a release:** Download the tarball for your platform from [Releases](https://github.com/cncf-mcp/server/releases) (e.g. `cncf-mcp-0.1.0-aarch64-apple-darwin.tar.gz`).
2. **Install from local formula:**  
   Update the `url` and `sha256` in `cncf-mcp.rb` to match the release, then:
   ```bash
   brew install ./deploy/homebrew/cncf-mcp.rb
   ```
3. **Tap (when available):**  
   If we publish a tap (e.g. `cncf-mcp/tap`), you will be able to run:
   ```bash
   brew tap cncf-mcp/tap
   brew install cncf-mcp
   ```

## Updating the formula for a new release

- Replace `v0.1.0` and `0.1.0` in the formula with the new version.
- Run `shasum -a 256 <tarball>` on each platform tarball and set the `sha256` values in the formula.
