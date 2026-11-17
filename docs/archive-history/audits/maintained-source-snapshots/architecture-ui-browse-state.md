# Ui Browse State

## Browse feature state (2026-11-04, R41 source refactor)

Browse is composed in `pages/browse.rs`; it no longer owns dictionary queries,
Word Set persistence, or form editing state. Its child components live in
`components/browse/`: `search_form.rs` owns automatic search controls,
`results/mod.rs` selects the status component and owns result-list wiring;
`results/status.rs` renders loading/empty/error states,
`results/word_card.rs` renders typed vocabulary entries, and
`results/pagination.rs` renders page navigation;
`word_set_panel.rs` owns the create/rename/delete/backup input fields.

`BrowseProvider` owns the route-scoped `BrowseState` reducer and abortable
DuckDB query effect. Search requests carry generation IDs, and superseded
responses cannot replace the latest state. Search and Retry both validate the
current saved-set boundary. Missing required Yew provider contexts are treated
as developer invariant violations and unwrapped; storage, query and user-data
failures remain recoverable results.

`WordSetsProvider` owns `WordSetsState` with the saved collection, selected
management target and a single success-or-error feedback state. Its callback
executes validated mutations through `services/word_sets.rs`, writes storage
before committing reducer state, and dispatches a failure without discarding
the previous collection if persistence fails. Delete/import invalidate Browse
rather than automatically displaying unrestricted results. An active saved-set
toggle updates the exact candidate list and resets Browse pagination.

Both providers currently mount only for Browse. Practice Setup still reads the
same localStorage key independently when it mounts; a future cross-route
Word Set provider requires a separately reviewed lifecycle and storage-sync
design. Do not describe this route-local provider as app-global.

The source Browse UI is a pragmatic dropdown/card implementation. It does **not**
implement the D45/A37 canonical chips, strategy control, URL-authoritative
query or numbered-page contracts. D45 approves the modern card/sidebar layout
and preserves the A37 query/URL semantics. A13 retains
that product discrepancy as open implementation work.

### Route composition and component imports (R43 - R45)

Browse's route branch imports `BrowseProvider`, `WordSetsProvider`, and
`DatabaseBoundary` explicitly at module scope. The route uses multiline nested
Yew tags for its provider composition; the providers remain scoped to Browse,
not the entire app. The Browse page and word-card component use readable
multiline callback and optional-action expressions. These edits are source
readability changes only: no provider ownership, event behaviour, or URL
contract is altered. A Node source-layout regression check covers the route
shape; Rust/Chrome checks remain local validation steps.

### Browse results component boundaries (2026-11-04)

The Browse input form observes query text, filter values and saved-set snapshots. Typing cancels the previous 320 ms debounce; filter changes and initial mount run immediately. Refresh explicitly reissues the current search, Clear resets the filter input state, and the reducer generation discards stale database responses. Word Set deletion clears the removed selection and reruns the query without an obsolete manual Search step. `BrowseResults` computes a `status_component` from the reducer state before its outer `html!` block. `BrowseLoading`, `BrowseEmpty`, and `BrowseError` are presentational status views; `BrowseWordCard` (now rendered as a compact single-column row) and `BrowsePagination` receive focused props/callbacks. The container alone resolves the required search and Word Set contexts and dispatches actions. `data-component` values and the older `data-browse-*` hooks remain stable. Do not fold these separate views back into nested match expressions inside `html!`. This source refactor still requires local Rust/Chrome validation.
