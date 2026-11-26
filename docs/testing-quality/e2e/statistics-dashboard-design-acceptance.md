# Statistics 2.0 design and acceptance matrix

**Status:** approved documentation-first requirements, pending implementation. These checks apply to the proposed Statistics redesign and do not imply that the current R564 preview passes them. Pair with the [product specification](../../product-ux/pages/learning-statistics-preview.md), [architecture](../../architecture/modules/statistics-dashboard.md), [evidence requirements](../../learning-practice/practice/learning-evidence.md), and [existing E2E strategy](e2e-architecture.md).

## Acceptance strategy

Use deterministic fictional data first. Test pure reducer/projection functions independently, then Yew/WASM build, then browser acceptance with measured viewport sizes and keyboard interactions. Do not mark a requirement verified from a screenshot, authored test, static source review or ZIP integrity alone. Distinguish "implemented", "compiled", "E2E passed" and "browser visually accepted".

### Fixture families

| Family | Minimum deterministic coverage | Purpose |
| --- | --- | --- |
| Empty | 0 words, 0 sessions, 0 patterns | Honest empty states, no fake cards, paginator bounds |
| Small | 1 to 8 words, 1 to 3 sessions | Last-page stability, short labels and small cohorts |
| Medium | At least 30 words, 20 sessions, 15 grammar patterns | Multi-page sections, semantic cell variation |
| Large | At least 5,000 word identities, 1,000 sessions and 200 grammar patterns | Performance, filter/sort, selection across pages |
| Stress | At least 10,000 words with long meanings, mixed Hanzi/traditional, missing metadata | Responsive overflow, identity stability, memory pressure |
| Timing | Simulated 2-second preview, cancelled navigation, failed/retried request | Async lifecycle, skeleton and loading |

Fixtures are fixed-seed, reproducible and include multi-session overlaps, duplicate attempted deliveries, date-boundary cases, missing skills, review-indicated facts and distinctly unassessed speaking/writing samples. They contain no user data, credentials or raw personal audio. If the test harness injects a clock, use the same explicit fake clock at domain and storage boundaries. Do not modify the operating-system time.

## UI acceptance checklist

| ID | Area | Acceptance condition |
| --- | --- | --- |
| ST2-001 | Demo source | Preview is opt-in, visibly marked Demo once, does not read/write the personal evidence store, never awards proficiency |
| ST2-002 | Default state | Without verified ledger, live Statistics shows truthful unavailable state and no fictional metrics |
| ST2-003 | Loading | Two-second Demo loading renders realistic dashboard skeletons, one screen-reader busy status, no extra permanent text |
| ST2-004 | Loading cancellation | Closing or navigating away invalidates pending completion; stale callback cannot re-open preview |
| ST2-005 | Layout | On common desktop/laptop, metric cards, chart and start of actionable shortlist are discoverable; no giant default table |
| ST2-006 | Responsive | Tablet/mobile tabs, charts, tables and toolbars remain usable without clipped actions or vertical nested scroll trap |
| ST2-007 | Themes | Light, Dark and System have semantic contrast, visible focus, readable chart labels and no color-only evidence |
| ST2-008 | Reduced motion | Charts and skeletons honor reduced-motion preference; loading state remains understandable |
| ST2-009 | Overview KPI | Every card has a precise definition, population/window and drilldown; word-session aggregate is not mislabelled sessions |
| ST2-010 | Charts | Chart.js draws source-consistent sample series, cleans up on unmount and has accessible tabular/summary equivalent |
| ST2-011 | Tabs | Seven sections including Grammar; keyboard and assistive-tech navigation, active-state feedback and focus management |
| ST2-012 | Evidence cells | Observed, review, absent, pending and assessed (only when qualified) have accessible labels and sourced drilldowns |
| ST2-013 | Search/sort/filter | Compound filters, stable tie-break sorting and item identities survive pagination; reset/clamp after filter changes |
| ST2-014 | Pagination | Full tables support 10/25/50/100, valid zero-result range, correct last page and stable paginator position |
| ST2-015 | Bulk selection | Select visible, clear and explicit all-filtered scopes; selected IDs stable across sort/page; large-scope confirmation |
| ST2-016 | Word Sets | Add-to-set prompts for destination and confirms duplicates/exclusions; no surprise mutation |
| ST2-017 | Practice | Vocabulary/grammar selection opens validated Practice setup draft, never launches exercise automatically |
| ST2-018 | Grammar | Grammar IDs/courses/assessment states remain separate from vocabulary; selected patterns preselect Sentence Transformation |
| ST2-019 | Skills | Coverage vs assessed performance vs missing evidence are distinct; Writing remains non-mandatory until approved |
| ST2-020 | History | Calendar/timeline and table agree on session counts, dates and details; no inferred history when recording disabled |
| ST2-021 | Insights | Every recommendation has evidence scope and reason; missing tests never become a false poor-performance claim |
| ST2-022 | Settings | Search 20 to 30 tile definitions, presets, toggles, keyboard reorder, reset and config validation |
| ST2-023 | 5,000+ records | No unbounded row DOM, controls remain responsive, selection correct and memory/render benchmark recorded |
| ST2-024 | Routing | `/` and `/zipeek/` base paths, navigation away and return, refresh and optional query-state semantics verified |
| ST2-025 | Errors | Actual data errors show clear recovery; empty state is distinct from loading, failure and privacy-disabled state |
| ST2-026 | Privacy | Fixture never reaches OPFS/DuckDB, telemetry or AI; absence of stored data is not mistaken for learner inactivity |

## Critical workflows to exercise

1. Open Stats with no verified ledger, enter Preview, observe dashboard-shaped skeletons, then close while loading and confirm no stale completion.
2. Enter Preview again, switch tabs, filter a 30-word dataset, navigate to its last page, change page size, verify no layout jump or invalid row range.
3. On large fixture, select a page, navigate, select all matching only after explicit count confirmation, sort and verify selected IDs remain correct. Open Word Set action but cancel; prove storage unchanged.
4. Select two words, open Practice setup with a selection draft, validate exclusions and supported exercise types, then return without starting; no session/evidence writes.
5. Select grammar points and open Practice setup with Sentence Transformation preselected. Verify grammar IDs are preserved, default Word Set remains selected and no automatic session begins.
6. In Skills, inspect an observed-but-unassessed Speaking item and verify it cannot be portrayed as mastered. Verify Writing has no automatic proficiency contribution.
7. Open History calendar then the same session in table mode, check count, stable session identity, fake-clock date boundaries and ordinary keyboard interaction.
8. Open an Insight and inspect the source evidence, then navigate to matching filtered items. Missing listening assessments must not say that a learner is weak at listening.
9. Configure a preset, toggle and reorder multiple tiles using keyboard, restore defaults and check that closing Preview clears the ephemeral configuration as documented.
10. In light/dark/system and reduced-motion states, inspect chart legend, semantic icons, focus indicators, tall translations, narrow widths, and mouse/touch handling.

## Measurable quality gates

- **Data correctness:** Fixture aggregates recompute deterministically; all displayed totals match the same underlying source and time window. No duplicate attempt double count.
- **Predictable UI:** Empty and final pages keep controls stable. Paginators never present a range starting at 1 when there are zero results.
- **Performance:** Capture actual baseline and post-change timing for first Dashboard render, pagination, filter, sorting and bulk selection on the large fixture. Do not assert a target was met without real measurement. Avoid long main-thread blocks and unbounded mounted rows.
- **Accessibility:** Use appropriate `tablist`/`tab` semantics or equally understandable navigation; headings form a coherent hierarchy; toggle and selection states are announced; charts have text/table alternatives.
- **Theme:** Semantic status labels are not dependent on raw fixed green/red colors and remain distinguishable in high-contrast conditions.
- **Privacy:** Verify by inspecting mocked storage/network calls that Demo does not read or write personal data and never invokes AI. The distinct live evidence source is blocked until the required policies are resolved.
- **Traceability:** Every tracked acceptance item records test command/result, browser and viewport when applicable, and failure artifacts. Do not close a gate based only on source implementation.

## Implementation-stage gates

**Stage A:** reducer/context contracts, deterministic fixture source, stable table/paginator, Overview chart and accessible skeleton loading. Unit tests, WASM build and representative Chrome flow required.

**Stage B:** vocabulary bulk selection/Word Set handoff, grammar, Skills, History, Insights and scalable tile Settings. Test source isolation, all-filtered selection and guarded Practice setup routing.

**Stage C:** full fixture matrix including 5,000+ items, visual review, keyboard/mobile, high contrast, reduced motion and both deployment base paths. Document measured performance and correct acceptance failures before any live data integration.

**Separate production evidence gate:** approved assessor thresholds, timezone, retention/consent, stable identities, durable write/readback and qualified observation projections must exist before using real learner metrics. This milestone authorizes no production schema changes or proficiency inference.

## R573 source-contract regression scenarios

- Simulate a source response whose `offset`, `limit`, `total` or number of returned records disagrees with the request. Expect a recoverable query error with Retry, not a misleading range or partial success.
- Simulate a populated page that disappears after a backend filter or dataset change. The explorer must navigate to the last valid page and requery it; a total of zero must display an honest empty range.
- Request Select all matching twice in rapid succession, change the search while a request is pending, and Clear before completion. Only the current matching scope may affect the selection; no stale request may restore cleared IDs.
- Check all seven tabs after the R572 Windows delimiter failure is corrected. Passing static source checks or ZIP integrity is not a substitute for compiling and running the browser scenario.

## R575 implementation evidence and outstanding gates

The source adds a twenty-entry typed optional tile registry alongside four core Overview metrics. Reducer tests cover visibility, order transitions, preset replacement, reset and preview-close reset. Aggregation tests verify total sessions and Practice/Games and scored/unscored partitions over the fixture. The Stats E2E scenario now exercises visibility reflected on Overview and move controls, but the browser test has **not** run in the packaging environment. The acceptance gate ST2-022 remains pending Windows WASM/browser confirmation, light/dark/mobile keyboard review and 5,000/10,000-record measurement. Local source-level assertions are not performance measurements.

The vocabulary and grammar selection columns now reserve an explicit 56px width with padding for the 36px hit target, to prevent overlap with adjacent row labels. Inspect both table widths in a real browser.

## R576 source implementation and verification targets

The source adds a reducer regression for History view switching and date-search pagination reset, and a query regression asserting that a selected date restricts session rows while preserving the seven-day summary. The History UI exposes `data-stats-history-view="timeline"` and `data-stats-history-view="table"` for browser assertions, plus `aria-pressed` on both view controls and seven-day date buttons. Verify that both modes show the same session counts, that a selected date filters the list without hiding other days from the chart, that clearing the date preserves the selected display mode, and that unscored Practice activities show **Not scored** in the table.

Skills must be inspected for distinct activity, review, missing-record and assessed-unavailable explanations. Insights must link the four required competency gap categories and the review category to the appropriate vocabulary query. All coverage figures remain illustrative, not graded outcomes. These are authored source contracts only, not completed browser tests. ST2-019, ST2-020, ST2-021 and the full Stage C quality gates are still unverified.

### R577 Grammar disclosure test supplement

- With the mock Grammar tab loaded, activate a pattern's Inspect control using mouse, Enter and Space; verify `aria-expanded` and the detail row, sample sentence, course category, illustrative attempts and explicit unavailable canonical/assessed identities.
- Expand one pattern, switch pages or search, and verify the disclosure closes without losing selected patterns. Reopen a record and verify its stable preview identity.
- Review selected grammar patterns and confirm the independent Practice link says it does not carry the selection. No Word Set, learner ledger or session writes may occur from this demo flow.
- A production adapter must independently verify canonical grammar/course identifiers and assessed provenance before presenting those fields as authoritative. This is not satisfied by fixture labels.

### R578 density, timestamp and fixture acceptance (pending execution)

- Confirm every Timeline row and History Table row displays a UTC time and fictional duration, not merely a date; UTC must be explicitly identified until the approved local-time formatter exists. Confirm descending full-timestamp order and tie-breaking identities.
- Confirm the 156-session fixture includes all 16 registered Practice types and eight demo Games, with Practice/Games interleaved on the newest page. Scored and unscored activities must retain their original assessment contract.
- Confirm Overview chart's weekly buckets agree with the optional compact Practice/Games breakdown; `Weekly breakdown` should be collapsed by default, have a visible keyboard focus treatment, meaningful headers and no raw unstyled debug table.
- Confirm Insights and Skills no longer repeat long evidence-policy disclaimers. One Demo indicator plus specific metric labels and tooltips is sufficient; absence of recorded activity still cannot be presented as failure or proficiency.
- Check long tables at 768px laptop height, 375px and 425px widths, and both themes. Primary selection actions and pagination remain available; overflow must not obscure Chinese content or column headings.
- Validate source query behavior with timestamp search, date filter, zero matches and async page changes. Check the seven-day activity summary remains independent of the paged History records and returns the appropriate count per category, and that every daily stacked segment adds up to the daily total.
- Re-run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and the existing Statistics E2E on Windows. This list is acceptance work, not completed test evidence.

## R580 added acceptance scenarios

- When opening Statistics preview, the header exposes a named icon control, without duplicating a Demo badge in the page body.
- Selecting the first row keeps the explorer controls and table position stable. Clear and Review remain keyboard reachable with accessible names.
- Select-all-matching is above the table and limited to at most 250 identities. Queries above that bound cannot be bulk selected.
- Vocabulary sorting from Hanzi and Seen column headers reverses the corresponding ordering, exposes `aria-sort` and keeps identity tie-breakers deterministic.
- The Grammar query source returns 550 unique records from the maintained teaching index. Check a non-first page, source-backed search suggestions, editorial status in details, and no invented assessed attempts.
- Overview's two primary cards align in height at desktop widths. History search and category filters remain usable at laptop and narrow viewports.
- Measure, do not assume, light/dark, keyboard, reduced-motion, screen-reader, 5,000/10,000 identities and both root/deployment routes.

## R581 Grammar query acceptance (not yet executed in browser)

- Verify region filters All, 1-20, 21-40, 41-60, 61-80 and 81-100 are accurately reflected in total counts and result IDs. The 81-100 fixture range contains 100 teaching entries.
- Sort Pattern twice to reverse order; sort Attempts twice to reverse numerical order. Check `aria-sort`, identity tie-breakers and first-page reset without losing intentionally selected IDs.
- Change Rows to 25, 50 and 100; inspect page count, the last short page, zero results and first-page reset.
- Search while on a later Grammar page and verify the suggestions come from a separate source query, never only that later page; inspect keyboard focus and menu dismissal behavior.
- Verify all-matching selection respects the active region and search and declines totals above 250; Clear must invalidate an in-flight bulk operation.
- Run locked Windows Rust native/WASM builds and the Statistics Chrome E2E; authored tests do not certify browser acceptance.

## R583 Grammar and overlay scenarios (awaiting browser execution)

- Grammar table and cards display the same bounded paginated source records and identical matched totals. Switching mode does not reset search, region, sort, page, page size or deliberate selection.
- Selecting a Grammar pattern and opening its inspector leaves the table row geometry unchanged. The inspector uses only the selected page record and does not manufacture translations, course IDs or assessment counts.
- Opening selection review after selecting across multiple pages does not append a long panel below pagination. Returning closes it without losing selection.
- Opening Overview's weekly breakdown keeps the To review card at its original height, and accessible table totals match its chart projection.
- Insights ranks evidence gaps by count, using a deterministic tie-breaker, and its per-row actions open correctly filtered Vocabulary views.
- Chrome acceptance still requires Escape behavior, focus containment/return, small-screen and screen-reader tests, keyboard switch between Grammar layouts and responsive layout snapshots in both themes. Authored E2E assertions are not execution evidence.

## R584 acceptance additions (not executed)

1. Verify the default preview returns exactly 240 unique Vocabulary IDs and 292 session identities, that every session focus appears among the 240 words, and that Vocabulary sums of encounters and word-session appearances equal the History projection totals for the same fixed scenario.
2. Verify distinct 5,000/10,000-entry lexical identities; stable seed produces identical records, a different seed changes the words, and source-backed queries honor 10/25/50/100 page sizes and selection limits. These are source unit tests until run on Windows.
3. From Grammar, open Collection and Rows via keyboard and pointer, traverse with arrow/Home/End, select by Enter/Space, dismiss with Escape, Tab and click-away, verify focus returns after a selection, and test high zoom/mobile and both themes. Repeat Rows in Vocabulary. No success is claimed until Chrome and assistive-technology observation.
4. Check compact desktop tables keep sticky headings and the pager visible while content is scrolled; on tablet/mobile avoid clipped actions and vertical scroll traps. Test Grammar card density and the Settings category-filter chips against 24 metric definitions.
5. Assert preview entry/exit and Demo state remain unchanged, no fixture enters personal learner storage, and preview-generated text does not imply a verified course or assessed skill.

## R585 screenshot-driven browser acceptance additions

- At laptop widths, History search, activity categories and view toggle should share a coherent wrapping toolbar. Verify every icon action has an accessible name, title and keyboard focus treatment, and that the selected state is announced.
- A scored History row displays a correct/attempted ratio and a proportionate bar; unscored rows show no bar. Confirm this is session correctness, not a mastery, proficiency or completion percentage.
- Verify Vocabulary and Grammar paginate without competing 65vh internal scroll containers, including 10/25/50/100 rows and narrow touch viewports.
- Verify Settings full-tile label clicks toggle the correct registry item, while up/down reorder buttons are distinct, keyboard-reachable controls.
- Verify selected Vocabulary/Grammar records render actual preview names/meanings/examples, not raw IDs; removal and Clear all update reducer selection. Confirm no Practice navigation or Word Set write occurs from preview identities.
- Exercise selection-dialog initial focus, Escape, Tab/Shift+Tab containment, close and return-to-trigger focus with keyboard and a screen reader. The R585 source does not yet claim that complete focus contract.
- Current source-only delivery is **unverified** until the locked Rust/WASM tests and actual browser test matrix execute.
