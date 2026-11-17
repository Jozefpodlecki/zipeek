# Exercise E2E coverage audit (2026-11-10)

This inventory was checked against the 16 selectable exercise IDs in `crates/zipeek-web/src/components/practice/setup/practice_builder/exercise_selector.rs`, the six `ExerciseVariant` values in `zipeek-core`, and actual Rust E2E test code. It records evidence, not an assertion of successful Chrome execution.

| Exercise | Existing browser coverage in source | Remaining acceptance gap |
| --- | --- | --- |
| Matching (3 variants) | `tests/suites/practice/matching.rs` exercises all three variants, recovery and fixed summaries | Run in managed Chrome |
| Multiple choice (3 variants) | `tests/suites/practice/mcq.rs` and `fixed_mcq.rs` cover all variants | Run in managed Chrome |
| Sentence translation | `tests/translation_fixture.rs` exercises deterministic AI fixture and browser learning flow | Real `zipeek-ai-server` browser integration |
| Speaking | `tests/speaking_interactions.rs` covers configuration and selection | Fake-device capture, stop, browser blob playback, discard/revoke, track cleanup and retry scenarios authored; Chrome run and physical-device verification pending |
| Listening choice | `tests/suites/practice/setup.rs` covers session and restoration | Replay of the same Mandarin phrase now asserted; run in managed Chrome |
| Type pinyin | `tests/practice_answer_options.rs` covers incorrect-answer retry and progression | Run in managed Chrome |
| Character builder | `tests/practice_answer_options.rs` covers automatic answer feedback | Run in managed Chrome |
| Word mosaics | `tests/suites/practice/setup.rs` exercises setup selection | Full eight-pair and wrong-answer scenario authored in `tests/suites/practice/word_mosaics.rs`; additionally verify visible selection, pair-specific red/green feedback, automatic deselection, progressive hints, accessible announcements, and reduced-motion behavior in Chrome against [Word mosaics requirements](../../learning-practice/practice/practice-outcomes.md) |
| Character hunt | `tests/exercise_interactions.rs` and setup scenarios cover hint and restoration | Run in managed Chrome |
| Fill in the blank | `tests/suites/practice/setup.rs` exercises answer, feedback, reload and continue | Run in managed Chrome |
| Audio dictation | `tests/suites/practice/setup.rs` checks play gating, reload, answer, feedback and continue | Run in managed Chrome; add audio failure/retry fixture |
| Sentence builder | `tests/suites/practice/setup.rs` checks answer, feedback and continue | Run in managed Chrome |
| Sentence transformation | `tests/suites/practice/setup.rs` exercises answer gating, reload, feedback and continue | Run in managed Chrome |
| Spot the mistake | `tests/suites/practice/setup.rs` checks repair, feedback and continue | Run in managed Chrome |
| Handwriting recall | `tests/suites/practice/setup.rs` checks reveal, canvas and continue | Run in managed Chrome |
| Image vocabulary | `tests/exercise_interactions.rs` checks selection modes, generation via both AI/image fixture servers, approval, answer grading and regeneration | Deterministic AI scene composition, image-server loading, approval, answer grading and regeneration authored; run in managed Chrome |

`tests/practice_coverage_pending.rs` retains legacy ignored placeholders until the newly authored browser journeys pass on Windows. The non-ignored Image Vocabulary and fake-microphone scenarios are implemented in `exercise_interactions.rs` and `speaking_interactions.rs`. No current-source Chrome results have been supplied for them; authoring a test does not prove successful audio capture or generated-image completion.

## AI server

`crates/zipeek-ai-server/src/http_tests.rs` contains six `#[tokio::test]` HTTP-level integration cases that start an in-process Axum router with a deterministic `FixtureBackend`. They cover model listing, model ID validation, JSON completion, SSE delta/DONE framing, untrusted browser origin rejection and permitted localhost preflight. These are meaningful HTTP integration tests, not proof that the packaged server executable runs with an actual model. `crates/zipeek-e2e/tests/translation_fixture.rs` tests a separate browser translation fixture; `tests/ai_learner.rs` is an ignored external LM Studio scenario, not an AI-server scenario.

`tests/ai_server_live_pending.rs` contains two ignored, deliberately unimplemented cases: running the real AI-server process and routing browser translation through it. Implement with a controlled model fixture and explicit opt-in environment rather than assuming a model is available in CI.

## Additional source-only acceptance findings

The Word Mosaics browser scenario verifies wrong-pair feedback, disabling matched pairs, eight-pair progress, and completion through the standalone route. The Speaking browser suite adds a deterministic microphone-permission denial with retry and target change. Both are authored but have not been executed in Chrome. Picture Vocabulary now prevents approval until its image loads and rejects invalid vocabulary source parameters. Its full browser completion scenario still requires an AI and image-server fixture.

The native AI-server executable supports an explicitly selected loopback-only fixture mode for real-process lifecycle and HTTP streaming tests. It is not a production model runtime; model-backed integration and a browser-to-server test remain pending.
