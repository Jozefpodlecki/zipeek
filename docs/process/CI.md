# CI and deployment

Local commands and prerequisites live in
[testing/TESTING.md](../testing/TESTING.md). This document owns workflow
structure, gates, caching, and Pages deployment.

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
- D27/A19 tracks the existing root `Cargo.lock`; the obsolete web-local lockfile
  is removed. Cargo validation and Trunk builds use locked resolution. A16 should
  use the root lockfile for cache keys and preserve locked validation in CI.

## Required follow-up (A16)

Use the approved toolchain and `web/` working directory for npm/Trunk. Run all
five [AGENTS gates](../../AGENTS.md#6-commands-and-completion), and Chrome smoke
coverage with the approved A15 lifecycle harness. Preserve the separate root
base path for development e2e and `/zipeek/` for Pages; upload `web/dist/`.
Add PR validation and account for Chrome, managed-driver availability, and
Trunk readiness. Workflow/dependency edits and any new tool pins require their
own approved implementation scope; this reconciliation changes no workflow.

A19 verified local Rust/Cargo 1.97.1, installed wasm target, Trunk 0.22.0,
Tailwind 4.3.3 and managed wasm-bindgen 0.2.114 configuration. No exact Trunk pin
is selected by this evidence. The npm manifest/lock agree on Tailwind 4.3.3,
but at that audit node_modules was absent; Trunk's managed Tailwind CLI built
without npm install.
The obsolete npm/style commands in deploy.yml still require A16 migration.

A06 subsequently installs the exact DuckDB-WASM npm dependency. Trunk now requires
`npm ci --ignore-scripts` from web/ before its local asset-packaging hook can run.
A16 must install those locked assets, preserve `/zipeek/` and include the focused
Pages/browser compatibility check; no workflow was changed in A06.