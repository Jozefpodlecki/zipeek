# Process and infrastructure

Approved e2e, lockfile, and crate-extraction decisions. Each entry links back
to the [index](index.md) and to its owning document. Bodies are the approved
requirement as recorded on the date shown; they do not imply the feature is
implemented.

---

## D09 E2e lifecycle

**Status:** Approved 2026-10-26.
**Owner:** [TESTING](../testing/e2e.md), [AGENTS](../../../AGENTS.md).
**Index row:** [D09](index.md#approved-decisions).

Harness owns a Trunk child and one managed Chrome WebDriver session per test
binary.

- **Session:** `WebDriver::managed(DesiredCapabilities::chrome())`. Do not
  install ChromeDriver separately or put it on PATH; the `manager` feature
  handles matching, download, subprocess and teardown. Driver download/cache
  availability is a prerequisite.
- **Serial by default:** `cargo test -p e2e -- --test-threads=1`. One session
  per binary avoids port/browser conflicts.
- **Readiness helper:** shared once in the e2e crate. 30-second timeout,
  100 ms exponential backoff capped at 2 s, HTTP-response or TCP-connect
  readiness, no fixed sleep. Bound each attempt/wait by the remaining timeout.
  Report failures with child exit status/output where available.
- **Base path:** Trunk starts *without* `--public-url /zipeek/`; e2e navigates to
  `http://127.0.0.1:<port>` at `/`. The `/zipeek/` base path is a Pages
  deployment concern validated separately.
- **Cleanup:** child guard kills and reaps the Trunk child on drop, including
  setup failure and test failure. Never leave an owned process behind.
- **Renames pending:** the current manifest is `zipeek-e2e`; renaming to `e2e`
  is required to support the target command.

This decision specifies the lifecycle; concrete modular harness layout remains
A15 work (see D28).

---

## D27 Workspace lockfile

**Status:** Approved 2026-10-27.
**Owner:** [AGENTS](../../../AGENTS.md), [TESTING](../testing/TESTING.md),
[CI](../CI.md).
**Index row:** [D27](index.md#approved-decisions).

Track the existing root `Cargo.lock` and remove its ignore entry, preserving
the current resolved graph. Under A19:

- Root `Cargo.lock` staged for tracking; `.gitignore` entry removed.
- Obsolete `web/Cargo.lock` deleted.
- Trunk builds enforce locked resolution via `web/Trunk.toml`.
- Trunk watches shared core sources plus root workspace/lock/toolchain files,
  so shared-core edits trigger rebuilds.
- Cargo validation uses `--locked`.

Direct version requirements are unchanged; dependency updates still require
explicit approval. No exact Trunk version is selected; the tested local
installation is 0.22.0.

CI migration (A16) must use the root lockfile for cache keys and preserve locked
validation in CI. Any transitive-lockfile policy beyond this decision remains
unresolved.

---

## D28 Modular e2e infrastructure (proposed)

**Status:** Proposed 2026-10-27 by user review. Existence of the need is
approved; concrete layout requires implementation review by A15.
**Owner:** [TESTING](../testing/e2e.md).
**Index row:** [D28](index.md#approved-decisions).

The user's 2026-10-27 review identified `e2e/tests/smoke.rs` as a dumping ground
and requested modular infrastructure. Concrete proposal lives at
[TESTING §A15 modularization proposal](../testing/e2e.md#a15-modularization-proposal-from-the-2026-10-27-review).

Approved requirements:

- Split into harness library, named scenario suites, page helpers, and browser
  fixtures.
- Retain D09's one session per binary and separate root/Pages validation.
- Preserve all existing assertions, including read-only rejection and
  `/zipeek/` assets.
- Run without manually starting a server; prove cleanup on failed startup and
  scenario; record build/session failures with artifacts.
- Do not kill unrelated processes occupying old ports; use dynamically assigned
  ports and per-run output directories.
- Rename the package to `e2e` and update its workspace lockfile entry
  deliberately.
- No new framework, driver service, or ChromeDriver subprocess.

Implementation: A15 owns the split. Proposed layout in `TESTING/e2e.md` is a
proposal, not approved structure.

---

## D30 Extract browser database adapter (proposed)

**Status:** Proposed 2026-10-27 by user review. Existence of the boundary is
approved; concrete crate name, dependency moves, and migration require explicit
approval before A33 starts.
**Owner:** [ARCHITECTURE](../architecture/ARCHITECTURE.md).
**Index row:** [D30](index.md#approved-decisions).

Extract the A06 browser adapter into a dedicated crate
`crates/zipeek-db-wasm` (package `zipeek-db-wasm`). Detailed proposal at
[ARCHITECTURE §A33](../architecture/modules/target-architecture.md#proposed-browser-database-crate-extraction-a33).

Dependency direction:

```text
web -> zipeek-db-wasm -> zipeek-core
web ------------------> zipeek-core
db-gen ---------------> zipeek-core
db-gen -> native duckdb-neo

Approved constraints:

Never make zipeek-core depend on the browser adapter or either DuckDB runtime.

The generator keeps its native connection and engine/storage guards; it does
not import the browser crate.

The library owns transport, typed decoding, and private SQL planning/binding.
Expose a narrow typed Database facade for initialization, metadata, ID and
hanzi/pinyin lookups, counts, ordered offsets, pages, and candidate views.
Keep JsValue, Arrow tables, SQL strings, and worker handles private.

DatabaseOptions explicitly carries database URL, asset base, and relevant
timeouts. The app resolves them for / or /zipeek/; the library must not
hard-code the hosting prefix, inspect routes, or silently select a CDN.

The library owns resource cleanup and failed-open disposal; the app provider
owns the lifetime of one client and its retry policy. Review an explicit
awaited shutdown operation; Drop alone cannot await cleanup. Do not silently
close a live shared client when one consumer releases its reference.

Keep the library free of Yew, Animato, localStorage, page loading components
and application-global context. A reducer must not depend on a JS worker
handle to represent Loading or Failed.

Native gates must still work: worker/JS code is wasm-only, and native open
returns a typed unsupported-target error. It must never silently fake a
working DuckDB connection.

The npm dependency stays exactly @duckdb/duckdb-wasm@1.32.0 in web. The crate
does not run npm or copy assets from build.rs.

The existing browser smoke probe's private snippet discovery must be replaced
by an explicit adapter-test entry point (as described in A15).

Migration order (per proposal):

Complete A15 isolation and preserve a reproducible browser baseline.

Approve crate name, public facade, dependency additions/moves, native-target
strategy; add workspace/path wiring and update Cargo.lock.

Move transport/decoding/private SQL with tests.

Keep providers and feature commands in web, injecting the extracted client.

Run native gates, typed query tests, real worker/read-only/full-file checks,
loading/retry, and actual /zipeek/ build tests.

Benefit: dependency/test isolation and a stable client boundary. Cost: another
manifest, target configuration, and JS asset integration boundary. Keep it
proposed until the user approves those concrete changes; feature work can use
the existing facade meanwhile.