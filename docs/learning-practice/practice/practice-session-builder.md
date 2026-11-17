# Practice setup and session execution

## Four-step setup

The learner follows **Path -> Exercises -> Vocabulary -> Summary**. The setup remains a coherent wizard; Back and Continue never discard configured settings inadvertently. Continue requires the current step to be valid. Selected and unselected controls must be visibly and accessibly distinct.

### Path

Present the supported Practice paths without empty spacer panels. A route that cannot create a valid session explains the reason before launch.

### Exercises

Start without implicitly preselecting exercise types. Provide **Select all** and **Clear all** actions with accessible icon labels. Select all only includes supported, currently available activities; AI-dependent exercises are disabled when their required service is unavailable. Preserve exercise configuration while temporarily deselected. The Configure tab exposes each supported exercise's applicable settings; exercises with none have a truthful no-settings state, not an unexplained blank panel.

Use the same visual controls for equivalent option types: segmented controls for short exclusive options, full-width radio rows for descriptive alternatives, checkboxes for independent flags and clear multi-selection patterns. Exercise-specific differences in available settings are permitted. See [shared exercise foundation](shared-exercise-foundation.md).

### Vocabulary

Select HSK/dictionary filters or personal Word Sets, honoring exact-entry exclusions for every exercise and replacement. Display the global excluded count and a link to Word Sets > Excluded. Eligibility counting is a separate cancellable, nonblocking operation, not a synchronous filter triggered by every keystroke. Do not interpret a calculation failure as zero eligible words. The global pool-exhaustion policy is configured in Settings, not in this wizard. See [vocabulary selection](../../dictionary-data/vocabulary/vocabulary-selection.md) and [counting](../../dictionary-data/vocabulary/eligibility-counting.md).

### Summary

Show selected exercise types, source filters, count and effective global policy. Save a named configuration as a Practice profile without including learner results or credentials. Profile management and versioned JSON export/import belong in Saved Practice profiles; see [profile portability](practice-profile-portability.md). Validate the session's eligible source before starting without blocking the UI thread.

## Execution contract

A dedicated Practice session workspace persists unfinished progress and can be resumed. The top-right leave control is the sole leave action. It does not silently end or discard the session. Active exercise cards do not repeat exercise names. Session progress counts completed slots, while exercise-local metrics and long-term learning evidence remain distinct. Skip and Replace preserves the configured slot and past submitted mistakes; results use three verdicts rather than a universal percentage.

The [outcome contract](practice-outcomes.md), [shared foundation](shared-exercise-foundation.md) and [acceptance checklist](../../testing-quality/acceptance/approved-behavior-acceptance.md) own those behaviors. The source's translation-specific session host must not be assumed to support all heterogeneous activities merely because the same setup wizard exposes them.

## Implementation and verification pointers

Inspect `crates/zipeek-web/src/pages/practice/setup/`, `crates/zipeek-web/src/components/practice/setup/`, `crates/zipeek-exercises/src/practice_draft/` and session persistence services before changing code. This specification is the target behavior, not proof of current implementation.

Earlier implementation snapshots and checkpoint details are preserved in [archived setup source](../../archive-history/audits/superseded-maintained/learning-practice--practice--practice-session-builder-source.md).
