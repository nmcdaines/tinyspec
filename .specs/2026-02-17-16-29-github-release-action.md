---
tinySpec: v0
title: Github Release Action
applications:
  -
---

# Background

In order to release changes to this repository we currently manually trigger `cargo package & cargo publish` commands on a developers' machine. This isn't ideal as it's a manual process.

# Proposal

Implement two GitHub Actions workflows and a local pre-commit hook:

1. **CI workflow** — Triggered on pull requests to `main`. Runs `cargo test`, `cargo clippy`, and `cargo fmt --check` to catch issues early.

1. **Release workflow** — Triggered on pushes to `main`. Compares the `Cargo.toml` version against the latest git tag. If the version is newer:
   
   - Runs `cargo test`
   - Creates a git tag (e.g., `v0.1.0`)
   - Builds cross-platform binaries for 4 targets: macOS ARM64, macOS x86_64, Linux x86_64, and Windows x86_64
   - Creates a GitHub Release with auto-generated notes and binary artifacts attached
   - Publishes to crates.io using `CARGO_REGISTRY_TOKEN`
1. **Pre-commit hook** — A local git pre-commit hook that runs `cargo fmt` (auto-fix), `cargo clippy --fix --allow-dirty` (auto-fix), and `cargo test` before each commit. Catches issues locally before they reach CI.

# Implementation Plan

- [x] A: Create CI workflow for pull requests
  - [x] A.1: Create `.github/workflows/ci.yml` triggered on PRs to `main`
  - [x] A.2: Add steps for `cargo fmt --check`, `cargo clippy`, and `cargo test`
- [x] B: Create release workflow for main branch
  - [x] B.1: Create `.github/workflows/release.yml` triggered on push to `main`
  - [x] B.2: Add version check step — extract version from `Cargo.toml` and compare to latest git tag; skip remaining steps if not incremented
  - [x] B.3: Add `cargo test` step as a gate before publishing
  - [x] B.4: Create git tag matching the `Cargo.toml` version (e.g., `v0.1.0`)
  - [x] B.5: Build cross-platform binaries using a matrix strategy (macOS ARM64, macOS x86_64, Linux x86_64, Windows x86_64)
  - [x] B.6: Create GitHub Release with auto-generated notes and attach binary artifacts
  - [x] B.7: Publish to crates.io using `CARGO_REGISTRY_TOKEN` secret
- [x] C: Add pre-commit hook
  - [x] C.1: Create a pre-commit hook script that runs `cargo fmt`, `cargo clippy --fix --allow-dirty`, and `cargo test`
  - [x] C.2: Re-stage any files modified by fmt/clippy fixes before allowing the commit
  - [x] C.3: Document hook setup in README (or provide an install script)

# Test Plan
