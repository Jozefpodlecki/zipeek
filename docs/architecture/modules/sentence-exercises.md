# Sentence translation: current contract and historical evolution

This document includes historical implementation checkpoints below. The current
source of truth is the **R62 bilingual generation/evaluation contract** and the
**R68 shared completed-result ledger**, not the old R54/R60 descriptions.

## R67 — Visible instance projection and complete local reset

The web-side `services/translation_instance.rs` now owns the word-bank shuffle
and the projection of generated sentences into learner-visible exercise
instances. It validates tile IDs as a permutation and excludes hidden solution
order and the reference target sentence. Both Chinese→English and
English→Chinese have native privacy/invalid-order tests. The page retains the
network and Yew state machine, but no longer constructs these payloads inline.

Word Set, model, endpoint and direction changes invalidate pending requests and
reset the tile selection, displayed bank ordering, local session and mixed-mode
exercise counter **together**. Mixed mode still chooses its random initial
direction on mount; generation failures do not advance the counter. No new
proficiency claims or persistence were introduced.

## R62 — Explicit translation direction and bilingual word banks

Translation direction is a property of the concrete exercise instance, **not** a choice delegated to the AI model and not an additional global exercise-kind enum. The pure `zipeek-exercises::translation` domain supplies `TranslationDirection` and `DirectionSelection`. The standalone page defaults to **Mixed**: a random first direction is selected once on mount and valid generated exercises alternate directions; manual choices are available. This is a balanced local sequence, not the `zipeek-round-v1` practice RNG. Failed or stale requests do not advance it.

The bilingual `Sentence` response includes `direction`, `chinese`, `english`, `target_words`, `word_bank`, `solution`. The application checks that the direction matches the request, the target words occur in Chinese, and that `solution` contains exactly the bank's tile multiset and reconstructs the target language sentence. Chinese target tiles are concatenated without spaces, English tiles with spaces. Both source and target texts are available to the evaluator, but only the source text and shuffled tiles appear before feedback. The evaluator explicitly receives direction and compares meaning rather than exact tile order.

The page, tile component and prompts now handle either direction. `sentence-generation-v3-bidirectional-word-bank` and `translation-evaluation-v3-bidirectional` are new **sentence-only** prompt versions; they do not modify `db-gen`'s sense-classification-v3 prompt, cache, schema or the ongoing database generation. The previous sentence-generation JSON contract is intentionally superseded in the experimental feature.

Outstanding: live LM Studio acceptance in both directions, native/WASM compilation, meaningful Chinese segmentation fixtures, browser E2E for mixed alternation and drag/drop, and connection of accepted translation verdicts to shared quiz sessions and evidence. Word-bank assembly does not imply independent Chinese writing or pronunciation ability.

## R58 — Deterministic tile operations

`zipeek-core/src/sentence_tiles.rs` now owns the pure ordered tile-ID
transitions for a word-bank answer: append, remove, move before another
selected tile, reject duplicate/out-of-range IDs, check complete selection and
construct the submitted text in either target language. Yew owns drag/drop event plumbing, model request
state and rendering; `zipeek-ai` owns model JSON and prompt contracts. The domain
module is directly unit-testable on native Rust without Yew or LM Studio.

Repeated identical tile strings are distinct tiles by integer ID. A complete
answer uses each ID exactly once; a partial/duplicate/out-of-range selection
cannot be submitted even if the UI button-state check were bypassed. Disabling
the word bank also disables drag/drop actions, not only its buttons. This is a
narrow responsibility, not a reason to move unrelated UI or AI logic to core.

# Sentence exercises and AI roles — R54 foundation

## Responsibilities

`zipeek-ai` owns versioned task-specific prompts, machine-readable response contracts and validation. `crates/web` owns interaction, visual word bank, selection order and model request lifecycle. `crates/e2e` remains the deterministic oracle for application behaviour. The model never decides whether an E2E assertion passed.

| Role | Prompt | Input | Validated output |
| --- | --- | --- | --- |
| Sentence author | `sentence-generation-v3-bidirectional-word-bank` | Named set, direction and allowed Chinese words | `direction`, `chinese`, `english`, `target_words`, `word_bank`, `solution` |
| Translation evaluator | `translation-evaluation-v3-bidirectional` | Direction, bilingual reference and assembled learner answer | `accurate`, `feedback`, `suggested_translation` |
| Simulated learner | `quiz-simulator-v1` | Chinese sentence and uniquely indexed displayed tiles | `selected_tile_ids`, `reason` |
| Sense classifier | `sense-classification-v3` | One source-backed Chinese sense | Controlled `pos`, `categories` |

The generator's `word_bank` and `solution` are tile-text arrays with equal multiplicities: each bank token is used exactly once. Repeated identical tokens represent distinct tiles with separate integer IDs. Model output is rejected for missing or extra tokens, invalid set membership, missing targets, empty strings, overly long text or unknown JSON fields. The evaluator receives the direction, source and reference translation, and the learner's assembled target-language text; it assesses meaning rather than checking the hidden tile order. As an experimental model-authored exercise, the source solution is not a verified gold translation and must not act as the sole grading oracle.

The browser applies a fresh Fisher–Yates shuffle using `Math.random()` on each accepted exercise. This is *not* part of canonical practice round generation and does not alter the `zipeek-round-v1` seed/replay contract. Buttons have stable tile IDs. Clicking adds/removes a tile; native HTML drag/drop moves tiles into/out of the answer and reorders selected tiles. The answer lane explicitly marks START and END, with distinct visual treatment. Clicking is the keyboard-accessible fallback; browser drag/drop compatibility and pointer/touch accessibility still need Chrome/manual validation and enhancement.

## Exercise taxonomy and future work

`zipeek-exercises/src/definition.rs` now defines the canonical conceptual contracts for Matching, Multiple Choice, Sentence Translation, Listening Choice and Speaking. Matching/MCQ/Listening Choice results belong to deterministic game logic; translation requires an evaluator, while speaking requires a future pronunciation evaluator. Definitions are keyed by stable IDs rather than a central enum; listening and speaking are not implemented yet.

An AI-driven E2E agent should be explicitly opt-in and separate from ordinary CI. It receives only the public question and displayed tile IDs, returns a proposed sequence, and the deterministic harness drives the UI and verifies observable states. Required follow-ups include fixed fixtures and seed control, sandboxed model configuration, response-budget caps, privacy review, failure traces, stable `data-component` and `data-*` locators, and human-reviewed expected outcomes. Do not feed the hidden answer into the simulated-learner prompt.

## Runtime and verification

- `cargo test -p zipeek-ai --locked` validates JSON shape, tile multiplicities and simulated-learner IDs.
- `node --test scripts/*.test.cjs` checks stable UI structure and lifecycle wiring.
- `cargo check --target wasm32-unknown-unknown -p zipeek --locked` verifies Yew/HTML drag event typing.
- Manual Chrome test covers mixed click+drag, repeated tiles, touch/keyboard fallback, submission and next sentence.
- Live model test must check actual `sentence-generation-v3-bidirectional-word-bank` JSON from LM Studio and reliable evaluation. The db-gen AI smoke only verifies *classification* and is not sufficient.

## R60 — Definitions and instances

The canonical metadata for exercise families lives in `zipeek-exercises::definition`. This supersedes the conceptual enum in `zipeek-ai::exercise_types`, which now re-exports shared definition contracts. A definition describes *what a learner sees and must produce*; an instance carries one concrete visible prompt, not the grading answer. `zipeek-ai::prompts::exercise_simulation` prepares definition-aware learner requests without hidden solutions. The legacy sentence-specific `quiz-simulator-v1` path remains for the experimental translation page until the generic simulator can be integrated and tested.

`zipeek-exercises::session` owns a pure sequence of exercised instances, completed results and lifecycle. Matching/MCQ bridge into it and Sentence Translation records page-local session
results after validated evaluation. These are not yet one combined cross-route
session, nor persisted proficiency evidence. The deterministic web reducers remain the scoring authority, not the model. For Matching, left/right identifiers are local to each side: exposing stable *shared* word IDs would disclose matches and is forbidden.

## R63 — feedback acknowledgement

The translation page now delegates feedback rendering to
`components/translation_feedback.rs`. Answer submission enters `Evaluating`, and
only a returned evaluation transitions to `Feedback`. The feedback surface is
visually green for an accurate translation and red for an inaccurate answer;
it always shows model feedback, and on failure presents the source exercise's
reference target sentence (not merely the model's suggestion). The only next-step
action is Continue, available after evaluation. Incorrect responses do not trap
the learner or silently retry. The page now records a page-local `QuizSession` result after validated
evaluation (R64); it does not persist learning-evidence events. Model evaluation is not assumed
infallible; reference translations are not a complete set of valid answers.

## R68 — Shared session result ledger

`QuizSessionProvider` now lives above the router. Sentence Translation keeps an
unfinished instance only within its page and submits the instance and verified
evaluation result together to the app-scoped session. This prevents navigation
away from a pending model response blocking later exercises. Changing the model,
Word Set, or translation direction invalidates pending work but **does not** reset
completed results from other exercise routes. The old page-local stats panel
now reads the shared session. Session history remains memory-only; AI evaluation
is not an objective linguistic ground truth.

## R69 — Transactional result registration

Completed translation results enter the global `QuizSession` via one atomic
`record_completed` call, only after the AI evaluator has returned a valid
verdict. A rejected score cannot leave a pending exercise in the shared ledger.
This is not an exactly-once event-delivery guarantee; the provider currently
reassigns receipt-time sequence IDs. Source-event identity is still open.
