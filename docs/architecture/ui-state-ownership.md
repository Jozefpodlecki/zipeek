# Ui State Ownership

> **Current approved target behavior:** [Shared visitation boundaries](../dictionary-data/vocabulary/vocabulary-visitation.md). Older implementation descriptions below record prior contracts and may not match the latest approved behavior or verified source.


## State ownership

| State | Authority and lifetime | Access and status |
| --- | --- | --- |
| Theme | App lifetime; existing approved saved preference | Implemented ThemeProvider, read/command hooks and persistence effect |
| Motion preference | App lifetime; live prefers-reduced-motion value | MotionProvider publishes a small preference snapshot; no new saved setting proposed |
| Database client | One app-owned runtime session | Implemented stable Dictionary context; typed read-only Database facade |
| Database readiness | App lifetime, independent of routes | Implemented status context, typed Retry command and pure reducer |
| HSK, PoS, mode, count, seed, offset and URL exclusions | Current URL, normalized by A08 | Route adapter produces immutable typed inputs; no second writable global filter store |
| Saved and pending exclusions | Explicit user actions, approved D13 lifecycle | A11 local setup hook validates saved IDs and exposes Retry/Reset; generation only, never union into loaded URLs. Shared feature commands can follow their first consumers. |
| Browse request/results/pagination | Browse feature scope | Feature reducer plus cancellable query effect; URL authority follows approved UX |
| Practice round, guesses, mistakes and progress | Practice session scope | Feature reducer; A07 builds round data, core FixedProgress tracks completion |
| Drawer expansion | AppShell; current reload-reset behaviour | Keep local unless another component actually needs shared commands |
| Hover, temporary focus and individual animated values | Owning component/controller | Local state/refs; never broadcast every frame through app context |

Do not create one application-wide reducer containing every result list, input
and animation tick. A provider belongs at the nearest shared ownership boundary.
Use props for leaf inputs such as a card label, selected state or click callback.
Components that need a shared service or feature state consume its hook directly;
intermediate layout components must not relay those values through several levels.

## Hanzi recognition feature ownership (R545 WIP)

The `recognize_hanzi` feature reducer owns input classification, Undo availability, request status, errors, candidate results, and the monotonic request generation. The page owns scoped async orchestration and workspace mounting; child components receive typed props. The browser crate owns canvas, pointer, upload and clipboard resources. Candidate parsing remains a pure feature module. Browser verification is pending.

## Statistics explorer ownership (R586 source-only WIP)

The Statistics page owns one route-scoped `DashboardState` reducer for shared filters, paging, selection sets and active Grammar detail. The query hook owns asynchronous request lifecycle, including stale-response protections. History timeline entry expansion stays local to its own row. `history/{day,entry,table_row}.rs`, `grammar/{detail,items}.rs`, `explorer/row.rs` and shared `stats/pagination.rs` receive typed properties and report selection/navigation through callbacks. They do not create new contexts or independent copies of shared state.

The parent explorers remain responsible for search/filter toolbars, bulk selection and query orchestration; a further boundary review is outstanding. This source-only checkpoint does not establish browser or WASM acceptance.

## R587 Statistics composition correction (source-only WIP)

`stats/mod.rs` owns route providers, preview activation and top-level selection review. Feature directories use `mod.rs` only for module declarations and exports; each page controller lives in `page.rs`. Settings catalogue search/category state is local, while persisted metric visibility and tile ordering stay in one route reducer. Settings children emit typed preset/metric/tile actions. Skills and Insights aggregation is pure and independently testable. History toolbar and Grammar view selector emit typed intent to their parent pages. Selection-review resolution is pure, and preview IDs still must not be transferred to Practice/Word Sets.

`state/mod.rs` retains atomic shared transitions and request phase invariants. `state/metrics.rs` and `state/filters.rs` only own types and local operations; they are not independent writable stores. History/Grammar/Vocabulary async pages retain query hooks and guarded bulk selection. No result of this source-only refactor should be treated as compiled or browser-accepted until the locked gates pass.

## R587 shared Statistics record snapshots (source-only WIP)

Read-only demo snapshots and section projections now use typed `WordRecords`, `GrammarRecords` and `SessionRecords` newtypes backed by `Rc<[T]>`, with construction from owned vectors and cheap shared clones. Each type exposes read-only slice operations, while bounded asynchronous query pages continue to use their own `Vec<T>` plus pagination metadata. The collection API does not expose mutation or mutable references, and no new context or reducer has been added. Locked WASM and browser acceptance remain outstanding.
