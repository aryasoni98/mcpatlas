---
name: cncf-pre-commit
description: Run CNCF MCP quality gates before commit: cargo fmt, clippy, tests, and conventional commit. Use when committing, saving changes, or before pushing in the CNCF MCP repo.
---

# CNCF MCP Pre-Commit

Run quality gates and produce a conventional commit for this Rust workspace.

## Workflow

1. Check `git status` and `git diff --stat`. Identify files to stage.
2. Run quality gates (do not skip unless user explicitly requests):
   - `cargo fmt --all`
   - `cargo clippy --workspace`
   - `cargo test --workspace`
3. Scan staged changes for: `unwrap()`, `dbg!`, `println!`, hardcoded secrets, TODO without ticket.
4. Stage specific files (never `git add -A` or `git add .`).
5. Draft commit message: `<type>(<scope>): <summary>` with optional body. Types: feat, fix, refactor, docs, chore, ci, perf, test, style. Scope: crate or area (e.g. core, data, search, graph, cli).

## Commands

```bash
cargo fmt --all
cargo clippy --workspace
cargo test --workspace
git add <path> ...
git commit -m "<type>(<scope>): <summary>"
```

## Guardrails

- Summary ≤ 72 characters. Body explains why, not what.
- No generic messages ("fix bug", "update").
- If clippy or tests fail, fix or document before committing; do not commit broken state.
