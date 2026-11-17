# Browser E2E testing

Zipeek uses the Rust `zipeek-e2e` crate and Thirtyfour with managed browser drivers. Scenario source and page objects are under `crates/zipeek-e2e/`.

- [Architecture](e2e-architecture.md) covers ownership of fixtures, browsers, tests and cleanup.
- [Recording procedure](e2e-recording.md) explains how to collect browser evidence.
- [Recording and evidence contract](e2e-video-recording.md) describes MP4 capture, step timelines, diagnostics and strict validation.
- [Requirements coverage](e2e-requirements-matrix.md) distinguishes authored tests from verified execution.
- [Exercise coverage](../acceptance/exercise-e2e-coverage.md) records exercise-specific scenarios.

An authored test is not automatically a passing test. A completed assertion is not evidence that an MP4 was captured, and a saved video is not proof that it can be played. Scenario manifests report these outcomes separately.

Earlier dated E2E engineering notes are retained in [historical E2E reference](../../archive-history/audits/e2e.md).

For developer and Codex evidence interpretation, follow the [review workflow](review-workflow.md).
