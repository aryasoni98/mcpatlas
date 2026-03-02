# Homebrew formula for CNCF MCP Server (Blueprint §3 Phase 1).
# Copy to your tap or: brew install ./deploy/homebrew/cncf-mcp.rb
# Update url and sha256 when cutting a new release (see deploy/homebrew/README.md).
class CncfMcp < Formula
  desc "MCP server for the CNCF Landscape"
  homepage "https://github.com/cncf-mcp/server"
  license "Apache-2.0"

  on_macos do
    on_intel do
      url "https://github.com/cncf-mcp/server/releases/download/v0.1.0/cncf-mcp-0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
    on_arm do
      url "https://github.com/cncf-mcp/server/releases/download/v0.1.0/cncf-mcp-0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_ARM"
    end
  end

  def install
    bin.install "cncf-mcp"
    bin.install "cncf-mcp-cli" if File.exist?("cncf-mcp-cli")
  end

  test do
    assert_match "cncf-mcp", shell_output("#{bin}/cncf-mcp --help", 1)
  end
end
