# Current engineering handoff

This file is an entry point, not a rolling checkpoint transcript. Dated implementation and test history is retained in [completed](../archive-history/completed/), [release history](../archive-history/RELEASE_HISTORY.md), and the root [release notes](../../RELEASE_NOTES.md). Do not paste old baseline counts, retired paths or old dependency pins into a current release.

## Read first

1. [Contributor workflow and project constraints](../../AGENTS.md).
2. [Current implementation and confirmed test evidence](../project-management/current.md).
3. [Open tasks and release gates](../project-management/tasks.md).
4. [Source reconciliation and unresolved source/policy mismatches](../archive-history/audits/SOURCE_RECONCILIATION.md).
5. [Architecture](../architecture/architecture.md), [Practice](../learning-practice/practice/practice-session-builder.md), [database](../dictionary-data/schema/db.md), and [E2E](../archive-history/audits/e2e.md).

## Native CLI runtime on Windows

`zipeek-cli` uses the pinned native DuckDB library. Cargo Make tasks invoking the CLI must depend on `duckdb-env`, including `pack`, `unpack`, and database artifact tasks. This task validates `duckdb.lib` and `duckdb.dll` and prepends the DLL directory to `PATH` for the CLI process. A Windows exit code of `-1073741515` (`0xC0000135`) indicates a DLL dependency could not be loaded before the CLI could run. Do not work around it by spawning Cargo from the CLI launcher or by copying arbitrary DLL versions into the project.

Archive pack/unpack use an existing `target/release/zipeek-cli.exe`. Build it explicitly with `cargo make release-cli-build` when missing or outdated; `cargo make unpack` itself does not build or run a nested Cargo process.

## Current handoff constraints

- Treat the lockfiles and installed asset manifests as authoritative for configured dependency versions. Dependency changes require explicit user approval.
- Check dictionary generation and installation independently from the source archive. Do not infer a valid runtime database from a manifest entry alone.
- Verify personal learning history durability and single-tab ownership before treating recorded evidence as authoritative. Recoverable unfinished sessions must not be discarded on persistence failures.
- Confirm authored lesson and exercise paths against current source, then run Rust/WASM and Chrome acceptance before claiming their browser functionality.
- Distinguish older user-reported test runs from current source acceptance. Use [current status](../project-management/current.md), [open tasks](../project-management/tasks.md) and [E2E review](../testing-quality/e2e/review-workflow.md) for verification.
- Keep architecture proposals and implementation boundaries in the task tracker until source inspection and approval establish their disposition.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../archive-history/audits/maintained-source-snapshots/build-deployment-handoff.md). Dated implementation claims in that snapshot are not current build or browser verification.
