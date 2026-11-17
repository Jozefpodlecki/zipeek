# Archived source snapshot: learning-practice/practice/practice-session-builder.md

This source snapshot preserves historical descriptions, not current requirements. Use the maintained document at its original location for approved behavior.



> **Current approved target behavior:** [Practice outcomes](../../../learning-practice/practice/practice-outcomes.md), [vocabulary selection](../../../dictionary-data/vocabulary/vocabulary-selection.md), and [profile portability](../../../learning-practice/practice/practice-profile-portability.md). Older implementation descriptions below record prior contracts and may not match the latest approved behavior or verified source.

## R448 setup and session interaction contract

- Practice setup begins without preselected exercise types, even in Quick mode. Continue is disabled until an eligible exercise is selected. Selected tiles use a distinct theme-aware background and visible Selected label; previous wizard steps show filled circles rather than checkmarks.
- The Choose exercise grid has a viewport-bounded vertical scroll region while the Back/Continue toolbar and Choose/Configure tabs remain accessible. Selected and unselected states must remain distinguishable in all six themes and by keyboard focus.
- Matching configuration selects 2, 3, 4, 5, 6 or 7 pairs per round. The choice is stored once under the session's shared Matching quiz configuration. Individual steps reference that configuration; the MCQ round count remains four. Invalid counts and insufficient distinct vocabulary must fail closed.
- `/practice/session/:id` is a distraction-free workspace without the global header, navigation drawer or application footer. A lightweight Leave session link returns to setup while preserving the local session at its last committed step boundary.
- Matching must visibly distinguish the selected first-column tile, show incorrect-pair feedback, and dim matched pairs. Hanzi use the configured Chinese font and readable typography. Animations must respect reduced-motion preferences.
- None of these presentation changes creates verified mastery or changes learning-history recording consent.

# Practice setup and execution

## Learner workflow

The four-step setup is **Path -> Exercises -> Vocabulary -> Summary**. Advanced Practice offers exercise choices and exercise-specific configuration. The selected source may be HSK filters or a saved Word Set, subject to exercise requirements. A session begins only when its chosen configuration passes draft validation.

Quick Practice and existing standalone exercise pages retain their supported behavior. Not every combination of exercises is backed by the same persistence runtime. Do not interpret the shared setup UI as proof of a fully unified cross-exercise engine.

## Planned Word Set eligibility rule

**Not implemented in current Practice setup.** A saved Word Set must have **at least ten distinct valid Hanzi-bearing vocabulary entries** before it is selectable as a Practice source. Sets below ten remain creatable/editable, but their Practice option is disabled with a visible, accessible count and reason (for example, `7 of 10 available; add 3`). Duplicates, deleted/stale IDs and unplayable entries do not count. Setup must revalidate at launch time; exercise-specific playability checks still apply. This is an entry-count requirement, not a promise that all ten entries will occur in one session or become proficient. See [learning evidence](../../../learning-practice/practice/learning-evidence.md) and [E2E plan](../../../testing-quality/e2e/learning-evidence-e2e.md).

## Sentence Translation

The **translation-only** Advanced Practice path creates a persisted UUID Practice record, not a URL-configured standalone page. Settings owns AI endpoint/model configuration. Setup owns vocabulary, direction and question count. Choices are Chinese to English, English to Chinese or Mixed and five, ten or Endless questions.

Finite runs persist the selected number of steps. Endless starts with a small schedule and extends as the learner advances, until explicitly finished. Saved generated instances and scored outcomes survive route changes/reload. Finishing stages the final local revision. The R283 checkpoint attempts a durable commit before cleanup when recording is enabled, and preserves the temporary record for retry if persistence fails. With recording disabled, cleanup occurs without a history write. Browser verification is pending.

## Word-bank Sentence Transformation (source checkpoint)

Advanced Practice now includes a separate **Sentence Transformation** option (distinct from Sentence Builder) in the source. Its UUID-backed fixed-length host uses three manually authored grammar exercises for 是……的, 把, and 比. Each prompt shows the original Chinese sentence and requested target pattern, shuffles selectable chunks, and offers undo, reset, and per-tile removal/reordering, with keyboard-accessible buttons. Some tiles are explicit distractors. The checker accepts only curated approved variants rather than claiming to understand arbitrary Chinese; the 比 example accepts two valid variations. Learners receive grammar-specific feedback and a reference answer. This activity is **practice-only** and is not proficiency or permanent course assessment evidence.

This is an independently selectable Practice MVP, not the full Courses mosaic, grammar syllabus, point progression or assessment release. Browser input and Chinese linguistic review are still required; neither Rust nor WASM compilation was available at this checkpoint.

## R495 Sentence Transformation configuration

The Advanced Practice Sentence Transformation Configure tab offers a searchable grammar pattern picker with browser autocomplete and three answer methods: word/phrase tiles, typed Chinese, or Both (switch per question before checking). The picker is deliberately limited to the three currently authored and assessed practice items: 是……的, 把, and 比. An unrecognized custom entry cannot introduce an unreviewed transformation question; an explicit warning appears and the previously selected pattern remains unchanged until a valid suggestion or All patterns is chosen. All patterns draws from the existing authored bank, so longer sessions may repeat items. This is not an autocomplete across the 550 draft Grammar teaching entries.

Configuration is saved in the Practice profile options and copied into the session's typed `SentenceTransformation` configuration. Session validation rejects unsupported pattern indices. Each question uses its selected pattern or draws from the authored bank by seed. Typed answers match curated accepted sentence strings after ignoring whitespace and terminal sentence punctuation; arbitrary grammatical paraphrases are not automatically assessed. Both mode offers learner choice between tiles and typing for the same question, not two mandatory submissions. A pure transformation reducer validates every input transition, locks submitted responses, and a keyed Yew exercise child resets draft UI state on question identity changes without changing durable session progress. Distractors are surplus word tiles in the shuffled bank that do not occur in the accepted sequence. The activity remains practice-only, not mastery evidence.

## Other exercise hosts

Matching and MCQ use their existing scored-round logic. Type Pinyin, Character Builder, Word Mosaics, Character Hunt and other specialized exercises have dedicated components and selection rules; do not claim every heterogeneous selection can be resumed through the translation-only UUID host. Read the actual `PracticeDraft` validation and `practice_launch.rs` for allowed combinations.

## R516 completion-screen presentation

The finite local Practice completion view presents a compact trophy heading, a percentage-based accessible progress bar, completed/skipped counters and clear Practice again / Explore vocabulary navigation. A perfect finish uses the "Excellent work!" heading; a session with skipped exercises uses "Session finished!" and accurately reports the partial result. Status messaging follows the existing learning-history lifecycle: pending finalization, recorded, not recorded, or recoverable saving failure with Retry. No progress, mastery or retention metric is inferred from the celebration UI. The summary controls and history semantics remain unchanged; the progress bar uses a 0-100 accessible value, with visible counts retained in text. The responsive summary fits smaller viewports and honors reduced-motion preferences.

## Current UX and unfinished work

The learner should not be asked to configure the AI URL/model inside a question, nor to manually discard a completed record. Unfinished-work reminders should offer clear Resume and End actions. Privacy notices must not suggest a non-existent pedagogical-data toggle. Persisted pedagogical history, storage failure recovery and cross-mode lifecycle parity are future tasks, listed in [TASKS](../../../project-management/tasks.md).

## Source and acceptance

- Setup: `crates/zipeek-web/src/pages/practice/setup/` and `crates/zipeek-web/src/components/practice/setup/`.
- Draft rules: `crates/zipeek-exercises/src/practice_draft/`.
- Session host: `crates/zipeek-web/src/pages/practice/local_session.rs`.
- Launch/persistence: `crates/zipeek-web/src/services/practice_launch.rs` and `practice_session_store.rs`.
- Browser acceptance: `crates/zipeek-e2e/tests/translation_fixture.rs`, `practice_answer_options.rs` and other targets in `crates/zipeek-e2e/tests/`.

See [E2E](../e2e.md) for executable commands and [CURRENT](../../../project-management/current.md) for validation evidence.

## R449 configuration component boundary

The Practice Setup page owns one `PracticeSetupFlow` reducer. The Exercises panel receives that existing reducer handle and is responsible for Choose/Configure layout only. A small settings host selects a focused configuration component for Matching/MCQ, vocabulary answer preferences, Character Hunt or Translation. Each component receives focused configuration values and one typed action callback; the host dispatches those actions to the existing reducer; the panel has no field-specific callback properties. Unconfigured exercise types retain their current explanatory surfaces, not invented controls.

Configuration tab identities come from `PracticeDraft::configuration_tabs()` and are validated against the currently selected definitions before reducer updates. The active tab is a stable static identifier rather than an arbitrary allocated UI string. The Choose/Configure stage is represented by `ExerciseStage`. Existing exercised options and persisted session configurations remain unchanged. This structure does not add an exercise registry or claim full coverage for hypothetical future exercise types.

## Service-dependent exercise selection (approved 2026-11-17)

AI-dependent exercises must become disabled when the required service cannot be reached or lacks the needed capability, with a reason and Settings action. Selecting a model does not prove connectivity. Offline-authored exercises remain independently selectable only when their own contract does not call AI. This behavior is a pending implementation and acceptance requirement, not a property of the current tiles.
