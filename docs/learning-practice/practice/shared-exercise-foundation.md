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
- Display session position, such as `Exercise 2 of 5`, consistently. Distinguish it from exercise-local `3 of 8 pairs matched` or `Stroke 4 of 7`.
- Display instructions close to the activity; show mistake counts only when meaningful and comprehensible.
- A single top-right **Leave Practice** control saves progress for resumption. Do not duplicate an Exit practice button in the card. Ending or discarding a session is a separate explicit action.
- Use shared rules for selected, incorrect, partial, correct, assisted, waiting, completed and disabled states. Each state needs text or shape signals in addition to color.
- Preserve current Zipeek dark-violet visual identity and scenic immersive Practice layout; responsive and reduced-motion modes must remain operable.

## Lifecycle and evidence contracts

The common lifecycle distinguishes preparing, active input, submitted feedback, completed and skipped/replaced. Each type may have additional internal states but must not bypass the host's durability and outcomes contract.

A submitted attempt retains its evidence, even when a later attempt is successful. Outcomes use **Correct**, **Partially correct** and **Incorrect** where semantically valid. Skip and Replace generates a new item of the same type/configuration within the same session slot, records the skipped attempt separately and does not award mastery. Successful completion alone is not sufficient for acquisition evidence.

Feedback should explain the actual error when determinable, avoiding invented grammar or unsupported certainty. Do not produce an immediate full solution where correction and progressively helpful hints are pedagogically appropriate. Keyboard users and screen readers must be able to understand selection, feedback and next actions without color or motion.

## Exercise-specific adapters

- Word mosaics: two-tile selection visibility, red/green pair feedback, deselection and progressive semantic hints; see [outcomes](practice-outcomes.md).
- Character builder: indicate misplaced or wrong characters, allow correction where permitted and distinguish assisted final outcomes.
- Multiple Choice: balanced variants, pinyin visible by default only when non-revealing, automatic pronunciation enabled by default where a Chinese pronunciation target exists, manual replay whenever supported. Respect autoplay restrictions.
- Write Hanzi: maintain canvas transforms, cancellation, assisted-stroke evidence and playback reliability; see [writing activity](../hanzi/write-hanzi-activity.md).
- Matching and Listening: unify equivalent configuration controls and error/selection semantics without conflating their exercise-specific progress metrics.

## Acceptance and migration

Migrate existing exercises incrementally; each migrated type must retain its prior working gameplay and gain the shared shell, typed outcomes, input accessibility and persistence integration. Avoid an all-at-once replacement. E2E scenarios must assert session position, no duplicated titles/exit buttons, correct attempt accounting, focus transitions, mobile usability and resume behavior. Link concrete tests in the [E2E matrix](../../testing-quality/e2e/e2e-requirements-matrix.md); do not mark them passing without a browser run.
