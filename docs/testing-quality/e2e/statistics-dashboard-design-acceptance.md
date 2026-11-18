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
