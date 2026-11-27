# Shared Practice exercise foundation

## Purpose and scope

All active Practice exercise types share a coherent Rust/Yew presentation and lifecycle foundation. The foundation supplies consistent accessibility, instructions, session status, feedback and navigation, while domain-specific components retain their own gameplay logic, input method and evaluation. This is an approved architectural target, not a claim that a shared implementation already exists.

## Component responsibilities

| Logical component | Owner and responsibility |
| --- | --- |
| Practice session host | Session identity, configured slots, durable persistence, scheduling and resumption, global progress and exit behavior |
| Exercise frame | Consistent content hierarchy, accessible instruction, common feedback region and shared action placement |
| Exercise progress | Exercise-local counts (pairs, strokes, rounds) independent of session position |
| Exercise content | Typed task state, interaction and evaluation for the concrete exercise type |
| Outcome/attempt adapter | Typed correctness, hint/assistance evidence, attempt counts and session reporting |
| Shared audio control | Manual playback, user preferences, availability and safe audio lifecycle |

Use composition and typed props/actions rather than a large conditional component that owns every exercise's local fields. Shared UI components may be flexible about geometry: a writing canvas, matching grid, word bank or speech exercise must retain enough space to work.

## Common visible layout

- In-session exercise cards **do not display the exercise type name as their header**. Names remain appropriate in setup, profiles and detailed results.
- Show a short, accessible action instruction with a `yew_icons` icon near the input. Retain task-specific Chinese sentences, vocabulary meanings and local progress labels as actual exercise content, not redundant exercise-type titles. The standalone Speaking route may keep its page heading; in-session Speaking uses the shared microphone instruction instead. Resumable Character Hunt uses the shared search instruction.
- Retain the existing themed `CreatureProgress` animal and animated reveal at the top of the session. The animal remains outside the transitioning exercise subtree and does not get remounted when advancing.
- Transition the exercise workspace between steps with the already-pinned `animato` crate. Respect `prefers-reduced-motion`, keep progression and persistence immediate, and avoid duplicate action invocation during transition.
- Center the active exercise workspace in the available region below animal progress, while allowing taller activities to scroll.
- Display session position, such as `Exercise 2 of 5`, consistently. Distinguish it from exercise-local `3 of 8 pairs matched` or `Stroke 4 of 7`.
- Display instructions close to the activity; show mistake counts only when meaningful and comprehensible.
- A single top-right **Leave Practice** control saves progress for resumption. Do not duplicate an Exit practice button in the card. Ending or discarding a session is a separate explicit action.
- Use shared rules for selected, incorrect, partial, correct, assisted, waiting, completed and disabled states. Each state needs text or shape signals in addition to color.
- Preserve current Zipeek dark-violet visual identity and scenic immersive Practice layout; responsive and reduced-motion modes must remain operable.

## Fixed and open-ended session modes

The session domain now distinguishes `SessionMode::Fixed` from `SessionMode::Endless`; this mode belongs to the session, not to any concrete exercise component. The session host owns advancement and persistent completion. The shared `ExerciseFrame` offers the explicit **Finish session** control for an active Endless session after at least one completed step, with a confirmation before recording completion. **Leave Practice** remains a different, resumable navigation action.

The session now schedules up to three active/future steps and queues AI preparation ahead of the current answer. A single generation job runs at a time; the window is replenished after completion or skip, so the next Sentence Translation can already be ready when the learner continues. This prefetch buffer is a best-effort target, not a guarantee of two ready results under slow or failed AI generation. It is also used by fixed sessions where upcoming generation is required. The existing per-step generation attempt limits and stale-result guards continue to apply.

The established implementation enables Endless only for Sentence Translation. The current schedule generator and validation remain bounded to 1,000 stored steps, so it must **not** be described as an unlimited stream. Broadening Endless to mixed exercise sets requires a scheduling and bounded-storage/history policy that can append deterministically without rebalancing prior rounds, prematurely completing, or growing the browser record without bound. A truly open-ended session needs that separate follow-up before general availability. The existing creature remains anchored above the session; its Endless presentation announces count completed without inventing a finite progress percentage.

Session view snapshots use a Yew reducer. Local confirmation state is a separate reducer. `ExerciseFrame` receives explicit typed props and callbacks rather than using a broad context solely for one child; use session contexts only when multiple descendants genuinely consume a common session service or state.

## Shared exercise surface migration (R704 WIP)

The registered exercise families now compose `ExerciseSurface` (`pages/practice/surface.rs`), a common responsive card with a single icon/action instruction and shared outer spacing. It is used by every currently registered exercise ID: Vocabulary (Type Pinyin, Character Builder, Word Mosaics), authored Fill Blank/Audio Dictation, Sentence Builder/Transformation, Spot Mistake, Local Quiz (Matching/Multiple Choice), Listening Choice, Write Hanzi, generated Translation/Image/Narrative, Speaking and Character Hunt. Individual DOM test markers stay on the exercise's outer wrapper. The standalone Speaking page retains its page heading and only mounts the shared surface in a hosted session.

This is a presentation-shell migration, **not** a new grading or persisted evidence model. Existing `ExerciseFeedback`, `ContinueExercise`, `ExercisePrimaryAction`, form-submit controls, gameplay reducers, progress counters, retry semantics and typed registry callback adapters retain their current responsibilities. The hosting `ExerciseFrame` continues to own session position and Endless Finish, while the separate `CreatureProgress` and `animato` boundary remain outside the exercise surface. Specific grading/feedback components must still be integrated into the richer attempt-evidence lifecycle, with WASM compilation and browser acceptance pending.

## Lifecycle and evidence contracts

The common lifecycle distinguishes preparing, active input, submitted feedback, completed and skipped/replaced. Each type may have additional internal states but must not bypass the host's durability and outcomes contract.

A submitted attempt retains its evidence, even when a later attempt is successful. Outcomes use **Correct**, **Partially correct** and **Incorrect** where semantically valid. Skip and Replace generates a new item of the same type/configuration within the same session slot, records the skipped attempt separately and does not award mastery. Successful completion alone is not sufficient for acquisition evidence.

Feedback should explain the actual error when determinable, avoiding invented grammar or unsupported certainty. Do not produce an immediate full solution where correction and progressively helpful hints are pedagogically appropriate. Keyboard users and screen readers must be able to understand selection, feedback and next actions without color or motion.

## Exercise registration and rendering

Exercise type identifiers remain stable string IDs in scheduled Practice steps. The session host delegates ready-step rendering to an immutable registry of `ExerciseDefinition` trait implementations in `crates/zipeek-web/src/pages/practice/local_session/registry.rs`. Each definition declares one or more supported IDs and renders the appropriate existing Yew component using a session-provided context. Closely related exercise IDs may share one definition. Exercise-family adapters now live in separate files under `local_session/registry/`, while `registry.rs` retains the trait, render context, explicit registration list and unsupported-ID handling. This is a transitional adapter boundary, not yet exercise-owned registration at each gameplay module. The registry rejects duplicate IDs in a unit test. Unsupported identifiers display the existing non-scoring fallback instead of selecting an arbitrary component.

A new exercise adds its own `ExerciseDefinition` and a single entry in the explicit registry. No global `ExerciseKind` enum, dynamic loading, new registration dependency, or central string `match` is required. The registry does not own state, hooks, persistence, the animal or transitions. Yew hooks remain inside the mounted exercise components. The host retains readiness, generation errors, session progress and transition ownership.

The session also mounts a lightweight `ExerciseFrame` that gives active exercises a consistent session-position label and an accessible region without adding a second visual card. Vocabulary exercise name headings and redundant Exit links are hidden when that activity is hosted inside a session, while standalone navigation is retained. These are incremental presentation changes and do not yet establish common feedback or action ownership.

The shared feedback/action component now serves Spot the Mistake, Sentence Transformation, Fill Blank, Audio Dictation, Sentence Builder and Local Quiz (Multiple Choice and Matching). It renders an announced correct, partial or incorrect status, themed container and consistent Continue control while retaining exercise-specific explanation content and test selectors. The verdict is a **presentation state**, not a persisted mastery or learning-outcome record. Authored exercises also present action instructions instead of exercise-type headings. Sentence Builder retains its retry policy and solution/explanation content, and Matching keeps its local pair-progress and disabled Continue policy. Generated Image Vocabulary, generated Sentence Translation, Type Pinyin, Word Mosaics, Character Builder and Listening Choice now also use the shared feedback/Continue presentation; generated image and translation prompts use shared icon instructions. Listening Choice preserves audio preparation, answer visibility and its existing Continue selector. Write Hanzi now uses a shared writing instruction while leaving the canvas, character prompt and per-character completion gating in place. Their grading and game-specific controls remain local. Other exercises still retain specialized feedback and navigation and require incremental migration. The session transition now advances its existing `animato` tween with measured timer intervals rather than assumed fixed 16 ms ticks; the exercise transition region now receives keyboard focus on step swaps, including reduced-motion swaps; interruption and focus behavior still need browser acceptance.

This registry change is a first migration stage, **not** evidence that the shared `ExerciseFrame`, typed outcome adapters and action/feedback regions have been implemented for every exercise. Further migration must preserve gameplay and follow the acceptance criteria below.

## Exercise-specific adapters

- Word mosaics: two-tile selection visibility, red/green pair feedback, deselection and progressive semantic hints; see [outcomes](practice-outcomes.md).
- Character builder: indicate misplaced or wrong characters, allow correction where permitted and distinguish assisted final outcomes.
- Multiple Choice: balanced variants, pinyin visible by default only when non-revealing, automatic pronunciation enabled by default where a Chinese pronunciation target exists, manual replay whenever supported. Respect autoplay restrictions.
- Write Hanzi: maintain canvas transforms, cancellation, assisted-stroke evidence and playback reliability; see [writing activity](../hanzi/write-hanzi-activity.md).
- Matching and Listening: unify equivalent configuration controls and error/selection semantics without conflating their exercise-specific progress metrics.

## Acceptance and migration

Migrate existing exercises incrementally; each migrated type must retain its prior working gameplay and gain the shared shell, typed outcomes, input accessibility and persistence integration. Avoid an all-at-once replacement. E2E scenarios must assert session position, no duplicated titles/exit buttons, correct attempt accounting, focus transitions, mobile usability and resume behavior. Link concrete tests in the [E2E matrix](../../testing-quality/e2e/e2e-requirements-matrix.md); do not mark them passing without a browser run.

## Generated narrative control migration

Story Builder, Reading Comprehension and Conversation Practice now share the action/instruction presentation primitives. Their Check, Reply and Finish actions are not session-level Continue actions: they still call the existing exercise scoring callbacks. Conversation Practice uses a local Yew reducer to keep turn position, answer selection and correct-turn count coherent. This migration does not implement the planned typed outcome-evidence adapter, adaptive AI dialogue or common feedback ownership for the narrative exercise family.

## Shared primary actions migration (R704 WIP)

The shared action presentation now covers Check controls in generated Image Vocabulary, Sentence Translation, Spot the Mistake, Sentence Transformation, authored Fill Blank and Audio Dictation, Character Builder, and Type Pinyin. Click actions use `ExercisePrimaryAction`; forms retain native Enter-key submission through `ExerciseSubmitAction`, which renders a submit button without replacing form submission handlers. Existing check availability rules, exercise-specific selectors and grading callbacks remain local.

This consolidation is presentation-only and does not imply that every activity uses a unified typed attempt or outcome contract. The standalone Picture Vocabulary workflow and the separate Speaking and Character Hunt feature components still require targeted integration review. The existing animal progress and the `animato` transition boundary remain outside these action controls.

## Typed registry outcome boundary (R704 WIP)

The session page now supplies one `Callback<ExerciseOutcome>` to the exercise registry. Definition adapters convert existing `Callback<()>` completion and `Callback<bool>` score interfaces into the current persisted `ExerciseOutcome::Completed` or `ExerciseOutcome::Scored { earned, possible }` forms. This removes outcome-shape translation from the session page and makes the registry boundary typed while retaining current gameplay semantics. A focused test checks both conversions.

This does not yet implement the approved three-verdict result model, partial credit, per-attempt evidence, or a unified retry/hint journal. Those contracts require an explicit model and persistence change before rollout to the remaining gameplay components. The creature progress component, its top placement and the `animato` transition are unchanged.
