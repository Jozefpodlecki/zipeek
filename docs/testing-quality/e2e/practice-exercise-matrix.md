# Practice exercise browser coverage inventory -

The authoritative registry is `crates/zipeek-exercises/src/definition.rs` (`DEFINITIONS`).
These entries describe **observed test scope**, not a claim that every behavior is passing in Chrome.
An exercise appearing in a setup selector or route test does not establish a completed answer,
recovery, score, persistence, or accessibility contract.

| Exercise | Browser evidence located | Missing depth |
| --- | --- | --- |
| Matching | Dedicated matching scenarios, round navigation | UUID history persistence and cross-tab duplicate launch |
| Multiple Choice | Setup and learner journey references | Dedicated answer, correction, resume and completion journey |
| Type Pinyin | Wrong-answer retry, settings, resume | Completed answer and history journey |
| Character Builder | Tile interaction, preferences, refresh, resume | Completion and history journey |
| Word Mosaics | Setup selection | Tile answer, finish, restore |
| Sentence Translation | Deterministic translation fixture | All directions, UUID history error/retry |
| Listening Choice | Route/interactions, prerequisites | Answer, recover, completion |
| Image Vocabulary | Selection interaction | Full scored result and recovery |
| Speaking | Dedicated standalone interactions | Audio lifecycle under UUID host and unscored evidence |
| Character Hunt | Hint/interactions and learner journey | Complete rounds, restore, history |
| Fill Blank | Setup selection | Answer, feedback, resume and finish |
| Audio Dictation | No exercise-specific browser scenario located | Full journey |
| Sentence Builder | Setup selection | Tile correctness, recovery, completion |
| Sentence Transformation | No exercise-specific browser scenario located | Grammar alternatives, feedback, recovery |
| Spot Mistake | Setup selection | Detection + correction journey |
| Handwriting Recall | Setup selection | Draw/reveal, restore, unscored completion |

`practice_answer_options.rs` now additionally tests that an existing UUID Practice session
blocks setup and remains resumable. This does not establish a cross-tab atomicity guarantee:
`localStorage` provides synchronous individual operations, not a compare-and-swap transaction.

## Required release coverage

For each type, test a representative launch, incorrect/correct or unscored action,
completion, and any applicable refresh/recovery. Test browser recording opt-in/out and
failure retry separately. Generated AI types need deterministic fixture cases; a live
model remains an optional integration run. Keep route smoke checks as a separate layer.

## Practice options acceptance (pending Chrome)

The new Type Pinyin preference exposes the expected Pinyin as a placeholder when guided hints are enabled. The hint is not evidence of independent recall. Character Builder offers optional automatic validation when the bank fills, including red incorrect-feedback and reset when retries are enabled. The browser suite now exercises configuration independence, the configured hint on launch and refresh, automatic answer submission, and same-tab restoration of `/practice` after the reminder discards a session. These additions are unverified until the Windows Chrome run completes.

The persistent `PracticeExerciseOptions` model contains configurable options for Character Builder and Type Pinyin only. It is not a statement that every option of all sixteen exercise types is snapshotted there. Other configuration fields (for example Matching variants, translation direction, and Character Hunt difficulty) are stored in the session's step metadata or vocabulary map where implemented. The remaining per-type option completeness audit is open.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/testing-quality-e2e-practice-exercise-matrix.md). Dated implementation claims in that snapshot are not current build or browser verification.
