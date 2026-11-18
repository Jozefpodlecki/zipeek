# E2E requirements-to-test matrix

This is a source-inspection inventory, not a run report. **No test below is marked successfully executed** until real Chrome artifacts are reviewed. "Existing test" means test source contains a related journey, not that it validates the final approved behavior.

| Approved behavior | Related browser test source | Source coverage | Runtime status | Missing evidence or assertion |
| --- | --- | --- | --- | --- |
| Word mosaics selection and match feedback | `crates/zipeek-e2e/tests/suites/practice/word_mosaics.rs` | Partial, legacy feedback and completion with new named checkpoints | Not executed here | Selected-state semantics, red/green states, progressive hints, accessible feedback |
| Sentence Relay token-level corrections | `crates/zipeek-e2e/tests/ai_game_journeys.rs` | Partial, older generic feedback | Not executed here | Misplaced token hint, drag/reorder, valid alternatives and punctuation |
| Multiple Choice variants | `crates/zipeek-e2e/tests/suites/practice/mcq.rs`, `fixed_mcq.rs` | Partial | Not executed here | Pinyin defaults, audio autoplay/replay, format-specific answer-leak protection |
| Practice persistence and replay | `crates/zipeek-e2e/tests/suites/practice/practice.rs` | Conflicting: seed/offset URL replay assertions | Not executed here | Current persisted session and JSON profile portability; retire URL replay contract |
| Practice scoring distribution | `crates/zipeek-e2e/tests/authored_session_completion.rs` | Partial | Not executed here | Correct, partially correct, incorrect distribution; separate skips; evidence |
| Character builder contextual retry | `crates/zipeek-e2e/tests/practice_answer_options.rs` | Partial | Not executed here | Position-level correction, progressive hints, assistance evidence |
| Write Hanzi stroke stability | `crates/zipeek-e2e/tests/exercise_interactions.rs` | Partial | Not executed here | Interruption, switching characters, resize and replay visual regression |
| Shared exercise layout | `crates/zipeek-e2e/tests/suites/practice/setup.rs` | Partial | Not executed here | No title or redundant exit, typed outcome contract and responsive consistency |
| Vocabulary exclusions and nonblocking count | `crates/zipeek-e2e/tests/suites/practice/setup.rs` | Partial | Not executed here | Global exact-entry exclusion, progress, cancellation, stale request suppression |
| Saved profiles versioned JSON | Practice setup journeys | Missing explicit import/export acceptance | Not executed here | Validation, privacy, duplicate names and absent Word Sets |
| Long evidence-based dragon header | Header and Stats browser journeys | Partial, old header presentation | Not executed here | Dragon-shaped fill, evidence source, detail panel, responsive fallback |
| Original-tab ownership | App shell browser journeys | Partial | Not executed here | Full-screen duplicate blocking and recovery after owner exits |
| AI Hanzi recognition | `crates/zipeek-e2e/tests/hanzi_recognition.rs`, `scripts/tests/recognition-workspace.test.cjs` | Authored explicit submission, fixture response, dictionary verification, Clear, offline draw/undo behavior; paste/upload Chrome evidence still missing | Not executed in Chrome | Paste/upload permissions, actual vision capability, cancellation at transport, Word Set interaction and `/zipeek/` |
| Docs navigation across 12 physical sections | `crates/zipeek-e2e/tests/docs_reader.rs`, `crates/zipeek-e2e/tests/route_matrix.rs` | Authored article samples across 12 sections, deep links, search and issue image decoding; static route references validated | Not executed here | Run Chrome against `/` and `/zipeek/`, inspect recorded checkpoints and verify route behavior |
| MP4 and failure evidence | `crates/zipeek-e2e/src/harness/recorder.rs`, `artifacts.rs` | Infrastructure implemented and updated | Not executed here | Actual playable Chrome MP4 and complete scenario artifacts |

The owning functional acceptance criteria are in [approved behavior acceptance](../acceptance/approved-behavior-acceptance.md) and the existing [exercise coverage audit](../acceptance/exercise-e2e-coverage.md). A new E2E journey must fail when an approved behavior is absent, rather than silently weakening an assertion to fit current implementation. Obsolete assertions should be retired alongside their corresponding implementation updates, not treated as proof of a superseded product requirement.

## Documentation browser regression coverage

The Docs E2E suite now uses the actual `project-management/tasks.md` heading instead of a deleted development task route. It includes separate scenarios for all twelve section samples, issue screenshot decoding, and search-to-article navigation. The Node documentation validator checks sampled source routes and heading targets. Rust unit tests also check the current lowercase filename ordering. These are authored checks, not passing Chrome or Cargo results.

## Artifact integrity tooling

`node scripts/e2e-evidence-audit.cjs <run-directory> --strict-evidence --decode-video --write-report` independently checks the manifests and referenced evidence after a run. `scripts/tests/e2e-evidence-audit.test.cjs` exercises valid, missing, inconsistent and unsafe artifact cases. This is an offline integrity check, not Chrome acceptance or an assertion that the user interface implements the approved behavior. Rust E2E checkpoint errors are now written into the scenario manifest and Codex feedback, with an optional strict evidence failure gate.
