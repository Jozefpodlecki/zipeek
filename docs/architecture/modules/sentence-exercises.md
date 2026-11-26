# Sentence exercise architecture

## Ownership

`zipeek-exercises` owns pure exercise definitions, draft validation and session-domain types. `zipeek-ai` constructs structured prompts, validates bilingual generation and evaluation responses, and handles seeded vocabulary selection and canonical sentence tiles. `zipeek-ai-client` is the HTTP transport. The Yew application orchestrates the configured provider, browser dictionary, rendered question and temporary Practice persistence.

For every supported Practice run, `crates/zipeek-web/src/services/practice_launch.rs` creates a UUID-backed `PracticeSession` in `practice_session_store/mod.rs`. The serialized record includes a typed configuration registry (including normalized translation direction), seeded scheduled steps, ISO start/completion timestamps, generated questions and scored outcomes. `practice_generated.rs` generates questions respecting that direction; `pages/practice/local_session.rs` renders, advances, restores and completes them. `pages/practice/generated_session.rs` collects tile selections, calls AI evaluation, and reports scoring to the session host.

The generated sentence model is **Chinese + English only**. The model does not provide pinyin, solution indices or tiles. Zipeek derives canonical tiles and validates reconstruction, including duplicate-word handling. The browser exposes `data-solution-order` and first/last flags for deterministic interaction testing. This is not a language-quality oracle or a guarantee of context-perfect segmentation.

The former `#/practice/translation` and other dedicated Practice exercise URLs are intentionally removed. The public `#/translate-sentence` route remains a separate free-text utility rather than a scored Practice session.

## Persistence and limitations

In-progress UUID Practice records live under `zipeek.practice.session.v1.*` in browser localStorage. They are temporary and support resumption. Finishing a run removes its record, while the results screen explains that there is no durable learning-history writer. The app's private DuckDB/OPFS work is not yet an authoritative outcome store. AI evaluation can be inaccurate and does not independently certify a translation.

## Tests

- Pure/unit tests near `zipeek-ai` generation and `crates/zipeek-web/src/services/practice_session_store/mod.rs` cover sampling, persisted direction, outcomes and Endless handling.
- `crates/zipeek-e2e/src/harness/ai_fixture.rs` is the deterministic Rust API fixture.
- `crates/zipeek-e2e/tests/translation_fixture.rs` tests generation, tile assembly, progression and interruption; `ai_learner.rs` is optional and requires LM Studio or a compatible provider.

Implementation detail and acceptance status: [current state](../../project-management/current.md), [E2E](../../archive-history/audits/e2e.md).


## Practice generated translation: state ownership and R608 boundary

The generated Practice translation path owns answer ordering and evaluation in `pages/practice/generated_session/translation.rs`; image-vocabulary loading and answer state live in its sibling `image.rs`. These are different independently mounted components. The current source does **not** implement the prospective R608 audio-only prompt and free-text response policy; see [Knowledge Points](../../learning-practice/pedagogy/knowledge-points.md). An AI evaluation presently returns sentence-level accuracy, feedback and suggested translation, not validated per-entity/per-dimension evidence. Dismissing a pending UI result is not a transport cancellation. Historical extraction notes are in [release engineering history](../../archive-history/audits/release-engineering-review-history.md).
