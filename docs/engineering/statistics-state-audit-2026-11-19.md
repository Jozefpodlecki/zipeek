# Statistics state ownership audit, R590 WIP

## Evidence and scope

Inspected the R589 source tree, particularly `stats/overview/chart.rs`, `overview/activity.rs`, `overview/breakdown.rs`, `stats/context.rs`, `stats/state/mod.rs`, `stats/query_hook.rs`, and Grammar/Vocabulary explorer page owners. This is a targeted state-management and correctness review, not a complete proof of correctness across the workspace. No Rust toolchain or Chrome session was available. Source observations below are not compiler or browser evidence.

## Ownership findings

| Area | Observed R589 source | Audit outcome | R590 treatment |
| --- | --- | --- | --- |
| Overview chart | One component owns two booleans (`ready`, `error`), a Chart.js handle, a stale-task guard, the async effect, fallback markup, and dialog visibility | Two booleans permit invalid/ambiguous status combinations, and concerns are unnecessarily coupled | Introduced `ChartStatus::{Loading, Ready, Unavailable}` and `overview/lifecycle.rs`; fallback moved to `overview/fallback.rs`; breakdown visibility stays local |
| Chart cancellation | A per-effect `Rc<Cell<bool>>` guard checks whether an async completion belongs to a still-mounted effect | This guard is appropriate in single-threaded WASM; not inherently unnecessary state | Retained guard and destroy-on-cleanup ownership pattern in the lifecycle hook |
| Fallback accuracy | A minimum 8% bar height was applied to zero-count weeks | Visually implies activity where there was none | Zero yields 0%; focused pure helper test authored |
| Chart instance | `use_mut_ref` retains `ChartInstance` without rerendering | Legitimate imperative-resource state | Kept in effect hook; explicitly drops previous retained chart before re-initialization |
| Breakdown visibility | Local `use_state<bool>` toggles one dialog | Appropriate ephemeral UI state | Kept local; no reducer action added |
| Async query pages | `use_query_page` stores `(query, QueryLoad)` and an effect-generation plus active flag | Stale query result protection exists; query/result association guards render | Preserve; evaluate redundant fields only after functional tests |
| Grammar/Vocabulary bulk selection | Each page duplicates active flag, query snapshot, generation counter, loading and error hooks | Maintenance and concurrency risk; query changes/cancellation need identical invariants | OPEN: extract typed reusable hook, with per-query tests and compiler validation; not mechanically edited in this checkpoint |
| DashboardState | Shared reducer owns selection, filters, pagination and overlay navigation; typed collections are separate in data layer | Some coupled transitions are appropriate; string identifiers and broad action interface remain debt | OPEN: action-by-action invariant audit, typed verified IDs at integration boundary; no parallel reducers invented |
| Chart theme | Effect dependency follows weekly buckets only, not accent CSS changes | Potential stale theme colors after switching theme; requires browser validation | OPEN: use explicit theme signal or observe theme changes after approved behavior contract |

## What must not be claimed

- Merely adding a `use_reducer` or extracting files does not establish good state management.
- `Rc<Cell<bool>>` does not make a component wrong by itself; it provides cleanup cancellation for futures whose underlying operation cannot be aborted.
- Static module checks and a successful ZIP CRC do not show that a Yew component compiles.
- No source-level change here proves correct chart behavior during theme changes, fast section switches or failed script imports.

## Validation requirements

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo make release-check
```

In the browser, change the relevant source sessions, switch section rapidly during chart import, test invalid/missing canvas behavior, switch themes, open and dismiss the breakdown with keyboard, and inspect zero-activity fallback weeks. Test `/` and `/zipeek/` bases where applicable. Open issues remain open until this evidence exists.
