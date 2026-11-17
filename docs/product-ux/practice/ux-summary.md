# Practice summary UX

The authoritative current contract is [Practice outcomes and session lifecycle](../../learning-practice/practice/practice-outcomes.md). This page specifies how those outcomes should be presented. No universal percentage, correct-pairs total or doughnut chart is required for mixed Practice sessions.

## Information hierarchy

1. Display a clear completion heading and accessible count of completed exercise slots.
2. Show a stacked horizontal distribution of Correct, Partially correct and Incorrect outcomes, alongside the three numeric counts as text.
3. Show type-specific secondary details only when relevant. Matching may show correct pairs and incorrect attempts. Hanzi writing may show assisted strokes. Translation may include evaluator-specific feedback.
4. Show separately recorded skips without counting them as scored verdicts. A skipped exercise normally receives a replacement, so skips are attempts rather than completed slots.
5. Show a primary Practice again action that retains the saved session configuration and starts fresh eligible content. Provide a secondary action returning to Practice Setup without erasing preferences.

## State and accessibility

The distribution must be understandable without color. Respect reduced motion. Reserve space for feedback and actions to avoid layout jumps. Do not show unsupported percentages, claim a mastery gain for assisted content or treat dragon progress as session progress. An empty or not-yet-available verdict set displays an honest empty state rather than inventing zero accuracy.

Session recovery is governed by locally persisted state, not URL round offsets. Where source implementation still uses older behavior, track the mismatch as an open acceptance issue, not an alternative product requirement. See [acceptance criteria](../../testing-quality/acceptance/approved-behavior-acceptance.md).
