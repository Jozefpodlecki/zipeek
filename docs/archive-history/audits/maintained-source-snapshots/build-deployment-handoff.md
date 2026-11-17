# Current engineering handoff

This file is an entry point, not a rolling checkpoint transcript. Dated implementation and test history is retained in [completed](../../completed), [release history](../../RELEASE_HISTORY.md), and the root [release notes](../../../../RELEASE_NOTES.md). Do not paste old baseline counts, retired paths or old dependency pins into a current release.

## Read first

1. [Contributor workflow and project constraints](../../../../AGENTS.md).
2. [Current implementation and confirmed test evidence](../../../project-management/current.md).
3. [Open tasks and release gates](../../../project-management/tasks.md).
4. [Source reconciliation and unresolved source/policy mismatches](../SOURCE_RECONCILIATION.md).
5. [Architecture](../../../architecture/architecture.md), [Practice](../../../learning-practice/practice/practice-session-builder.md), [database](../../../dictionary-data/schema/db.md), and [E2E](../e2e.md).

## Current handoff constraints

- The approved DuckDB-WASM dependency is `1.33.1-dev114.0` in the installed npm package and D08 policy; do not change it without new approval.
- Generator SQL, the artifact writer and the installed manifest declare `character_readings`, but the physical DuckDB binary is absent from this source-only archive. Its table inventory still requires independent verification after artifact regeneration/installation.
- The R283 source includes an opt-in personal DuckDB Practice-outcome writer, idempotent commit/readback, a failure-retry path, and a validated on-demand summary of completed sessions, scored outcomes and distinct **UTC completion dates**. None is Rust/WASM- or browser-verified in this checkpoint. UTC completion dates are not equivalent to learner-active dates, and no proficiency evaluation is implemented.
- Temporary completed Practice state is deleted only after successful opted-in history commit, or when recording is disabled. On persistence failure it remains recoverable. Do not describe the stored evidence as authoritative before browser durability, cross-tab and export/import acceptance.
- An authored Sentence Transformation Practice-only source implementation now has three grammar patterns, distractors, curated accepted answers and an interactive Yew host; it is not compiled or browser-accepted, and does not implement Courses assessments.
- Translation-only Advanced Practice launches a UUID-backed session; the latest Rust/WASM build and Chrome E2E acceptance of that exact integration are still pending.
- Earlier user-reported Windows passes cover 31 harness unit tests, the accessibility route matrix and six Browse scenarios, plus a **prior** standalone live AI translation scenario. Do not present these as acceptance of the current persisted learner journey.
- Unresolved architecture proposals are tracked in `docs/project-management/tasks.md`; do not implement them without their required review and approval.

## Commands

```powershell
cargo fmt --all -- --check
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
cargo test --release --locked -p zipeek-e2e --test translation_fixture -- --nocapture
cargo make e2e-full
```

The source-only archive contains no generated dictionary binary. The latest source-level tests do not substitute for the required Rust/WASM and browser acceptance checks. Read `CURRENT.md` before reporting acceptance status.
