# Testing and validation

> **Current approved target behavior:** [Approved behavior acceptance](approved-behavior-acceptance.md). Older implementation descriptions below record prior contracts and may not match the latest approved behavior or verified source.


Rust unit/integration tests and real Chrome runs answer different questions. JavaScript source-contract tests under `scripts/` were retired by decision in the historical source. A test appearing in the repository is not proof that it has run successfully. Current acceptance state is maintained in [CURRENT](../../project-management/current.md); open gates are in [TASKS](../../project-management/tasks.md). Historical evidence belongs in [release history](../../archive-history/RELEASE_HISTORY.md).

## Commands from the workspace root

```powershell
cargo fmt --all -- --check
cargo test --workspace --locked
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

Run affected Rust tests first for routine work. For comprehensive release acceptance use:

```powershell
cargo make e2e-full
cargo make pack-e2e
```

`e2e-full` first checks WASM, compiles all release E2E targets, then runs the managed Rust/Chrome suites serially. The pack task prepares evidence for review. Browser suites use a managed server, an isolated Chrome profile and scenario-specific artifacts; see [E2E](../../archive-history/audits/e2e.md) and [E2E architecture](../e2e/e2e-architecture.md).

## Current browser coverage

- Shared catalog: 34 concrete route cases under root and the `/zipeek/` deployment base, plus separate accessibility and 320/390/768px responsive checks.
- Behavioral suites: Browse/Word Sets, Settings/Debug, Practice setup and persistence, courses, games, speaking, translation, writing, shell navigation and errors. See test targets under `crates/zipeek-e2e/tests/`.
- Sentence Translation: deterministic Rust AI fixture and optional live provider journey through Settings, Practice setup and the UUID-backed generated exercise. Fixture tests cover canonical tiles, finite completion, restore and Endless behavior.
- AI fixture: `crates/zipeek-e2e/src/harness/ai_fixture.rs`; the old Node `ai-server.cjs` is removed.

**Acceptance limitation:** The user reported successful Windows accessibility, Browse and an earlier live translation test. The newest UUID translation implementation and full suite still need their own execution. An ignored test is not a pass; route presence is not equivalent to complete interaction or WCAG compliance.

## Live AI test

Provide a running OpenAI-compatible endpoint and exact model ID, then run:

```powershell
$env:ZIPEEK_E2E_AI_ENDPOINT = 'http://127.0.0.1:1234/v1/'
$env:ZIPEEK_E2E_AI_MODEL = 'your-loaded-model-id'
cargo test --release --locked -p zipeek-e2e --test ai_learner -- --ignored --nocapture
```

The live test is intentionally excluded from deterministic CI. It sends actual requests to the locally configured provider; it is not a general assertion of model correctness.

## Related focused guides

[Browser database](../e2e/browser-db.md), [matching](../e2e/matching.md), [MCQ](../e2e/mcq.md), [startup](../e2e/startup.md), [shell](../e2e/shell.md), [E2E recording](../e2e/e2e-recording.md), [evidence](../e2e/evidence.md), [UI acceptance](ui-acceptance.md).

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/testing-quality-acceptance-testing.md). Dated implementation claims in that snapshot are not current build or browser verification.
