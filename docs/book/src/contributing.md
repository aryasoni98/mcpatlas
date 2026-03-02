# Contributing

We welcome contributions. See the repository root [CONTRIBUTING.md](https://github.com/cncf-mcp/server/blob/main/CONTRIBUTING.md) for:

- Development setup (Rust, build, test)
- Architecture overview (crates and responsibilities)
- Quality gates (format, clippy, tests)
- Pull request checklist and conventional commits
- RFC process for architectural changes ([GOVERNANCE.md](https://github.com/cncf-mcp/server/blob/main/GOVERNANCE.md))

Quick start:

```bash
git clone https://github.com/cncf-mcp/server.git
cd server
cargo build
cargo test --workspace
```

For significant changes (new storage backends, plugin ABI, transport behavior), open an RFC in `docs/rfcs/` and follow the process in the repo GOVERNANCE.md.
