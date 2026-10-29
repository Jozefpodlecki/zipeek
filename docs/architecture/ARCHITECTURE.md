# Architecture

This file is the overview. Detailed module notes live under
[modules/](modules/); database and schema details live under
[schema/](schema/); UI state boundaries and motion proposals live in
[UI_ARCHITECTURE.md](UI_ARCHITECTURE.md).

## Repository snapshot

Described as of 2026-10-29 during an ongoing repository migration. This
describes files on disk, not the last committed release.

- Root `Cargo.toml` includes `web`, `crates/db-gen`, `crates/zipeek-core`, and
  `e2e`.
- `web` is the Rust 2024 Yew CSR package `zipeek`. `main.rs` initializes the
  environment, logging, and renderer; `lib.rs` declares the app modules.
- `web/src/` contains `app`, `components`, `contexts`, `env`, `models`, `pages`,
  `route`, `reducers`, `hooks`, `services` and `theme` modules. App mounts
  `ThemeProvider`, `DatabaseProvider`, `HashRouter`, `AppShell`,
  `Switch`, then `DatabaseBoundary` for recognized routes. The 404 fallback
  bypasses database readiness so unavailable data cannot hide route recovery.
- `route.rs` declares Home, Random, PracticeSetup, three match variants, Browse,
  three MCQ variants, and a `/404` fallback. Browse and Random remain feature
  placeholders; A11 implements setup controls and canonical Start navigation. Practice normalizes URL state against dictionary rows,
  loads candidate views and builds a deterministic round, with guarded
  Loading/Ready/Failed+Retry. Endless MCQ has answer/feedback/Continue controls;
  Matching and MCQ have Endless/Fixed progress/completion and shared summaries;
  A48 also supports mixed-type sessions without changing core round replay.
- Trunk configuration and HTML live in `web/`. Development port is `1420`.
  Styles live in `web/styles/`; HTML copies `web/public/` assets and the
  generated static bundle.
- D27 makes root `Cargo.lock` authoritative. Trunk enforces locked builds and
  explicitly watches web, zipeek-core and root workspace manifest/lock/toolchain
  files.
- `e2e` is a workspace library/executable with modular root and Pages suites.
  A15 owns hidden Trunk/Node children, dynamic ports, shared bounded readiness,
  one managed headless Chrome session per integration binary, scenario reset and
  diagnostic artifacts. Root serves `/`; Pages builds/serves `/zipeek/` with an
  isolated dist directory. No external server is required.
- `db-gen` has a modular verified-input preparation pipeline and the D21/D23
  native-engine/storage guards. It creates the approved words table and two HSK
  indexes, imports 12,100 primary readings in memory, applies empty-primary
  CC-CEDICT fallback, and exports a compatible artifact. `zipeek-core` supplies
  Word/WordId, HSK and PoS types.
- A06 opens the final dictionary through DuckDB-WASM 1.32.0 with matching local
  workers; all 12,100 rows pass read-only browser/Node checks.
- Neo is restored at its prior resolved Git revision. D23 requires actual engine
  v2.0.0 and verifies v1.4.0 storage compatibility on the output connection
  before any future dictionary file creation.

## Target architecture

The root Makefile.toml provides explicit local db-check/db-generate/db-install/
serve/dev tasks. scripts/db-workflow.cjs validates whole-bundle checksums and
pinned inputs, stages a complete copy, backs up web/static and rolls back a
failed publication. Generated candidates/backups stay in target/db-workflow.
Only explicit tasks generate data; Trunk hooks and rebuilds do not. A44/A46
verified the regenerated bundle through native export, HTTP bytes and one managed
Chrome dictionary/MCQ load. No new tool/package dependency was added.

The 2026-10-27 review requests stronger feature/provider boundaries and motion
support. [UI_ARCHITECTURE](UI_ARCHITECTURE.md) proposes scoped reducers/contexts,
an Animato adapter and a stable route-transition host. A31 implements scoped
theme/readiness reducers, stable typed dictionary access and named commands.
Motion and future feature reducers remain proposals, with no implicit dependency
or feature approvals. A15's implemented modular e2e harness is documented in
[testing/e2e.md](../testing/e2e.md#implemented-layout-and-ownership).

D31/A35 implements Tailwind-first component styling. Complete utilities live in
mounted Yew components, with explicit Rust source detection, shared theme-token
aliases and focused critical bootstrap/loading CSS. The unused v3 configuration
and bespoke shell/card selectors are removed. Shell cleanup/responsive defaults
are implemented; A31 state boundaries are implemented and motion remains proposed. See
[UI_ARCHITECTURE](UI_ARCHITECTURE.md#tailwind-styling-implementation-d31--a35).

Proposed browser database crate extraction (A33, D30): see
[modules/target-architecture.md](modules/target-architecture.md#proposed-browser-database-crate-extraction-a33).
Proposal only; requires explicit approval before implementation.

## Runtime data flow (summary)

1. A developer manually runs the native `crates/db-gen` binary against approved,
   licensed sources.
2. The generator produces `web/static/hanzi.duckdb`, which is committed and
   deployed with the static app.
3. DuckDB-WASM opens that file read-only in the browser.
4. Query results drive browse pagination, Word of the Day, and practice rounds.

Full flow, browser query modules, startup handoff, and shell details are in
[modules/data-flow.md](modules/data-flow.md), [modules/browser-db.md](modules/browser-db.md),
and [modules/web-shell.md](modules/web-shell.md).

## Modules

| File | Contents |
| --- | --- |
| [modules/db-gen.md](modules/db-gen.md) | Native generator module map, verified-input pipeline, import stages |
| [modules/zipeek-core.md](modules/zipeek-core.md) | Shared domain types, comparison/fallback boundary, practice URL, pure round builder |
| [modules/browser-db.md](modules/browser-db.md) | A06 browser query modules, runtime/row/query separation, e2e harness status |
| [modules/web-shell.md](modules/web-shell.md) | Startup handoff, loading shell, A27–A30 shell, landscape composition |
| [modules/data-flow.md](modules/data-flow.md) | Runtime data flow, dictionary artifact and Pages serving |
| [modules/target-architecture.md](modules/target-architecture.md) | Target architecture, A33 crate extraction proposal, workspace graph |

Database/schema details (sources, DDL, import rules, PoS mapping, queries, runtime
pin, generator dependencies, reproducibility, artifacts, regeneration) live under
[schema/](schema/). Start at [schema/DB.md](schema/DB.md).

UI state and motion proposals live in [UI_ARCHITECTURE.md](UI_ARCHITECTURE.md).

## Development phase

During development, breaking code/data changes and regeneration are acceptable;
backward compatibility and application/database versioning are not required.
This does not authorize changing pinned dependencies without approval (D10).

Round-seed replay is a narrow exception: D19 keeps v1 as the only format; a
future v2 requires explicit approval.

## A21 route integration — 2026-10-28

Practice loading lives in services/practice_loader.rs, hooks/practice.rs and
reducers/practice.rs. The page renders the scoped loading snapshot and Retry
command; data queries and build_round stay outside render. Six variant routes
share the pipeline and active Practice navigation semantics. AppShell updates
document.title and focuses main on route changes, including Back/Forward, while
query-only changes leave focus alone for future input controls. The 404 view uses
a router link instead of an unchecked navigator context and retains shell sizing.
No route-transition animation, gameplay, Browse search, Random shortcut or daily
selection is implemented by this integration.

## CI and publication (A16)

ci.yml validates PR/push/manual runs on Windows with the approved toolchain and
all five gates/owned Chrome suites. Successful non-PR master runs call deploy.yml
as a reusable Ubuntu release build and Pages deployment, using web/ and /zipeek/.
See [CI](../process/CI.md) for permissions, caches and remaining hosted validation.
No package or runtime architecture changes were made by the workflow migration.

## Practice setup (A11)

PracticeSetup owns a local pure SetupState reducer; ActivityPicker, HskPicker and
SetupChoice are presentational leaves with immutable inputs/named callbacks.
Six activity tiles lead through Route::practice and the existing router query
serializer. Start emits core query_pairs with offset zero, no seed/PoS, and the
validated saved exclusion set. Initial selection is Hanzi to meaning for the
user's quiz-first priority, HSK 3.0 level 1, Endless; Fixed defaults to 10.
Valid numeric levels survive system changes; HSK 3.0 advanced resets to level 1
on switching to HSK 2.0. Selected Fixed count survives an Endless toggle.

hooks/exclusions.rs owns asynchronous saved-ID validation using the stable
Dictionary reader, generation checks and abortable orchestration. The storage
key is zipeek.excluded_words, a JSON array of integer WordIds. Parsing deduplicates
valid typed IDs; one bound batch lookup removes unknown IDs before count/Start.
No storage normalization or writes occur on load. Reset explicitly removes the
key and clears the local set; setup stays visible and the next Start URL omits
exclude. Query failures expose Retry and keep Start disabled. Reset failure keeps
the count/storage and shows an inline error. Storage read failure/unavailability
logs and uses an empty in-memory set; malformed values only warn in debug.

The page is a scrollable full-width column with Tailwind utilities and native
fieldset/button semantics. There is no extra global filter provider or setup
persistence. Practice URL loads remain independent of saved exclusions. Actual
Matching and MCQ are playable in both modes, including mixed-type sessions.

## Endless MCQ milestone (A10)

McqPractice stays mounted through query Loading/Failed/Retry on an Endless MCQ
route. The A48 use_practice_session hook now exposes immutable projections plus
Select/Continue commands for both exercise families;
the pure McqState reducer owns the selected option, cumulative correct and streak.
Only the first valid answer counts. Duplicate/stale selections and Continue
before selection are ignored. Continue records the expected normalized next
query; only that resolved round retains counters. Other query/variant changes,
leaving the feature and reload start fresh. Pending requests retain existing
stale-response/cancellation guards. Scores are never persisted.

Practice preparation retains selected MCQ Word rows from a bound batch lookup
for secondary traditional display and pronunciation. Missing display rows fail
preparation. It also retains the exclusion-clean strict pool size for nominal
offset advancement; seeded context differing from the URL filter fetches the
next unseeded filter's pool. No core builder, comparison key or RNG changes.
Continue uses advance_offset and normalized query_pairs/router serialization,
clears seed, retains filters/exclusions and reports wrapping without resetting
session counts. The Fixed extension below reuses that session boundary.
When the strict pool is empty but widening succeeds, wrapping may leave the
normalized URL unchanged. Continue and same-offset replay explicitly request
preparation again so the new round resolves and the session does not stall.

ListenButton uses native speech synthesis, zh-CN, and the represented word's
simplified hanzi; errors log and show Audio unavailable. It is a sibling of
answer buttons, so listening does not select an answer. Number shortcuts ignore
editable controls, Listen, modifiers, composition and held-key repeats. New
questions focus their prompt. Feedback uses instantaneous colors and text icons;
there are no motion effects or live practice announcements.

## Fixed MCQ and summary (A10 / D35)

McqState also owns core FixedProgress, mistakes, highest streak, start and finish
timestamps. Ready starts wall-clock timing; expected Continue transitions retain
the timer/progress independently of offset wrapping. Continue increments completed
progress once; the final Continue freezes time and renders PracticeSummary without
loading an extra question. Stale/duplicate completion cannot update the result.
Reload/external navigation starts fresh. Randomness is used only by Play again:
existing getrandom supplies a new valid pool offset distinct from the current one
when the pool has multiple offsets (otherwise the canonical zero fallback is
reprepared as a fresh session);
ordinary shared URLs and round generation remain deterministic.

PracticeSummary has immutable stats and named restart/setup callbacks. Its chart
effect calls the opaque wasm-bindgen summary bridge, which lazily imports the
local vendor/summary/summary.mjs bundle relative to document.baseURI. The effect
destroys the chart on unmount and immediately destroys a late creation result.
Failure retains text and navigation. JS owns the chart and its 300 ms count/arc
animation, cancels frames/listeners on destroy and finishes instantly on a live
reduced-motion preference. Stats stay in DOM text; no scores are persisted.

The pre-build prepare-summary.cjs verifies exact chart.js 4.6.1 and esbuild 0.28.2
and bundles only Chart/DoughnutController/ArcElement with tree shaking. It copies
Chart.js, color and esbuild licenses and writes bundle metadata under ignored
vendor/summary, which the existing vendor copy directive packages. It never
installs packages or changes the dictionary. No CDN, runtime bundler, scales,
tooltip or legend. A09 reuses the summary for its approved pair metric.

## Matching and mixed session ownership (A09 / A48)

Practice owns one SessionState reducer via use_practice_session. Its ActiveRound
enum holds the current matching or MCQ reducer state; only the active round exists.
Interaction rules remain pure in reducers/matching.rs and reducers/mcq.rs. Typed
expected navigation retains shared correct/first-attempt/mistake/streak, highest
streak, FixedProgress and wall-clock fields on the next resolved round, even when
its family changes. Reload or an external query resets the session. A generation
counter and key/serial guards reject stale matching selections and flash timers.
Named callbacks/immutable projections reach the feature views; no raw dispatch,
JS/database handles or global session provider escapes this boundary.

MatchingPractice and MatchCard retain eight cards in their shuffled positions.
Same-side alternate selection is ignored; repeat selection/Escape clears. Wrong
attempts lock matching input for 400 ms and disqualify both WordIds from first-
attempt eligibility. Correct pairs lock independently, so another pair can match
while the previous pair settles to its muted surface. Local Tailwind animation tokens implement a 200 ms
shake and 350 ms green-to-muted background feedback; motion-reduce removes them. Listen remains
independent of selection. Typed insufficient-pool state exposes setup recovery.
Preparation batch-fetches selected Word rows for all six variants; label keys,
builder draw order/canonical input and core RNG remain unchanged.

SetupState owns a nonempty canonical selected-type list. Start chooses with
existing ambient entropy; later Next/replay choices exclude the previous type
when multiple types are selected. exercise_types.rs owns canonical list parsing,
random selection and the optional types query metadata. query_pairs still formats
the core round fields, and the router serializes the combined query exactly once.
The current route determines the displayed exercise and URL loads never randomize.
Each shared-session Ready accumulates four required items for matching or one for
MCQ. The summary uses accumulated first-attempt items / required items across the
actual sequence. Only Next/Start/replay use ambient type entropy; build_round keeps
the unchanged v1 single-stream replay contract. No new dependencies or DB changes.


## Practice presentation (A49)

Matching/MCQ components cap their own opaque board at 56rem; AppShell remains
full width. MatchCard owns a solid state surface and sibling selection/Listen
buttons inside one visual card. A52 removes the audio footer, uses a right-edge
44px Listen target and 88px minimum matching height; text reserves horizontal
space for audio/feedback. Matching board spacing is compact; long rows can grow
without clipping. Display pinyin comes from the already-loaded
Word metadata only on Hanzi-to-English matching and Hanzi-to-meaning MCQ. No
loader/query/core/session changes. Match success animates background color for
350 ms rather than making text/background translucent; reduced motion skips it.
