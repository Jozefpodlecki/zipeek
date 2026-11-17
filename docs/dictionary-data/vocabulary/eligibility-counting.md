# Nonblocking vocabulary eligibility calculation

## Requirement

Eligible vocabulary counts may require substantial HSK, category, part-of-speech, source and global-exclusion filtering. **No eligibility count, pool preparation or validation may freeze the browser main thread.** An asynchronous Rust future that performs a long synchronous calculation without yielding does not meet this requirement.

## Data semantics

Compute the count from the currently selected vocabulary source and filter combination, including exact-entry global exclusions, not an unfiltered dictionary total. Preserve OR semantics among selections within a filter group and AND semantics between applicable groups according to the actual configured source. Count dictionary identities without accidentally double-counting aliases or joined HSK memberships. Exclusion applies to the complete vocabulary entry, never arbitrary substrings.

## Request lifecycle

- Run costly work outside the UI event loop, preferably in a worker supported by the existing database runtime, or in explicitly bounded yielding batches where a worker is not possible.
- Give each filter revision a generation/request identity. Cancel prior work when possible; otherwise ignore stale results after cancellation, unmount or newer filter changes.
- Show a visible progress component while computing. Use measured phases/batches for determinate progress and an indeterminate state when meaningful progress cannot be measured. Never invent a completion percentage.
- Permit continued editing, navigation, keyboard interaction and screen reader use during calculation.
- Cache equivalent results when beneficial, invalidating when dictionary source, personal Word Sets, exclusions or relevant filter definitions change. Cache keys must not conflate sources or leak user data.
- Distinguish `calculating`, `counted zero`, `counted positive`, `cancelled`, `failed` and `stale` states. A failed count must never be rendered as zero.
- Offer Retry for failures and an accurate diagnostic without exposing private data. Do not silently broaden HSK or Word Set scopes to manufacture available entries.

## Session-launch boundary

The optional displayed count must not block wizard navigation. Starting a session still requires successful eligibility validation; if necessary, show an explicitly cancellable preparation state while validation completes. A confirmed empty pool yields a clear error, while an exhausted but nonempty pool follows the globally configured reuse-or-finish policy. User-visible responsiveness is required in both cases.

## Acceptance

Test rapid filter changes, active cancellation, late/stale response arrival, exclusions updated from another view, cache invalidation, database failure, zero eligible entries, large-dataset performance, and navigating away mid-calculation. Capture the progress widget in the E2E evidence suite and separately verify main-thread responsiveness under load; no numerical timing guarantee is implied without measured performance targets.

See [vocabulary selection](vocabulary-selection.md), [setup](../../learning-practice/practice/practice-session-builder.md) and [E2E coverage](../../testing-quality/e2e/e2e-requirements-matrix.md).
