# Statistics dashboard: architecture and state contracts

**Status:** approved Statistics 2.0 architecture, partially implemented through R567. `stats/state.rs` now owns coupled UI transitions, `stats/context.rs` provides route-local state, and `stats/overview/mod.rs` owns the initial fixture-backed Chart.js plot and shortlist. The remaining structure below is the pending architecture plan, not an implementation claim. The product contract is [Statistics design](../../product-ux/pages/learning-statistics-preview.md); the acceptance plan is [Statistics design acceptance](../../testing-quality/e2e/statistics-dashboard-design-acceptance.md).

## Ownership boundaries

- `zipeek-web` owns dashboard composition, typed feature state, Yew context/provider and presentation. It does not independently compute proficiency or modify the authoritative assessment rules.
- Pure transformation/domain functions own sort, filtering, grouping, aggregate projection, selection semantics and insight derivation over validated facts. Start in focused modules; do not add a crate merely for speculative reuse.
- `zipeek-browser` owns browser API interop and Chart.js module/instance lifecycle through Rust newtypes. Use pinned Chart.js and the existing thin `wasm-bindgen` dynamic import boundary; no new JS application logic.
- Personal learning storage services, when authorized and browser verified, own transactional reads/writes and exports. Their future projection interface must distinguish stored Practice outcomes from qualifying proficiency evidence.
- Fixtures belong to a read-only deterministic `Demo` data source. They never touch storage, pedagogy worker, AI providers or network, and are not silently promoted to production data.

Follow [UI state ownership](../ui-state-ownership.md), [Rust/WASM interop](rust-wasm-interop.md) and [coding standards](../../engineering/coding-standards.md). No dependency additions are approved in this design.

## R571 typed paginated query contract

The `stats/query.rs` query boundary is the first API-shaped data contract for the Vocabulary, Grammar and History explorers. `StatisticsQuerySource` exposes three asynchronous methods returning owned `QueryPage<T>` values. `DemoStatisticsSource` implements the same trait against in-memory records; future services must implement the contract rather than expose unbounded `Vec<T>` responses.

`PageRequest` uses an explicitly validated zero-based page index, a limit of 1 through 100 and a bounded offset. `QueryPage` contains a page of records, the **total number matching the filters before pagination**, the offset and page size. This keeps the page count stable on short and empty pages. Search is trimmed, case-normalized for Latin text and limited to 256 bytes; the upper bound protects network requests and avoids unbounded server work. Grammar uses course/pattern/description search. History uses date/activity/focus plus a Practice/Games category. Vocabulary uses Hanzi/Pinyin/meaning, a typed evidence-gap filter and a typed deterministic sort order. All three have a stable ID tie-breaker to prevent a row moving between otherwise equal-ranked pages.

The existing seven-tab `StatisticsSource::load` contract remains responsible for bounded section projections and per-tab loading. `StatisticsQuerySource` is a separate async contract for page retrieval. **R572 now drives the Vocabulary, Grammar and History explorer pages through this asynchronous contract.** A reusable Yew query hook owns each request lifecycle, keys results to the exact filter/sort/page query and rejects completions after a query change or unmount. The existing route-scoped reducer owns selection across pages, and bounded Select all matching requests scan the source in pages of at most 100 records, declining selections above 250.

History's seven-day activity display uses a separate `ActivityQuery`, with explicit date strings supplied by the fixture viewer and aggregate counts returned by the data source. It does not infer seven-day totals from a single session page. Production adapters must obtain applicable dates from the approved local-time policy and implement aggregation without fetching the full session ledger.

The Overview, Skills and Insights projections still use section-level bounded loads. The `DemoStatisticsSource` internally materializes small fixture collections to answer queries; this is **not** permission for production sources to load thousands of records and paginate them inside Yew. Canonical learner identities, remote API implementation, source versioning and real Word Set/Practice handoff remain separate, gated contracts. Do not assume loading all 10,000 records into a tab is acceptable in production.

R573 strengthens this boundary after the first Windows R572 compiler report. Every completed Vocabulary, Grammar and History page is validated against its exact request before a Yew component can render it: response offset and limit must agree with the request, rows must fill the page up to the matching total except for a genuinely short final page, and pages beyond the end must be empty. Mismatched responses become recoverable `InconsistentPage` query errors. Bulk all-matching queries apply the same validation on each requested page, then reject repeated identities across pages. This guards against partial/misaligned remote responses and selection duplication, but does not yet define server snapshot consistency under concurrent dataset changes or canonical learner identity.

The paginated explorers derive visible range labels from the returned page, never from the requested page size. If a future source shrinks the result set and the saved page falls beyond the end, the reducer moves to the last valid page before requesting it again. Request-generation guards apply to all-matching selection as well as page reads: Clear and a newer matching request invalidate old selection completions. These contracts are implemented as source code and authored tests; acceptance still requires Windows WASM compilation and live browser observation.

The query logic and `QueryLoad<T>` lifecycle are pure Rust so backend adapters can be tested without a browser. Queries must be run against canonical, validated records, not constructed from untrusted string identities. Current fixture IDs are only demonstration identities. The 10,000-record tests exercise ordering, limits and counts; they are not runtime benchmarks or a latency guarantee. No network or personal storage is used by the demo implementation.

## Proposed module boundaries

```text
crates/zipeek-web/src/pages/system/stats/
  mod.rs                     Feature entry, provider setup, route lifetime
  state.rs                   State/actions/reducer, invariants, tests
  context.rs                 Narrow typed context and dispatch hooks
  data.rs                    Read-only data-source trait and projections
  fixture.rs                 Deterministic demo identities, sessions, assessments
  selection.rs               Typed selected-ID/filter snapshot operations
  components/
    dashboard_shell.rs       Toolbar, section navigation, accessible status
    metric_tile.rs           Metrics and definitions
    chart_panel.rs           Chart with tabular equivalent
    data_table.rs            Sort, stable table viewport, responsive rows
    pagination.rs            Typed pagination state and controls
    evidence_indicator.rs    Semantic status glyph/text/tooltip
    selection_toolbar.rs     Bulk actions with confirmation
    loading_skeleton.rs      Preview loading shape and busy announcement
  sections/
    overview.rs
    vocabulary.rs
    grammar.rs
    skills.rs
    history.rs
    insights.rs
    settings.rs
```

This is a planning map, not a file-count quota. Extract modules based on actual coupling, avoid duplicated containers and props, and retain straightforward local components when a file is small. Do not create global providers for simple local form controls.

## Feature reducer and context

The feature reducer owns state with invariants that must change together. A possible shape is:

```rust
struct DashboardState {
    phase: PreviewPhase,
    section: StatsSection,
    date_filter: DateRangeFilter,
    vocabulary_query: VocabularyQuery,
    grammar_query: GrammarQuery,
    vocabulary_table: TableState,
    history_table: TableState,
    grammar_table: TableState,
    selection: SelectionState,
    tile_layout: TileLayout,
    insight_focus: Option<InsightId>,
}

enum PreviewPhase {
    Closed,
    Loading { request_id: u64 },
    Ready,
    Failed { request_id: u64, error: PreviewError },
}
```

Transitions use typed actions (for example, `PreviewRequested`, `PreviewReady`, `PreviewClosed`, `SectionChanged`, `QueryChanged`, `PageChanged`, `SelectionChanged`, `TileToggled`, `PresetApplied`). State transitions clamp invalid pagination, invalidate stale async completions and reset only the state implied by the action. The reducer must not derive assessment judgments from demo counts.

A `StatsUiContext` may expose a stable state handle plus typed dispatch actions to sibling sections that genuinely need it. Keep heavyweight immutable projections in a separate read-only `StatsDataContext` or a memoized selector layer so a checkbox does not force every chart and table row to reconstruct. Local row hover, tooltip visibility and temporary input focus remain local component state. Avoid prop drilling through five wrapper levels, but do not substitute a giant global context for small typed props.

A provider must be mounted under a route owner that cleans up on navigation. No duplicate provider trees, unexpected optional contexts or hidden dependencies on app-global storage are allowed. Follow the existing `AppEnvironment` injection pattern for browser capabilities and user preferences.

## Data-source abstraction and typed identity

The UI consumes read-only projections, not tables of arbitrary strings. Use typed keys for `WordId`, `HanziId`, `GrammarPointId`, `SessionId` and `ObservationId` that cannot be mistaken for one another. A presentation projection separates dictionary labels and facts from evidence metadata.

The intended abstraction has two modes:

- `DemoSource`: deterministic, in-memory and offline; yields fictional identities, sessions and evidence, tagged `Demo` at the source boundary.
- `VerifiedEvidenceSource` (future): reads only records that meet explicit durability and assessor provenance gates. It must not be substituted into the preview until storage and assessment decisions are implemented and browser verified.

A lightweight projection API could conceptually support `overview(filter)`, `vocabulary(query, page)`, `grammar(query, page)`, `skills(filter)`, `history(query, page)` and `insights(filter)`. It must expose `total_count`, stable page IDs, a last-updated boundary and provenance/capability flags, not just raw `Vec<String>` columns.

For 5,000+ word identities, filtering/sorting/pagination should run over indexed or memoized projections. Do not linearly clone full records or create thousands of Yew callbacks on every movement/selection. Use deterministic secondary sort keys and take bounded pages. Benchmark before adding virtualization, worker offloading or caching layers.

### Provenance and semantic constraints

Facts include `source_kind`, `assessor_kind`, `assessment_kind`, `assistance_level`, `occurred_at`, `session_id`, `attempt_id` and item identity when applicable. An observational row cannot claim successful speech assessment from a recording's existence. Reporting capabilities must distinguish **available, unavailable, unassessed and invalid**, rather than converting unknown into false. A grammar course-completion award is not the same as vocabulary proficiency or a grammar exercise attempt.

The demo source can exercise all statuses, but its outputs are marked non-assessed; UI components must never accidentally reclassify demo rows by inspecting their numerical counts. Avoid creating a synthetic `proficient: bool` for demonstration data.

## Table state and pagination invariants

`TableState` has a page index, validated page size, typed sort column and direction. It is always paired with a query/filter snapshot version. Filtering changes clamp or reset page, sort remains deterministic, and range formatting handles an empty result without displaying `1-0 of 0`. Page sizes initially 10/25/50/100 for explorers, with a compact fixed shortlist in Overview.

Stable visual height comes from layout reservation or min-height, not fake rows. The paginator stays anchored, but mobile and narrow screens allow normal document scrolling. Horizontal table scrolling is allowed when semantic row readability is preserved. Virtualize only if 5,000+ fixtures demonstrate that pagination and memoized projections are insufficient.

Selection is a typed set of stable item IDs plus dataset/filter identity, not a `Vec<usize>` of visible row positions. Model explicit scopes: visible page, all matching query and manual selection. `Select all matching` must display an exact or bounded count and require confirmation when large. Selection across pages must be preserved across sort/pagination, not silently lost. Changing incompatible source/identity kind must invalidate or migrate selection according to an explicit visible action. User-visible counts cannot mix words and grammar points.

## Practice and Word Set handoffs

The dashboard prepares a **draft**, not a running exercise. Selecting vocabulary offers Word Set add and Practice setup; selecting grammar points offers Practice setup with Sentence Transformation preselected. Both use typed identifiers, validate eligibility and exclusions, respect the existing selected/default Word Set, and provide an explicit review step before Start. Large selections should use a scoped transfer object rather than huge query strings; implementation must respect route reload requirements and explicitly disclose if a draft is temporary.

No direct pedagogical records are written by selecting rows or opening a dashboard insight. Word Set mutation occurs only after the user confirms an explicit destination. The existing Practice setup and session-domain contract, rather than the dashboard, decides whether a requested activity configuration is valid.

## Charts, metrics and insight rules

Use the existing pinned Chart.js via typed `zipeek-browser` bindings. A Rust component owns `ChartInstance` and disposes it on unmount or dataset change; Rust owns dataset mapping, theme token resolution and accessibility descriptions. No direct JS model or lifecycle state in `zipeek-web`. Reuse Chart.js only where comparing a time series or distribution is clearer than a table. Each chart has an equivalent accessible summary/table and a deterministic animation/no-motion mode.

A metric definition must expose stable key, label, unit, population, date interval, deduplication rule, and explanation. Avoid a composite or opaque "mastery" value. Technical metrics such as vocabulary session appearances are optional tiles and explain their precise counting rule.

Insights are deterministic rules over eligible projections, each with `InsightId`, source scope, reason, evidence links, time window and an optional typed recommended action. A missing observation must not become a performance-failure insight. Prioritize explainability and avoid sending personal evidence to AI providers.

## Tile configuration model

`TileDefinition` declares a stable ID, category, renderer, required source capabilities and supported dimensions; `TileLayout` declares a validated ordered list of enabled tile IDs and optional display preferences. Offer a few curated presets (Learning, Practice, Detailed) and search/filter across the library. Preserve an explicit Reset action. Constraints include unique tile IDs, upper bound, well-defined default layout, safe handling of unavailable tiles and accessible move-up/down controls.

The design preview stores configuration only in current Yew state. No persistent preference key or import/export schema is approved by this document. Future persistence requires a separate decision on format, ownership and fallback behavior, not an incidental localStorage write.

## Async and lifecycle contract

The Demo loading transition may intentionally last two seconds. Associate its completion with an incrementing request generation; on navigation, preview close or restart, invalidate the generation and drop the timer/future safely. A stale completion must never reopen preview after cancellation. Real data reads must not have an artificial delay. Model errors and no-data separately, with bounded retry and focused feedback. The UI must not expose sample data during the temporary loading phase.

## Unit and integration testing priorities

Pure reducer tests: phase transitions, stale loading completion, invalid tab changes, selection scope, filter/page reconciliation, preset validation. Projection tests: stable sorting and page boundaries at 0, 1, 8, 30, 5,000 and 10,000 records, ID uniqueness and deterministic fixture generation. UI tests: keyboard tabs, selection retention, accessible status indicators, large filter changes, theme/contrast, and route lifecycle. E2E tests: root/hosted routing, disabled live statistics, Demo separation, Word Set confirmation, Practice draft not auto-start, grammar handoff, chart fallback, reduced motion and error states.

The formal stepwise gate and scenario matrix live in [Statistics design acceptance](../../testing-quality/e2e/statistics-dashboard-design-acceptance.md). This architecture alone does not authorize production evidence wiring, schema changes or new dependencies.

## R567 implementation boundaries

The route-scoped reducer owns vocabulary and grammar queries, deterministic sorting, page indices, selected identities and selection-review visibility. Explorer components consult the existing scoped context and receive immutable demo fixture data. Each selection uses a stable string identity, not a row index, and remains selected when pagination changes. Changing a query resets the relevant page.

A distinct Grammar tab uses illustrative pattern records, not the canonical learning-evidence ledger. Skills can focus the vocabulary explorer on items whose illustrative evidence is absent or flagged for review. History is an expandable session view. Insights expose counts and a route into the evidence explorer instead of claiming automatic proficiency diagnoses.

The Chart.js bridge now uses `serde_wasm_bindgen::Serializer::json_compatible()` for plain JavaScript configuration objects, with a Rust-rendered visual activity fallback while the chart is unavailable. Browser acceptance is still required to confirm module import, theme changes and Chart.js lifecycle.

Temporary selection review does **not** mutate Word Sets or transfer its IDs to Practice. Its link opens the independent Practice setup with no selected-item payload. A typed, explicitly reviewed transfer contract is still required before offering a true bulk-practice action. Production evidence and stored learning records remain out of scope.

## R568 preview-state controls

`DashboardState` remains the sole reducer owner of preview lifecycle, tab selection, vocabulary page index and page size, query/sort state, selected vocabulary and grammar IDs, and Overview tile selection. `DashboardPreset` supplies immutable initial card selections for Focused, Activity and Detailed; a subsequent individual toggle changes the selection without maintaining a redundant preset flag. Page sizes are allowlisted at 10, 25, 50 and 100. Reducer actions reject oversized matching-result bulk selections (over 250) and preserve off-page selected identities. The UI reports the selection ceiling before a user can invoke it. All state stays in memory and is cleared on a new preview; no browser storage, network transfer, DuckDB schema or dependencies change.

## R570: section-scoped asynchronous projection loading

Statistics no longer renders the seven demo sections by reading fixture constants in their presentation components. A route-scoped `StatsSourceContext` provides a `StatisticsSource` implementation with an asynchronous `load(DashboardTab)` contract. The `DemoStatisticsSource` converts immutable sample constants into owned `WordRecord`, `GrammarRecord`, and `SessionRecord` projections. A future verified implementation can supply the same typed results, but it must separately satisfy provenance, privacy, and assessment authorization requirements. Merely swapping the source does not constitute approval to use personal records or infer proficiency.

A separate `StatsDataContext` owns `StatisticsDataState` and `SectionLoad` transitions. Loading, successful data (including an empty result), and recoverable failures are distinct states. The reducer rejects a payload whose variant does not match the requested section, preventing an unexpected API response from reaching a rendering component. Each request has a generation number and tab identity; stale responses cannot supersede a newer load. Leaving an active tab invalidates its pending load; closing the preview clears the section cache. Completed section projections are cached for the life of the preview. Retry starts a new generation. Data loading uses Rust futures without introducing a new browser JavaScript service.

`LoadedSectionContext` is mounted only after the requested section has succeeded and holds an immutable `Rc<SectionPayload>`. Overview, vocabulary, grammar, skills, history, insights, and settings read their projection through that provider. Fixture constants remain in the demo adapter and tests, not in rendering. This eliminates presentation-level coupling to the mock backend and keeps stateful filtering and selection in the existing UI reducer.

**Scope boundary:** R570 fetches a complete bounded projection for each section, and explorer search, sort and pagination still operate over that loaded projection. It is suitable for the current offline fixture and for future bounded API results. Before enabling a large remote source, extend the typed request/response contract with query and filter identity, page size, cursor or offset, stable total counts, server-side sort, and cancellation semantics. Do not fetch thousands of personal records merely to reapply the current client-side filters; do not infer a production paging implementation from this foundation. Preserve demo provenance labeling until an explicitly approved verified evidence source exists.

The two-second demonstration transition is only the initial preview gate; each section has a separate real request lifecycle and no artificial section delay. The module's error UI offers Retry and does not treat an empty result as a transport error. In the future, ensure provider errors are mapped to safe user-facing text and do not expose raw personal records or credentials.

## R575 metrics registry and ownership

`stats/tiles.rs` owns a typed twenty-entry optional tile registry with labels, category, evidence-safe explanation and pure projection aggregation. It owns the order/visibility data structure, while `DashboardState` owns state transitions and `stats/settings/mod.rs` owns the searchable library and keyboard-operable up/down controls. `stats/overview/mod.rs` renders additional cards only when selected. Do not move these aggregates into components that read global fixture arrays; they consume `LoadedSectionContext`'s owned Overview projection and must remain replaceable by the asynchronous source. Four existing core cards remain independently toggled. Preview closing resets the ephemeral tile configuration. No persistence or new dependencies are introduced.

## R576 History view ownership and insight projection

`DashboardState.history_view` is a typed route-scoped `HistoryView` with Timeline and Table variants. Changing the view does not change the underlying `HistoryQuery` or reset the current page. Selecting a date dispatches the existing `HistoryQuery` action, which resets pagination and triggers a new bounded async session request. The `ActivityQuery` retains the seven-day context when the search value represents one of those dates, and otherwise honors the user's search/category filter. This deliberate split avoids misleading chart changes caused by choosing a single date while preserving query-keyed async safety.

Skills and Insights continue to consume owned, async-loaded `WordRecord` projections, never private storage. Presentation computes category counts only from fixture evidence statuses. Assessed performance remains unavailable unless separately qualified assessor projections are introduced under the learning-evidence approval gate. Neither the added view variant nor the expanded insight cards require a new dependency or new persistent schema.

## R577 Grammar detail identity and provenance boundary

Grammar `GrammarRecord` contains separately nullable `canonical_id`, `course_id`, and `assessed_attempts` fields. The demo source intentionally supplies `None` for all three. The existing `id` remains a stable **preview-only** row identity, suitable for pagination, disclosure and local selection but **not** for a canonical grammar-point or Practice transfer. The course title is a sample category, not a verified course attribution. `attempts` counts fictional practice activity; only independently qualified `assessed_attempts` can be described as assessed evidence. Production adapters must verify identifiers and provenance before populating these optional fields.

The Grammar detail panel consumes the current async page's owned `GrammarRecord` and never re-reads `GRAMMAR_SAMPLES`. `DashboardState.grammar_detail_id` owns the open disclosure; changing the search/page or leaving Grammar clears it, while selection persists. Future remote detail APIs should add a typed by-ID request if a detail payload exceeds the page projection. Avoid silently loading entire grammar catalogues into presentation components.

The selection review remains an explicit no-transfer boundary. Its Practice link only opens the independent setup route without carrying fixture IDs or changing Word Sets. The approved ST2-017/ST2-018 handoff still requires canonical stable IDs, explicit exclusions, a preview/confirmation stage, Sentence Transformation preselection, default Word Set validation and no auto-start. These are not implemented by R577.

## R578 component and technical-debt audit (2026-11-19)

The Statistics feature retains a coherent module structure. `stats.rs` composes the route. `state.rs` and `context.rs` own route-scoped UI state, selection and section navigation. `data.rs` owns typed owned section projections, the Demo source and its separate async data reducer. `query.rs` owns validated query requests and deterministic pure filtering, sorting and aggregation. `query_hook.rs` owns request-generation and disposal behavior. `overview.rs`, `explorer.rs`, `grammar.rs`, `skills.rs`, `history.rs`, `insights.rs`, `settings.rs`, `tiles.rs` and `controls.rs` remain independently focused presentation features. This is intentional, not a reason to introduce another global state manager or props through unrelated parents.

R578 expands `SessionRecord` with `started_at` (a UTC ISO 8601 timestamp with a `Z` suffix) and `duration_minutes`. The Demo source synthesizes these deterministic values in one place; components **only format** them and cannot invent a timestamp from a date. History sorts by full timestamp with an ID tie-breaker and searches either the date or the timestamp. The seven-day `ActivityBucket` query now returns Practice and Games counts as well as the daily total, and the compact chart composes a two-category stacked bar with a visible legend. The demo intentionally displays UTC until a production local-time policy and browser formatting adapter are approved. Its expanded dataset combines 72 declared catalogue sessions and 84 recent interleaved records, without qualifying any fictional outcomes as learner evidence. The Overview chart and breakdown read the **same** loaded session projection and the chart remains independent of History page queries.

### Tracked technical debt and owner

| Priority | Debt | Boundary and next acceptance |
| --- | --- | --- |
| P0 | Native and WASM compilation, Chrome light/dark/responsive/keyboard E2E still unverified in this packaging environment | Test the actual R578 source on Windows; fix compiler or browser failures before acceptance |
| P0 | Canonical vocabulary/grammar IDs and verified Practice/Word Set setup draft transfer remain unavailable | Define and validate source identities in the domain and destination setup contracts; never send fixture IDs to storage |
| P1 | Production timezone policy, calendar range and local date/time formatting remain unspecified | Add approved time source and explicit timezone identity; replace preview UTC formatting through the browser service boundary |
| P1 | The demo data source regenerates owned vectors on every query; large-scale production caching/paging and source versioning have no adapter | Implement a proper persistent query source with bounded pages and stable IDs, no UI-wide prefetch |
| P1 | The Chart.js activity renderer is pinned to a fixed five-week window and does not currently react to theme changes without a data reload | Extend typed chart input to include time range and theme; verify cleanup, reduced motion and fallback in browser |
| P1 | Demo History remains a seven-day static activity selector, not a navigable calendar with date range and exercise-type facets | Extend `HistoryQuery` and activity aggregation through the source; keep query identity and pagination in reducer |
| P1 | Validated 5,000/10,000 record **browser performance** and full keyboard accessibility are outstanding | Capture measurements and screenshots, not just authored unit tests |
| P2 | Settings tile registry is functional but lacks tile previews and broader dashboard drag/keyboard layout choices | Use the existing typed registry and reducer; do not duplicate config state within each tile |

**Audit outcome:** No new dependencies, global contexts, cross-crate state manager or second fixture access layer are justified. Continue to keep data-model generation out of view components. Technical debt is tracked in current TASKS rather than being described as complete or being buried in a historical release note.

## R579 interaction and simulator ownership audit

`stats/simulator.rs` owns the deterministic preview-data generator, separate from `data.rs` which implements the asynchronous `StatisticsSource` and `StatisticsQuerySource` contracts. The default preview maintains its small owned projections; test scenarios can create 5,000 or 10,000 uniquely keyed vocabulary rows and request bounded pages. Fixture generation is deterministic, with stable result ordering and explicit scored/unscored session categories. The current fixture is not a production identity source.

UI-only control styling remains within the explorer components, while select/toggle state stays in the route reducer and async response generations remain in the query hook. The selection toolbar occupies its slot from the first render: selection updates the count and enabled controls without inserting a new block above the table. Icon actions must retain descriptive labels, focus states and tooltips. The History day navigator uses compact buttons with full accessible date/count labels and category proportions.

**Debt retained:** Reconstructing the same fixture vectors on each async query is CPU-expensive for larger scenarios; introduce a shared snapshot or indexed fixture source after an actual profile. Grammar source fixtures remain defined in the Grammar module, and production identity validation, browser timezone preferences, remote date ranges, source cache invalidation, responsive testing and browser benchmarks remain open. Do not merge reducer/UI state with source/query state to solve these issues.

## R580 ownership and technical-debt checkpoint

The grammar catalogue is sourced at compile time from the repository-maintained `docs/learning-practice/GRAMMAR_TEACHING_INDEX.tsv`. The Statistics simulator parses that source into owned `GrammarRecord` projections, preserving its teaching ID, region and editorial status. The Yew Grammar explorer sees only asynchronously queried page projections, never the source TSV itself. The simulator caches immutable vocabulary, session and grammar snapshots in per-thread `Rc` storage. This is a fixture-source optimization, not a production API cache policy.

The existing UI reducer still owns search, selected identities, sorting, page positions and review panel visibility; the query layer owns bounded retrieval and async lifecycle. New alphabetical and encounter sort directions use stable ID tie-breakers. Do not introduce another feature-local global state store.

Remaining debt: true source-backed suggestion endpoint instead of page-scoped suggestions; production canonical learner/grammar identities and guarded draft handoffs; query cancellation and responsive/browser performance measurement; timezone-aware History presentation and time-range navigation; complete keyboard interaction audit and production selection descriptors for very large filtered sets.

## R581 Grammar query extension

`GrammarQuery` now owns a typed teaching-region group, a five-mode deterministic sort and validated page request. `DashboardState` retains the active region, sort and page size, resetting the current page and expanded detail when any of these changes; selected preview teaching IDs are retained. The demo query applies search, region filtering, stable sorting, total calculation and slicing in that order. Both the main table and the independent six-item suggestion query go through the same `StatisticsQuerySource::grammar` async contract and response validation. The suggestion query no longer inherits the currently selected table page.

The teaching-region grouping is not course attribution or HSK level: the source index exposes teaching regions but not verified course relationships. The current `GrammarRecord` is a demo projection. All 550 source records have pending editorial review; no evaluated grammar proficiency or assessment score can be inferred from the generated `attempts` count. Query callbacks currently issue independent requests for suggestion results on changed input; dedicated debouncing, combobox focus semantics and a canonical API suggestion index remain tracked debt. Do not move suggestion search, region filtering or row sorting into the Yew table view when a production adapter is introduced.

## R582 async interaction ownership and remaining debt

History week offset is coupled to active date filters and belongs in the route-scoped `DashboardState` reducer, not local per-row hook state. Date-window construction is a pure query-layer function. `ActivityQuery` accepts the seven dates independently from `HistoryQuery`, so the activity chart does not silently change when the table changes page. Week controls remain mounted during async transitions to prevent avoidable layout shifts. The preview reference date is intentionally fixed for deterministic fixtures and must be replaced by a validated local-calendar contract in production.

Grammar suggestion focus and keyboard interaction is component-local ephemeral state; the search string remains in the dashboard reducer, while suggestion page requests remain owned by `use_query_page`. The source-wide query is bounded and guarded against stale updates but is not debounced and is not a dedicated semantic suggestion endpoint. The current combobox needs browser and assistive-technology acceptance for blur/pointer dismissal. These are tracked debt, not completed compliance claims.

## R583 ownership and technical-debt checkpoint

`DashboardState.grammar_view` owns the current Grammar layout; the toggle is an independent display preference and must not reset query pages or selected IDs. Grammar table and card presentations read the same `QueryPage<GrammarRecord>` from `use_query_page`. The Grammar inspector receives a cloned item from that bounded page and must never reach into TSV fixtures or issue independent unmanaged remote reads. A future full detail endpoint can be introduced through the typed source interface if content cannot fit the page projection.

Selection review is composed once by `StatsContent` and uses the existing route reducer and selected preview identities. The Overview breakdown remains local, ephemeral display state because it is not shared between sections. Insights sorting is a deterministic projection over its loaded section; it does not add persistent state. No new provider or state library is justified by this work.

**Technical debt:** The separate dialog/sheet implementations share spacing and focus-management patterns that warrant consolidation into a reusable accessible overlay primitive once browser validation confirms behavior. Modal focus trapping, Escape, focus restoration, body-scroll ownership and small-screen handling are not yet complete. Production selection handoffs, provenance, canonical IDs, chart lifecycle, history timezone and 10,000-row browser performance remain tracked independently.

## R584 seeded simulation and themed controls: ownership audit

`stats/vocabulary_seed.tsv` is a generated preview-only compact extract of the packaged HSK vocabulary dataset. `stats/simulator.rs` is its single parser and sampler: a seed-dependent full-cycle permutation through 11,470 unique lexical entries preserves reproducible identity without recycling the same 30 rows. The standard demo uses 240 sampled words and 292 session projections. Each session focus is chosen from that vocabulary; default word encounter, session and active-day totals are rolled up from those same sessions. The independent 550-entry grammar teaching index and editorial identities are unchanged. 5k/10k word stress scenarios reuse the seeded sampling contract, not the browser UI. Never expose these preview IDs as canonical learner identities.

`stats/controls.rs` now owns one feature-local `ChoiceSelect` component with typed `ChoiceOption`, controlled value and on-change callback. It owns only ephemeral popup/highlight state. Grammar collection and page-size choices still dispatch the route-scoped reducer; Vocabulary likewise. Do not move filter, page or selection state into a component-local hook. A later shared-ui extraction is warranted only after browser validation and proven reuse outside Statistics.

Long desktop Vocabulary/Grammar tables and Grammar cards use capped display regions while navigation and pagination remain outside those regions. This is presentation-only; the query layer still returns bounded pages and never loads a 10k-row DOM. Mobile acceptance must confirm that nested scroll is not a trap. Optional Overview tiles now have reduced visual verbosity without changing registry aggregation or visibility state ownership. No dependency or schema changes.

**Remaining technical debt:** verify ComboBox/ChoiceSelect focus restoration, click-away, key handling and mobile semantics; improve usage-based grammar taxonomy with approved editorial mappings rather than guessing; measure 5k/10k browser data-source performance; implement canonical Practice/Word Set transfer after identity approval; verify local time and Chart.js theme changes. Rust/WASM and Chrome acceptance were not available during this source handoff.
