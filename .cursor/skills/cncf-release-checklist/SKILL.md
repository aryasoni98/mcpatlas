---
name: cncf-release-checklist
description: Execute or verify the CNCF MCP release checklist: multi-arch build, container image, signing, SBOM, Homebrew. Use when cutting a release, publishing binaries, or updating release workflow.
---

# CNCF MCP Release Checklist

Follow BluePrint.md Phase 1 and DEEP_PLAN for release and supply-chain steps.

## Pre-release

- [ ] All tests pass: `cargo test --workspace`
- [ ] Clippy clean: `cargo clippy --workspace`
- [ ] Version bumped in Cargo.toml (and Chart.yaml if applicable)
- [ ] CHANGELOG or release notes updated

## Build and publish

- [ ] **Multi-arch binaries**: x86_64-apple-darwin, aarch64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu. Upload to GitHub Release.
- [ ] **Container**: Build multi-arch image; push to ghcr.io (or configured registry). Use minimal base (e.g. Chainguard).
- [ ] **Signing**: Sign binaries and container with cosign (or project standard).
- [ ] **SBOM**: Generate SPDX SBOM (e.g. syft) and attach to release or image.
- [ ] **Homebrew**: Update formula checksum and version; publish to tap if applicable.

## CI workflow

Release workflow (e.g. `.github/workflows/release.yml`) should:

- Trigger on tag or manual dispatch.
- Build Rust for all four targets.
- Build and push container (multi-arch).
- Run cosign sign and optional attestations.
- Create GitHub Release with assets.
- Optionally update Homebrew tap repo.

## References

- BluePrint.md §3 Phase 1 (Homebrew, release workflow), §8 (Security & Compliance), §11 (90-day roadmap).
- DEEP_PLAN.md §10 (Development Workflow), §12 (Roadmap).
