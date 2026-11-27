# Practice outcomes and session lifecycle

## Authority and implementation boundary

This document states approved target behavior. It does not assert that the current application implements or passes every requirement. Individual implementation and verification gaps are recorded in [open tasks](../../project-management/tasks.md) and the [issue library](../../issues/OPEN.md).

## Exercise verdicts

A completed scored exercise reports one of three outcomes: **Correct**, **Partially correct**, or **Incorrect**. Keep the meaning of these outcomes explicit for each exercise type. Do not manufacture a percentage, a binary success, or a universal pair-based count for mixed exercises. Not all exercise types can produce partial credit; those types report only meaningful outcomes.

Display an accessible stacked horizontal bar and textual counts for the three verdicts in the session summary. Use distinct colors and text labels. An empty outcome category has a count of zero. Do not use a Chart.js doughnut or require an additional charting dependency. The summary does not report a single universal percentage. Provide exercise-specific supplementary details only when meaningful, such as matching attempts, writing mistakes and assisted strokes.

Pair counts and streaks remain available within matching games where they have a clear meaning. Translation uses the three verdicts, and handwriting may record accepted, rejected and assisted strokes. A hint or assistance does not automatically confer independent success or mastery.

## Completion and progression

Session progress counts completed exercise slots against the configured session length. The dragon represents durable evidence of acquisition, not completion or exposure, and must be derived from persisted pedagogical evidence rather than an independent counter. Render progress and the dragon as separate, clearly labelled concepts.

There is **no Reset exercise** action. Skip discards the current unfinished exercise and generates a replacement of the same exercise type in the same slot, retaining that type's saved configuration. A session configured for five translation exercises still requires five completed translation exercises even after skips. A skipped item does not receive a scored verdict or mastery evidence. Previously submitted attempts and mistakes remain recorded where an evidence contract supports them. A replacement should avoid immediately repeating skipped material where eligible alternatives exist. Count skipped attempts separately from completed exercise verdicts.

Continue advances only after the required submission or completion. Resume restores the locally persisted session. A new session reuses its saved profile configuration and generates fresh content, respecting vocabulary eligibility, history and the configured pool exhaustion policy. Retain local persistence rather than encoding learner state, filters, exclusions or random offsets into URLs.

If an exercise is retried, previously recorded attempt evidence must not be erased. The exact per-exercise retry scoring contract remains an implementation-specific acceptance topic; do not infer success or mastery from a retry alone.

## Accessibility and summary content

Use real DOM text for verdict counts and progress. The stacked bar is supplementary. Follow the contextual copy policy: clear instructional feedback during exercises and restrained encouragement on achievement screens. Explain why an answer was correct, partially correct or incorrect when the evaluator has defensible reasons. Respect reduced motion, avoid layout jumps, and keep actions operable by keyboard.

See [vocabulary selection](../../dictionary-data/vocabulary/vocabulary-selection.md), [learning evidence](learning-evidence.md), and [acceptance criteria](../../testing-quality/acceptance/approved-behavior-acceptance.md).

## Word mosaics: selection and pair feedback

Word mosaics is an interactive Chinese-to-meaning matching exercise. This section specifies approved target behavior, not verified current behavior. The shared Practice exercise frame owns session position, exercise instructions, navigation, common feedback presentation and persisted outcome reporting. Active exercise headers do not repeat the exercise type name, and an additional Exit practice button must not duplicate the session's top-right leave control. The game retains its own matched-pair count as an exercise-specific metric.

- **First selection:** The chosen tile remains unmistakably selected until the second selection, cancellation or a state transition. Use a prominent outline and elevated/contrasting treatment. The state must also be communicated without relying only on color. A second selection makes both tiles identifiable while their pairing is evaluated.
- **Correct pair:** Both selected tiles receive immediate, clearly visible green confirmation, after which they leave the interactive pool or become clearly locked as matched. Update the matched-pair count only once per successful pair, and prevent repeated scoring. Any optional Chinese audio is governed by the shared audio preferences.
- **Incorrect pair:** Briefly emphasize both selected tiles in red, show concise text that identifies the attempted pairing when helpful, record one incorrect submitted pairing, then automatically clear the selection so the learner can try again. The tiles remain available. Use visible error indicators beyond color. The feedback must not consist solely of the generic message `Not a match. Try again.`
- **Progressive hints:** The first mistake may receive concise error feedback without disclosing the answer. Repeated mistakes involving a tile progressively provide a meaningful semantic clue, and eventually may gently emphasize an appropriate candidate when the system has reliable mapping information. Do not invent a meaning or disclose the complete solution immediately on the first wrong selection. Hints and eventual correct matching must retain prior mistake/assistance evidence and must not automatically count as independent mastery.
- **Input and transitions:** Mouse, touch and keyboard users can make and change selections. Feedback timing is long enough to perceive, but does not leave the grid stuck in a feedback state. Respect reduced-motion preferences by retaining state, outline and text feedback without requiring flashing animation. Announce results accessibly and return focus predictably. Prevent double clicks and stale timers from resolving a different later selection.
- **Shared design:** Preserve the application's existing dark-violet card and immersive Practice view. Reuse shared selection, progress, feedback and actions conventions where possible, while keeping pair logic in the Word mosaics component. Avoid putting a separate custom header or redundant Exit practice control in this exercise.

An actual browser run is required to verify timings, semantic hints, screen-reader announcements, input handling, persistence and result scoring. Screenshot evidence of the current generic message does not establish that these requirements are implemented.

## R704 typed-outcome integration status

The persisted `ExerciseOutcome::Assessed(ExerciseAssessment)` shape now supports explicit `Correct`, `PartiallyCorrect` and `Incorrect` verdicts, cumulative attempts/mistakes/hints and an assistance flag. `Completed` remains unassessed, and old `Scored { earned, possible }` values remain readable for existing saved sessions. The session summary uses a stacked verdict bar based only on assessed outcomes, with completed-but-unassessed outcomes reported separately rather than classified as correct. Binary generated evaluators supply a minimum one-attempt assessment; they cannot infer partial correctness, prior retries or hint use. Migrating every exercise to report full evidence remains outstanding and requires actual evaluator integration and browser acceptance.
