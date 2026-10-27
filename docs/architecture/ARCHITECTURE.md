# Architecture

This file is the overview. Detailed module notes live under
[modules/](modules/); database and schema details live under
[schema/](schema/); UI state and motion proposals live in
[UI_ARCHITECTURE.md](UI_ARCHITECTURE.md).

## Repository snapshot

Described as of 2026-10-27 during an ongoing staged repository migration. This
describes files on disk, not the last committed release.

- Root `Cargo.toml` includes `web`, `crates/db-gen`, `crates/zipeek-core`, and
  `e2e`.
- `web` is the Rust 2024 Yew CSR package `zipeek`. `main.rs` initializes the
  environment, logging, and renderer; `lib.rs` declares the app modules.
- `web/src/` contains `app`, `components`, `contexts`, `env`, `models`, `pages`,
  `route`, `services` and `theme` modules. App mounts `ThemeProvider`,
  `HashRouter`, `AppShell`, `DatabaseProvider` and `Switch`.
- `route.rs` declares Home, Random, PracticeSetup, three match variants, Browse,
  and a `/404` fallback. MCQ routes are approved by D20 but not declared. Browse
  and setup are placeholders; Practice parses query state and renders Loader.
- Trunk configuration and HTML live in `web/`. Development port is `1420`.
  Styles live in `web/styles/`; HTML copies `web/public/` assets and the
  generated static bundle.
- D27 makes root `Cargo.lock` authoritative. Trunk enforces locked builds and
  explicitly watches web, zipeek-core and root workspace manifest/lock/toolchain
  files.
- `zipeek-e2e` is a workspace package with `src/main.rs` and `tests/smoke.rs`.
  Both use managed Chrome and an external Trunk server on port 1420. Only the
  test enables headless mode; the executable suppresses the automation banner.
  Shared readiness, owned Trunk lifecycle, and the package rename to `e2e`
  remain open (A15).
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
an Animato adapter and a stable route-transition host. The existing providers
still use state handles; these proposals are not implemented or implicitly
approved dependency changes. A15's concrete modular e2e proposal is in
[testing/e2e.md](../testing/e2e.md#a15-modularization-proposal-from-the-2026-10-27-review).

D31 requires Tailwind-first component styling. Tailwind's build/import is still
active, but much of the recent shell/card UI uses bespoke main.css selectors;
this is recorded drift, not an approved architectural switch. A35 owns migration
with visual parity; see
[UI_ARCHITECTURE](UI_ARCHITECTURE.md#tailwind-styling-correction-d31) and
[CODING_STANDARDS §Tailwind](../process/CODING_STANDARDS.md#tailwind-styling-d31).

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