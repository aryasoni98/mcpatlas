# Homebrew formula for MCPAtlas (Blueprint §3 Phase 1).
# Copy to your tap or: brew install ./deploy/homebrew/mcp-atlas.rb
# Update url and sha256 when cutting a new release (see deploy/homebrew/README.md).
class McpAtlas < Formula
  desc "MCP server for the CNCF Landscape"
  homepage "https://github.com/aryasoni98/MCPAtlas"
  license "Apache-2.0"

  on_macos do
    on_intel do
      url "https://github.com/aryasoni98/MCPAtlas/releases/download/v0.1.0/mcp-atlas-0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
    on_arm do
      url "https://github.com/aryasoni98/MCPAtlas/releases/download/v0.1.0/mcp-atlas-0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_ARM"
    end
  end

  def install
    bin.install "mcp-atlas"
    bin.install "mcp-atlas-cli" if File.exist?("mcp-atlas-cli")
  end

  test do
    assert_match "mcp-atlas", shell_output("#{bin}/mcp-atlas --help", 1)
  end
end
