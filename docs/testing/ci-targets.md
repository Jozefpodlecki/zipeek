# CI and target coverage

Current CI state, required gates, and the PR checklist. Part of the
[testing index](TESTING.md). Workflow structure and deployment live in
[CI](../process/CI.md); lockfile policy in
[D27](../process/decisions/process-and-infra.md#d27-workspace-lockfile).

## Current CI and target coverage

- `ci.yml` currently runs `cargo test` on Windows using Rust `1.97.0`, triggered
  by pushes to `master` or manual dispatch.
- `deploy.yml` builds on Ubuntu using root-level npm/style/Trunk paths from the
  previous layout and no explicit `/zipeek/` public URL.
- Full fmt, Clippy, workspace, wasm, and Chrome smoke coverage is intended, but
  the exact CI matrix is not implemented. PR-triggered validation also needs work.

Track these gaps in [TASKS](../process/TASKS.md) and [CURRENT](../process/CURRENT.md).
Editing workflows or fixing application build failures requires a separate
approved implementation plan.

## Required gates

For code changes, all five gates must pass:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --locked -- -D warnings`
3. `cargo test --workspace --locked`
4. `cargo check --target wasm32-unknown-unknown -p zipeek --locked`
5. `trunk build --public-url /zipeek/` (from `web/`)

The wasm check does not replace the Pages build; both are required. UI behaviour
changes also require Chrome e2e smoke coverage once the e2e harness exists. If
any gate is blocked by a missing prerequisite, report it honestly — never mark
it as passed.

For documentation-only changes: Markdown structure, relative links, command
accuracy, and consistency with repository facts. Rust builds and e2e are not
required.

## PR checklist

- Scope and any dependency/schema/decision changes were explicitly approved.
- Applicable checks passed, or failures and missing prerequisites are reported.
- UI changes include Chrome smoke results when the harness is available.
- Pages builds retain `/zipeek/`.
- CURRENT/TASKS, completed outcomes, and affected architecture/product/DB/testing
  documents are updated.
- The PR describes resulting behaviour and validation, with relevant failure output.

## A16 follow-up (open)

A16 owns CI and Pages deployment migration. Required:

- Use the approved Rust `1.97.1` toolchain; current workflows select `1.97.0`.
- Use the `web/` working directory for npm/Trunk.
- Run all five gates and Chrome smoke coverage with the A15 lifecycle harness.
- Preserve the separate root base path for development e2e and `/zipeek/` for
  Pages; upload `web/dist/`.
- Add PR validation and account for Chrome, managed-driver availability, and
  Trunk readiness.
- Preserve locked npm assets (`npm ci --ignore-scripts`) and the focused
  Pages/browser compatibility check added in A06.
- Review the pre-existing Picomatch tooling advisory
  (`target/a06-npm-audit.json`) before any separately approved dependency fix.

Workflow/dependency edits and any new tool pins require their own approved
implementation scope; this reconciliation changes no workflow.