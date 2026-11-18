# Statistics dashboard: architecture and state contracts

**Status:** approved Statistics 2.0 architecture, partially implemented through R567. `stats/state.rs` now owns coupled UI transitions, `stats/context.rs` provides route-local state, and `stats/overview.rs` owns the initial fixture-backed Chart.js plot and shortlist. The remaining structure below is the pending architecture plan, not an implementation claim. The product contract is [Statistics design](../../product-ux/pages/learning-statistics-preview.md); the acceptance plan is [Statistics design acceptance](../../testing-quality/e2e/statistics-dashboard-design-acceptance.md).

## Ownership boundaries

- `zipeek-web` owns dashboard composition, typed feature state, Yew context/provider and presentation. It does not independently compute proficiency or modify the authoritative assessment rules.
- Pure transformation/domain functions own sort, filtering, grouping, aggregate projection, selection semantics and insight derivation over validated facts. Start in focused modules; do not add a crate merely for speculative reuse.
- `zipeek-browser` owns browser API interop and Chart.js module/instance lifecycle through Rust newtypes. Use pinned Chart.js and the existing thin `wasm-bindgen` dynamic import boundary; no new JS application logic.
- Personal learning storage services, when authorized and browser verified, own transactional reads/writes and exports. Their future projection interface must distinguish stored Practice outcomes from qualifying proficiency evidence.
- Fixtures belong to a read-only deterministic `Demo` data source. They never touch storage, pedagogy worker, AI providers or network, and are not silently promoted to production data.

Follow [UI state ownership](../ui-state-ownership.md), [Rust/WASM interop](rust-wasm-interop.md) and [coding standards](../../engineering/coding-standards.md). No dependency additions are approved in this design.

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
