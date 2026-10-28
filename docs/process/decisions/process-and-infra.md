# Process and infrastructure

Approved e2e, lockfile, and crate-extraction decisions. Each entry links back
to the [index](index.md) and to its owning document. Bodies are the approved
requirement as recorded on the date shown; they do not imply the feature is
implemented.

---

## D09 E2e lifecycle

**Status:** Approved 2026-10-26.
**Owner:** [TESTING](../../testing/e2e.md), [AGENTS](../../../AGENTS.md).
**Index row:** [D09](index.md#approved-decisions).

Harness owns a Trunk child and one managed Chrome WebDriver session per test
binary.

- **Session:** `WebDriver::managed(DesiredCapabilities::chrome())`. Do not
  install ChromeDriver separately or put it on PATH; the `manager` feature
  handles matching, download, subprocess and teardown. Driver download/cache
  availability is a prerequisite.
- **Serial by default:** `cargo test -p e2e --locked -- --test-threads=1`. One session
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
- **Implemented by A15, 2026-10-28:** package renamed to `e2e`; the root
  lockfile records only that identity change, with dependencies preserved.

A15 implements this lifecycle and D28 modularity. Root owns Trunk serve; Pages
owns its separate prefix build and Node static server. Both are verified in Chrome.

---

## D27 Workspace lockfile

**Status:** Approved 2026-10-27.
**Owner:** [AGENTS](../../../AGENTS.md), [TESTING](../../testing/TESTING.md),
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

## D28 Modular e2e infrastructure

**Status:** Proposed 2026-10-27; concrete A15 scope approved and implemented
2026-10-28. No dependency additions or upgrades.
**Owner:** [TESTING](../../testing/e2e.md).
**Index row:** [D28](index.md#approved-decisions).

The user's 2026-10-27 review identified e2e/tests/smoke.rs as a dumping ground.
A15 implements the layout in [testing/e2e.md](../../testing/e2e.md#implemented-layout-and-ownership),
with a typed harness error module and failure-process fixture in addition to the
proposed modules. Named scenarios share one session per binary.

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

Implemented by A15: eight root scenarios (seven original scenarios with the
adapter probe separated) and one Pages scenario, represented by two integration
tests. Four lifecycle tests prove failure/unwind/descendant cleanup; two expected
browser faults prove same-session isolation. Artifact capture preserves primary
failures. The stale Home h1 selector now targets the existing header Home link,
retaining the Zipeek text assertion and user branding without app changes.

---

## D30 Extract browser database adapter (proposed)

**Status:** Proposed 2026-10-27 by user review. Existence of the boundary is
approved; concrete crate name, dependency moves, and migration require explicit
approval before A33 starts.
**Owner:** [ARCHITECTURE](../../architecture/ARCHITECTURE.md).
**Index row:** [D30](index.md#approved-decisions).

Extract the A06 browser adapter into a dedicated crate
`crates/zipeek-db-wasm` (package `zipeek-db-wasm`). Detailed proposal at
[ARCHITECTURE §A33](../../architecture/modules/target-architecture.md).

Dependency direction:

```text
web -> zipeek-db-wasm -> zipeek-core
web ------------------> zipeek-core
db-gen ---------------> zipeek-core
db-gen -> native duckdb-neo
```

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

## D33 Accept build-tool Picomatch advisories

**Status:** User acceptance recorded 2026-10-28; existing ReDoS answer reconfirmed
and extended to the POSIX advisory during A37.
**Owner:** [CI](../CI.md).
**Index row:** [D33](index.md#approved-decisions).

Accept picomatch 2.3.1's CVE-2026-33671 (extglob ReDoS) and CVE-2026-33672
(POSIX character-class matching) under the condition that it remains a build-tool
dependency receiving developer-controlled glob patterns, not application user
input. Do not add an override, run npm audit fix or refresh the lockfile for these
advisories. Reassess if that input/deployment boundary changes. An independently
approved future Trunk/Tailwind upgrade may naturally bring a fixed version.

Affected version 2.3.1 and patched lines 2.3.2 / 4.0.4 are retained for reference.
Acceptance is not remediation: npm may continue to report the advisory. A39's
proposed repair was created despite the existing acceptance and is retired by
this correction. No dependency target or graph changes.
