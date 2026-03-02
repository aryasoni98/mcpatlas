# Contributing

We welcome contributions. This page summarizes how to build, test, and submit changes.

## Build and test

```bash
git clone https://github.com/cncf-mcp/server.git
cd server
cargo build
cargo test --workspace
```

## Quality gates

1. **Format:** `cargo fmt --all`
2. **Lint:** `cargo clippy --workspace --all-targets`
3. **Tests:** `cargo test --workspace`
4. No `unwrap()` in non-test code; use `?` or `.context()`

## Pull requests

- Open a PR with a clear description and type (feat, fix, docs, etc.).
- Ensure CI passes (format, clippy, tests).
- See [CONTRIBUTING.md](https://github.com/cncf-mcp/server/blob/main/CONTRIBUTING.md) in the repo for full details, conventional commits, and code of conduct.

## Code of conduct

We follow the [CNCF Code of Conduct](https://github.com/cncf/foundation/blob/main/code-of-conduct.md).
