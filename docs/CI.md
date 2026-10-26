# CI and deployment

Local commands and prerequisites live in [TESTING](TESTING.md). This document
owns workflow structure, gates, caching, and Pages deployment.

## Current workflows

- Both workflows run on pushes to `master` or manual dispatch; neither has a
  pull-request trigger. Both select Rust `1.97.0`, below the `1.97.1` policy and
  crate minimum. Local `rust-toolchain.toml` selects `1.97.1`.
- `.github/workflows/ci.yml` runs `cargo test` on Windows. It caches Cargo
  binaries, Git sources, registry data, and target artifacts with a lockfile key.
  With e2e now in the workspace, browser prerequisites must also be accounted for.
- `.github/workflows/deploy.yml` builds on Ubuntu, installs wasm-bindgen CLI
  `0.2.114`, uses Trunk `latest` and Node 22, and invokes npm, stylesheet, and
  Trunk commands at obsolete root paths. It omits `--public-url /zipeek/` and
  uploads `./dist/` rather than the web build output.
- Root `Cargo.lock` is ignored; `web/Cargo.lock` exists but is not the workspace
  lockfile. The current cache key can therefore miss workspace resolution changes.

## Required follow-up (A16)

Use the approved toolchain and `web/` working directory for npm/Trunk. Run all
five [AGENTS gates](../AGENTS.md#6-commands-and-completion), and Chrome smoke
coverage with the approved A15 lifecycle harness. Preserve the separate root
base path for development e2e and `/zipeek/` for Pages; upload `web/dist/`.
Add PR validation and account for Chrome, managed-driver availability, and
Trunk readiness. Workflow/dependency edits and any new tool pins require their
own approved implementation scope; this reconciliation changes no workflow.
