# Write Hanzi activity

## Entry and selection

Write Hanzi is available as a standalone learning activity and as a Practice exercise. Its vocabulary finder supports configured Word Sets and dictionary search by Hanzi, English meaning and Pinyin forms, with visible source identity and keyboard-operable suggestions. An empty Word Set must never make broader dictionary suggestions appear to belong to that set. Relevant Dictionary and Word discovery actions may pass a selected word into the workspace.

## Writing workspace

The learner traces characters using pointer or touch input against canonical reference geometry. Show clear success, rejection and reference-loading errors, without accepting a missing or corrupt reference as completed handwriting. Preserve a stable drawing area during completion and responsive resize. Undo, guidance and Watch replay must not leave stale strokes or interfere with active input.

Practice options include guidance (`none`, `start_cue`, `full_guide`), optional stroke-order playback, independent Pinyin and meaning display, and manual word pronunciation when supported. Assistance after repeated failed attempts must be distinguishable from independently accepted strokes. A completed assisted stroke is not automatically mastery evidence.

## Playback and animation reliability

Canonical stroke animation, guidance preview and the learner's current stroke must share the correct reference-to-screen transform and have independent, cancellable lifecycles. Repeated Watch/replay, character changes, re-renders and viewport resizing must not produce displaced fragments, duplicated outlines, stale masks or clipped strokes. Interrupted playback must leave a coherent canvas. Honor reduced-motion settings, touch input and keyboard controls.

The screenshots provided by the developer show possible disconnected stroke fragments. Their cause is **unverified**; inspect reference geometry, SVG masking, transform order and competing timers before asserting a root cause. Record reproducible browser evidence across varied Hanzi.

## Practice integration and results

The Practice session host owns advancement, persistent session data and scored outcomes. Writing components return stroke and assistance evidence without independently updating global dragon progress. The active exercise uses the shared frame and has no redundant exercise-type title or exit button. See [Practice outcomes](../practice/practice-outcomes.md), [shared frame](../practice/shared-exercise-foundation.md) and [testing acceptance](../../testing-quality/acceptance/approved-behavior-acceptance.md).

Past implementation notes, matcher details and provisional thresholds remain in [archived Write Hanzi source](../../archive-history/audits/superseded-maintained/learning-practice--hanzi--write-hanzi-activity-source.md); do not mistake that snapshot for browser acceptance.
