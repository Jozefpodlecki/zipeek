# Architecture

This file is the overview. Detailed module notes live under
[modules/](modules/); database and schema details live under
[schema/](schema/); UI state boundaries and motion proposals live in
[UI_ARCHITECTURE.md](UI_ARCHITECTURE.md).

## Repository snapshot

Described as of 2026-10-28 during an ongoing staged repository migration. This
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
  Loading/Ready/Failed+Retry. Ready remains a placeholder until A09/A10 gameplay.
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
question controls remain A10/A09; setup navigation reaching a prepared placeholder
does not claim a playable quiz.
