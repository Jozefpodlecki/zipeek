# Statistics dashboard: product and interaction design

**Status:** approved design direction for the next implementation milestones. This is a normative UI/UX target, not a claim that the described interactions exist in the current browser build. R567 includes seven demo tabs and search/selection interactions, but is not a learning-record reader. Implementation is explicitly deferred until the design and its acceptance contracts are reviewed.

See [Statistics feature architecture](../../architecture/modules/statistics-dashboard.md), [Statistics design acceptance](../../testing-quality/e2e/statistics-dashboard-design-acceptance.md), [learning evidence](../../learning-practice/practice/learning-evidence.md), [personal storage](../../dictionary-data/personal/personal-data-storage.md), [interface policy](../foundation/interface-and-platform-policy.md), and [grammar courses](../../learning-practice/courses/courses-mosaic.md).

## Product position and success criteria

Statistics is the learner's **learning intelligence and action workspace**, not a decorative report or an opaque mastery scoreboard. Its four principal questions are:

1. What vocabulary, Hanzi and grammar have I encountered or practised?
2. Where is there recorded evidence, missing assessment or a credible need to review?
3. How has activity and performance changed over time?
4. What is the next meaningful action, and can I initiate it with the relevant items?

An acceptable experience makes provenance inspectable, does not overclaim learning, and works for both a beginner with no evidence and an advanced learner with 5,000+ word identities and thousands of sessions. The most useful discovery-to-action journey should take no more than a few deliberate steps. Users may inspect data without being forced into Practice or changing Word Sets.

The dashboard is not allowed to invent an overall mastery percentage or a single composite competence score. Historical Hanzi recognition acquisition, dynamic vocabulary proficiency, grammar-course completion and incidental study activity are distinct facts. A chart may visualize evidence but must not silently convert attempts into ability.

## Information architecture and navigation

Seven main sections are planned, with icon and text labels on desktop. On narrow screens, use a horizontally scrollable accessible tab strip or a labelled section selector, not squeezed text or icon-only navigation.

| Section | Question answered | Primary interaction | Production evidence dependency |
| --- | --- | --- | --- |
| Overview | What happened recently, and what is worth acting on? | Review a small shortlist; drill into detail | Qualified activity and evidence projections |
| Hanzi & Words | What is known about each identity? | Search, filter, inspect, select, add to set or practise | Stable word/Hanzi IDs and assessed observation provenance |
| Grammar | Which grammar patterns have been taught, attempted or assessed? | Filter patterns, inspect course/assessment, select and configure practice | Canonical grammar-point identity and attribution |
| Skills | What has been observed separately in each capability? | Inspect coverage and qualified outcomes, filter relevant items | Per-skill observation validation and assessor contracts |
| History | What occurred on a date or during a session? | Explore activity/calendar, open a session and its attempts | Durable session and attempt records |
| Insights | Which evidenced gaps merit attention? | Inspect a traceable reason, then create a practice draft | Deterministic insight rules over qualified projections |
| Settings | How should I view and organize the dashboard? | Choose tiles, layout presets, display options | Local UI configuration only; no assessment decisions |

Keep Statistics immediately after Browse in the main navigation. Default to Overview. The top of the page contains one clear header, a compact time-range selector when meaningful, the active section tabs and an explicit Demo marker only in preview mode. Do not repeat generic headings such as "Learning statistics" inside panels when context is already clear. Secondary explanations should live in accessible info controls or drilldowns; persistent visible text is reserved for actual uncertainty, consent and error states.

## Overview composition

The landing viewport should be intentionally curated rather than a full evidence table. Use an adaptive grid with these regions:

1. **KPI strip:** three to four primary cards by default, each with an understandable denominator, date scope, and inspectable definition. Suggested cards are Words encountered, Completed Practice sessions, Active days and Qualifying observations if the latter is actually available. Word-session pairs is a valid technical aggregate but must be opt-in and relabelled **Vocabulary session appearances**. It counts unique `(word identity, completed session ID)` combinations; one session can contribute to many words. It is not a number of sessions and not a proficiency score.
2. **Activity chart:** weekly sessions or eligible learning observations for a selectable time window, with a complementary accessible table or textual summary. Never use a chart to fabricate activity or assessment. If activity is unavailable, show a concise empty state and a link to Practice, not zero-valued invented bars.
3. **Skills snapshot:** a compact explanation of coverage across independently evidenced competencies, optionally with a link to Skills. A stacked visual is acceptable only if the categories and denominator are explicit.
4. **Actionable shortlist:** up to four or five items with a factual reason (for example, no qualified listening observations), a clear inspect action and an explicit path to Practice setup. Do not surface an automated "weakness" claim merely because a skill was never tested.

Do not paginate a large table on Overview. Link to the full filtered Hanzi & Words page instead. Prefer a balanced composition: visible KPIs, a chart and the beginning of the shortlist at common laptop heights, no giant blank chart or uncontrolled nested scroll. Ordinary document scrolling is allowed, but the default view should not require scrolling to discover its primary purpose.

An advanced user's all-time encounter counts can be large. Use grouping, filters, truncation with tooltips and compact numeric formatting without losing the exact value on focus or drilldown. Cards must never use a highly ambiguous metric label merely because it is shorter.

## Hanzi & Words: scalable evidence explorer

This is a proper data explorer, not a decorative table. It separates **Hanzi identities** from **whole-word identities** using a clear type filter or segmented view; a word's evidence does not automatically prove its constituent Hanzi. Present simplified/traditional form as relevant, Pinyin, short meaning, HSK/category metadata where sourced, distinct sessions, days, encounters, last relevant activity and per-competency evidence. Column visibility can be configured without losing access to meaning or identity.

### Evidence-state vocabulary and inspection

| Display status | Meaning | Visual language |
| --- | --- | --- |
| Observed | At least one valid observation exists for that identity/competency, not a pass | Compact check-style semantic glyph with accessible text |
| Review indicated | Qualified evidence currently supports a review action under a published rule | Attention glyph and a theme-aware review token |
| No evidence | No qualifying observation is recorded | Neutral dash glyph with accessible text |
| Pending assessment | Data exists but assessment provenance or thresholds are not sufficient | Neutral clock/question glyph, not success or failure |
| Assessed (future) | An approved assessor rule actually produced an assessment | Distinct badge with date, threshold and provenance accessible |

A tooltip or details panel must reveal observation count, last timestamp, assessment kind, assistance level and source session/attempt identifiers when actually available. The display must distinguish "no observation" from "learner failed" and "not yet assessed" from "not proficient". Theme tokens, icons and text must reinforce one another; color alone never carries the result. Avoid repeating long cell phrases across thousands of rows.

Writing is a fifth **display category for illustrative observations**, not an approved fifth requirement of vocabulary proficiency. The approved mandatory competencies are recognition, listening, assessed speaking and appropriate sentence use. Writing's validator and eligibility remain unresolved; mock entries must not promote it to a mastery condition.

### Search, filter, sort and pagination

- Search exact Hanzi, traditional/simplified variants, Pinyin and English meanings as supported by indexed data, with clear query semantics; do not promise fuzzy search without an implementation.
- Filter by identity kind, HSK/course, Word Set, evidence state, individual skill, date interval and assessed-versus-unassessed provenance when present. Filters are composable and show removable active-filter tokens.
- Sort by stable, typed columns with a deterministic secondary key (stable identity ID) so entries cannot jump between pages. Display current sort order to assistive technology.
- Support page sizes 10, 25, 50 and 100 on the full explorer, with page number and range, first/previous/next/last where appropriate. Clamp page index after filtering and page-size changes. Empty results show a real empty state.
- **Prevent last-page layout jumps:** keep the table surface at the established viewport height using an appropriate min-height or layout reservation. Reserve presentation space rather than insert phantom semantic table rows. On mobile use natural scrolling and a usable compact row layout when horizontal tables are impractical.
- At 5,000+ identities do not mount every row at once. Paginate or window a projection and profile DOM update cost before adding virtualization. Avoid using array indices as persistent row identities.

### Bulk selection semantics

Selection is an explicit UI state keyed by stable *typed identity IDs*, not row indices or display strings. It persists across pages and sorting within the same dataset/filter version. Support select visible page, clear selection and explicit "Select all matching results" with the count stated before confirmation; selection across 5,000+ matches must not happen accidentally. Show the number selected and whether selection is across pages. Changing to an incompatible identity type or dataset invalidates selection with clear notice; filtering should not quietly discard already selected identities. Selection must be keyboard accessible, and shift/range shortcuts are optional.

The contextual selection toolbar is the main action surface:

- **Add to Word Set:** choose or create a set, review duplicates/exclusions and confirm; do not silently write.
- **Practise selected:** navigate to Practice setup carrying an explicit validated selection and its origin. Preselected items are a draft, not an auto-start; validate availability, exclusions, max session size and supported exercises. Do not copy 5,000 words into a URL or bypass the existing session setup contract.
- **Inspect / export (future):** only when relevant data and privacy contract support these actions. No fabricated CSV of assessor data.

When selection is empty, actions are disabled with accessible explanation available on demand. Action controls should not crowd every table row; row click opens inspection, while bulk actions live in the selection toolbar.

## Grammar: first-class evidence explorer

Grammar evidence must not be absorbed into the vocabulary table. A grammar pattern row identifies a canonical point ID, authored title/pattern, course and HSK association where verified, instructional progress, assessed attempt count, latest activity and evidence/assessment state. Filter by course, level, teaching status, date and assessed performance. A selected row reveals examples, explanation, course link and provenance of attempts, distinguishing assisted instruction from independently scored assessment.

Grammar selection supports the same explicit visible/all-filtered semantics as vocabulary, keyed by grammar point IDs. **Practise selected** opens Practice setup with Sentence Transformation preselected *as a draft* and the chosen grammar IDs preserved in typed state. The user's selected/default Word Set remains the initial vocabulary source unless they choose another. Compatibility checks must occur before Start, including required word coverage, lesson readiness and exercise constraints; unsupported combinations provide actionable errors rather than silently broadening scope. Never auto-start upon table selection. Link a taught pattern to its course but do not imply it has passed a scored grammar assessment.

The grammar assessment contract in [learning evidence](../../learning-practice/practice/learning-evidence.md) and the approved rolling-window decision remain authoritative. The dashboard may project them only after relevant identifiers, assessed attempts and persistence are verified.

## Skills: five observations, four approved proficiency competencies

Skills should answer three different questions rather than show five uniformly labelled "Observed" bars:

- **Coverage:** which identities have any qualifying observation of each kind?
- **Performance:** for validated, scored attempts, what was correct/incorrect and under what assessor, assistance and time window?
- **Evidence gap:** what was not tested or needs a reasoned review?

The five presentation families are Recognition (eye icon), Listening (headphones), Speaking (mic), Writing (pen), and Sentence use (text/quote). Each summary includes a distinct icon, clear numerator and denominator, an accessible distribution visualization, a short recent trend where provenance exists, and a "View items" drilldown that transfers the correct filter to the explorer. A count such as `26 / 30 observed` means 26 identities have sample recognition observations, **not** 26 words mastered or 26 successful assessments. Never label unassessed coverage as performance.

For fixture design show varied coverage and missing states across five skills rather than repeating identical status text. In production, speaking assessment is unavailable until the assessment contract exists; a stored recording alone is not evidence of speaking correctness. Writing is illustrative only until explicitly approved as an assessed requirement. Charts show unknown/missing separately from observed and review-indicated; they must not force mutually exclusive colors onto logically overlapping categories.

## History: chronological explorer and calendar

History should offer two interoperable views: **Timeline** (default, approachable) and **Table** (dense, filterable). A date-range selector, activity calendar/heatmap, exercise-type filters and query navigate *actual persisted* completed sessions. Opening a session reveals date/time (with timezone), configured Word Set/vocabulary source, exercise types, scored attempt counts, context, optional per-item breakdown and valid evidence links. Daily activity is not the same as qualifying proficiency days until a documented timezone policy is implemented.

The timeline groups sessions by local date without assuming a timezone for future production data. Activity badges identify session types; avoid decorative repetition. Pagination uses stable height and valid date order. A calendar cell is focusable and its details are available without hover. If a privacy preference disables recording, the production History view must not quietly manufacture records or label a lack of records as lack of practice.

## Insights: explainable next actions

Insights are derived **only from attributable evidence**, not generated personality judgments or vague recommendations. Three families are planned: coverage gaps (observed one skill, missing another), validated performance trends (repeat assessment errors when scored) and actionable review groups (criteria and dates visible). Every insight has a short title, factual explanation, denominator/time period, source drilldown, and a clear optional action to create a Practice draft.

Absence of listening observations supports "Listening has not been assessed". It does **not** support "You are bad at listening". Do not recommend a practice threshold or "due" date until the scheduling policy is approved. Explainability should remain available by keyboard and assistive technology. With no authorized records, show a single honest empty state, not a full gallery of zero-based insights.

## Dashboard Settings and customization

The current four theme-aware switches do not scale to 20 to 30 tiles. The target is a searchable **tile library** grouped by Activity, Vocabulary, Competencies, Grammar and Review. Provide compact toggle controls with clear labels, a visible selected count, and a live layout preview. Support recommended presets (Learning, Practice, Detailed), restoring the default, and accessible reorder controls. Arbitrary drag-and-drop is not required for the first acceptance milestone; any drag feature must have a keyboard equivalent.

Keep content definition separate from presentation configuration. A tile definition has a stable `TileId`, category, minimum supported dataset capabilities, compact/full span, optional explanation and renderer identity. The user-selected tile layout is typed and validated: no duplicates, recognized IDs only, bounded count, mandatory identity/display rules enforced. Settings should never silently change assessment thresholds, sample data, learner records or the Word Set used in Practice. In the design phase the layout is ephemeral; future persistence requires a separately approved privacy/versioning decision.

## Preview, loading, empty states and privacy

The existing preview is opt-in. It uses deterministic fixture data in component memory, does not access the personal DuckDB, pedagogy worker or network, and must not persist into user learning records. Keep **one** discreet `Demo` indicator with an accessible explanation that values are invented, not personal and not assessed. Do not repeat lengthy notices in each section. Default production Statistics remains an honestly unavailable view until the evidence pipeline is connected.

The requested two-second preview delay is a **demonstration only**. Use dashboard-shaped skeletons reflecting the actual KPI/chart/shortlist layout instead of a blank card and center spinner. Skeletons are decorative to screen readers; expose one meaningful busy status and one completion announcement. Respect reduced motion and unmount cleanup. Cancel a pending timer when preview closes or the route changes. Production data must never incur an artificial delay. Error states need a retry action; empty states need a next step when one exists.

No fixture data or sample indicators may be sent to optional AI providers or written to storage. An AI-authored insight concept is not approved for production data. Do not introduce background analytics, telemetry, remote learner profiles or external data submission without separate approval.

## Theme, layout, responsiveness and accessible interaction

The violet/Zipeek design system, typography, tokenized surfaces and landscape background remain. Charts use the approved pinned Chart.js library through `zipeek-browser` Rust newtypes and thin interoperability, not a new JS application service. Charts must have readable labels, alternate accessible data and reduced-motion handling. Theme contrast must be checked for light, dark and system mode; evidence colors are semantic and never the only signal.

Desktop uses a readable dashboard column and space-efficient multi-column panels. Tablet collapses gracefully; mobile uses single-column cards and an overflow-safe tab selector. Table columns must not shrink to illegibility. Avoid nested vertical scroll traps, fixed screen-height calculations that hide focus, and huge empty spaces on short lists. Keep focus visible and return it sensibly after dialog, tab and pagination operations. Buttons should have appropriate touch targets, even where only an icon is visible.

## Design stages and non-goals

**Stage A: foundation and Overview.** Typed reducer/context boundaries, stable tables and pagination, skeletons, one real fixture activity chart, curated metric tiles, clear error and accessibility states.

**Stage B: deep exploration.** High-volume Hanzi/Words explorer, bulk selection and Practice draft handoff, first-class Grammar, richer Skills, History timeline/calendar, explainable Insights and scalable tile Settings. All use deterministic, wholly fictional fixtures.

**Stage C: visual and behavioral acceptance.** Verify root and `/zipeek/`, dark/light/system, keyboard/assistive tech, responsive widths, 5,000+ fixtures, stable pagination and performance. Do not change the production learning-evidence schema merely to support a demo.

No live proficiency calculations, schema migrations, persisted demo data, mandatory Writing assessment, automatic lesson scheduling, inferred mastery or auto-started sessions are in scope for the design milestone. Real personal data integration is a later gated project, not a side effect of preview development.

## Open policy dependencies

Before production evidence integration, settle the per-session encounter cap and retry rules; per-skill sample thresholds and assessor validation; the recording/retention/export/delete contract (especially audio and free text); the learner timezone policy; vocabulary identity continuity; and validated grammar-attribution rules. Preserve these decisions in the project task register rather than hard-code new values. The UI can design honest placeholders now, without pretending those policies have been decided.

## R567 design implementation notes

The demo Vocabulary explorer supports text search over Hanzi, Pinyin and meaning, deterministic sorting, ten-row pagination and page-level selection. Selection is deliberately retained when changing pages or the search query, with a separate review step. A clear-selection action prevents users from confusing a filtered view with their whole selection.

The Grammar explorer uses 16 illustrative pattern identities and sample encounter counts for layout and pagination design. These are not authoritative grammar-point IDs or validated learner attempts. A typed integration with the course registry and Practice builder is still necessary.

The Skills panels distinguish observed, review and absent sample activity. Their action opens the corresponding filtered vocabulary explorer. History offers a compact activity strip and expandable session details. Insights show traceable fixture-based counts rather than generated learning claims.

A temporary review dialog displays the current selection and explains that opening Practice setup does not transfer demo IDs. It must not be presented as a completed bulk Practice handoff. The Dashboard's Chart.js activity panel has a visible graphical fallback to prevent a blank chart if module initialization fails.

## R568 scoped UI implementation: presets, paging and selection safety

The fixture-only dashboard adds three in-memory Overview card presets (Focused, Activity and Detailed), while retaining individually configurable cards. Presets alter presentation only and do not create or assess observations. Vocabulary page sizes are 10, 25, 50 and 100 rows and changing the page size resets pagination to the first page. This is a UI-scoped setting, not a saved personal preference.

For vocabulary and grammar explorers, **Select all matching** explicitly selects the current filtered results, not all records in the project. It is bounded to at most 250 matching identities; above the bound the control is disabled and the user must narrow the query. Selections survive page and sort changes until explicitly cleared or the preview is restarted. The selection review displays a bounded sample and the actual count, never thousands of labels. This bound applies only to the current fixture preview; a future production projection may instead use a validated server/worker-side selection descriptor and review step.

No actual Word Set insertion or Practice setup selection transfer is implemented by these preview changes. Sample words are labels rather than verified dictionary identities, and sample grammar IDs are not canonical curriculum IDs. A production handoff requires the approved typed identity and compatibility contract, explicit review and no silent auto-start. These requirements remain open.

## R569 fixture-only exercise breadth and controls

The Statistics History demonstration draws its exercise identities from the current `zipeek-exercises::definition::DEFINITIONS` registry (sixteen Practice types) and adds a separate catalogue of representative game surfaces. Each session has a stable fictional ID, date, exercise identity, category, focus item and attempt count. Practice-only exercises have **no scored correctness field**; speaking recordings, Write Hanzi, Spot the Mistake and Sentence Transformation are never silently represented as qualified scored proficiency evidence. Game results in the preview are illustrative gameplay counters only, not assessed vocabulary competencies. This catalogue is not evidence that each route currently persists history.

History filters by Practice/Games and search text, preserving stable pagination. Fixture sessions are intentionally repeated across dates so the seven-day activity display and the Overview time series can demonstrate varying activity; none are stored. The other six tabs continue to render from the same in-memory preview without network or personal database access. Acceptance must inspect that filter, pagination and tab switching work in Chrome; source-level wiring alone is not browser verification.

The interaction policy now requires shared theme-integrated inputs with accessible semantics. The preview implements a reusable checkbox-style selection control in the Statistics feature while leaving full custom select/listbox behavior unimplemented until its keyboard contract is verified.

## R570 implementation checkpoint: independently loadable sections

Each of the seven preview tabs is now loaded through an asynchronous section source instead of synchronously importing fixture arrays inside the corresponding UI. The panel has an accessible, shape-preserving loading state; a failed load has a contextual retry action; an empty successful response stays a valid empty view. Changing tabs cancels ownership of a pending view request and previously loaded panels remain available in the preview cache. All values are still fictional, non-assessed and in-memory. No production learning evidence has been connected.

The client-side explorer filtering used by the finite demo remains intentional for now. Remote, high-volume evidence requires a later approved query/pagination contract and per-item identity validation. The current interface should not be advertised as server-paginated or as a live learning dashboard.
