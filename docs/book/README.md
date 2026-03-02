# MCPAtlas — mdBook

This directory contains the [mdBook](https://rust-lang.github.io/mdBook/) source for the project documentation (Blueprint §3 Phase 1). When the site is deployed via GitHub Pages, the book is available at **`/book`** (e.g. `https://<org>.github.io/<repo>/book/`).

## Build

From the repository root:

```bash
mdbook build docs/book
```

Output is written to `docs/book/build/`.

## Serve (live reload)

```bash
mdbook serve docs/book
```

Then open http://localhost:3000.

## Structure

- `book.toml` — mdBook configuration.
- `SUMMARY.md` — Table of contents (root; copy in `src/` for some mdBook versions).
- `src/` — Chapter Markdown files (Introduction, Getting Started, Configuration, Tools Reference, etc.).
