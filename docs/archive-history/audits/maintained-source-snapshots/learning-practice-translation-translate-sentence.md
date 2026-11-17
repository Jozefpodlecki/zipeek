# Sentence translation: utility and Practice exercise

Zipeek has **two different translation experiences**. They share Settings-owned AI endpoint and model configuration but do not share scoring semantics.

## Translate a supplied sentence

`#/translate-sentence` accepts text authored by the learner and provides a model-generated translation, including optional analysis where supported. It is a utility, not a scored Practice question. Source input limits, direction controls and cancellation/error handling are owned by the translator page and AI client. Do not count its requests as Practice results.

## Sentence Translation in Practice

Under Practice -> Advanced, select **Sentence Translation** as the only exercise, select a vocabulary source, translation direction (Chinese to English, English to Chinese, or Mixed) and five, ten or Endless questions. The launch creates a UUID-backed record at `#/practice/session/<id>`; it does not require AI model or endpoint fields inside the exercise.

Generated questions use a seeded sample of at most ten distinct eligible source words. The generation prompt requests only a Chinese sentence and an English translation. Zipeek validates the response, constructs the ordered answer and shuffled word bank, persists the generated instance, and exposes stable canonical tile-order attributes for tests. Repeated words are assigned occurrence-aware positions. These attributes are an E2E interaction seam, not proof the sentence is linguistically valid.

The learner assembles the translation, submits it for AI evaluation and sees feedback before advancing. Fixed runs stop at five or ten questions; Endless continues until explicitly finished. Reload and navigation can restore saved unfinished questions. Completion removes the temporary record and explains that results are not durably retained.

## AI and privacy

Settings -> AI stores the endpoint (`ai_url`) and model (`ai_model`) in the consolidated preferences document (`zipeek.preferences.v1`). The optional API key is held in tab memory, not persisted. Requests go from the learner's browser to the chosen provider and require compatible CORS and transport security. Do not place an administrator API secret in a public browser deployment.

The R283 checkpoint introduces an unverified opt-in Practice-outcome writer and recording preference; browser acceptance and portable export are pending. The personal OPFS/DuckDB facilities are not an exercise-outcome store. See [personal data](../../../dictionary-data/personal/personal-data-storage.md) and [technical boundary](../../../architecture/modules/sentence-exercises.md).

## Implementation and tests

- Translation utility: `crates/zipeek-web/src/pages/translation/sentence_translator/`.
- Standalone generated exercise: `crates/zipeek-web/src/pages/translation/sentence_practice/` (still independently routable).
- Persisted Practice: `crates/zipeek-web/src/services/practice_launch.rs`, `practice_generated.rs`, `practice_session_store.rs`, and `pages/practice/local_session.rs`.
- AI config and request client: `crates/zipeek-web/src/services/ai/`, `crates/zipeek-ai/` and `crates/zipeek-ai-client/`.
- Deterministic and optional live Chrome tests: `crates/zipeek-e2e/tests/translation_fixture.rs` and `ai_learner.rs`.

See [current validation](../../../project-management/current.md) before claiming the latest Practice integration is browser-verified.
