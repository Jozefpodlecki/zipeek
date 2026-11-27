# Process and infrastructure

Approved e2e, lockfile, and crate-extraction decisions. Each entry links back
to the [index](index.md) and to its owning document. Bodies are the approved
requirement as recorded on the date shown; they do not imply the feature is
implemented.

---

## D09 E2e lifecycle

**Status:** Approved 2026-10-26.
**Owner:** [TESTING](../../archive-history/audits/e2e.md), [AGENTS](../../../AGENTS.md).
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
**Owner:** [AGENTS](../../../AGENTS.md), [TESTING](../../testing-quality/acceptance/testing.md),
[CI](../../build-deployment/ci.md).
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
**Owner:** [TESTING](../../archive-history/audits/e2e.md).
**Index row:** [D28](index.md#approved-decisions).

The user's 2026-10-27 review identified e2e/tests/smoke.rs as a dumping ground.
A15 implements the layout in [testing/e2e.md](../../archive-history/audits/e2e.md#implemented-layout-and-ownership),
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
**Owner:** [ARCHITECTURE](../../architecture/architecture.md).
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

Historically, the npm dependency stayed exactly @duckdb/duckdb-wasm@1.32.0 in web (superseded by D08 amendment of 2026-11-09). The crate
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
**Owner:** [CI](../../build-deployment/ci.md).
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


## D36 Proportionate local validation

Approved by direct user instruction on 2026-10-29. Routine edits use focused
checks and an obvious representative sample of affected behavior. Full
regression is reserved for very major work, such as substantial feature
completion or broad architecture/runtime/data changes, and release readiness.
Reuse successful evidence for unchanged code. After fixes, rerun affected checks;
expand only when failures or scope warrant it. Report the sample, omitted checks
and material gaps honestly. Documentation-only work retains documentation gates.
The five commands remain the full-regression set; existing automatic CI workflows
are unchanged. This supersedes the old local rule requiring all five gates and
the full browser suite for every code change. AGENTS section 6 is authoritative;
TESTING gives practical change-type guidance.


## D37 Clap generator CLI

Approved 2026-10-29: exact clap 4.6.7 with its standard defaults plus derive,
limited to native db-gen. Preserve existing locked package versions while adding
the required dependency graph. No app/wasm dependency, DB schema or source changes.
A45 replaces manual argument parsing with help/version, check and generate
--output DIRECTORY. No arguments show help instead of doing an implicit in-memory
import. Check explicitly reports that no database was written; generation prints
absolute output/database/manifest paths regardless of log filtering. Default
output remains repository web/static and existing directories are refused.
Focused native parser/CLI checks validate the change; checked-in DB replacement
remains separate A44 work. Owning instructions are in schema/regeneration.md.


## D38 Explicit database-to-dev tasks

Approved 2026-10-29: use installed cargo-make and existing Node/Rust/Trunk tools,
with no new dependencies, to provide db-check, db-generate, db-install, serve and
dev. The explicit dev command generates once, installs the complete verified
bundle with backup/rollback, then starts Trunk. Bare cargo make lists tasks;
serve alone reuses installed data. Never regenerate during Trunk rebuilds or
Pages builds. Keep all reports/INPUTS/attribution/notices with the database.
Validate every file checksum and pinned input manifest before replacement, stage
and recheck the complete copy, retain the previous directory and roll back failed
publication. Refuse unsafe paths, symlinks, partial bundles and checksum mismatch.
User approval covers this workflow's real bundle installation and focused smoke
validation. Candidates/backups remain under target/db-workflow for review.

## Tauri frontend/database separation (2026-11-26)

Approved by the user in this session: separate desktop database/assets first. The Yew/WASM UI remains shared; a Tauri frontend must omit browser dictionary preloads, compressed dictionary downloads and DuckDB-WASM/Arrow vendor assets. Browser builds retain their existing engine and `/zipeek/` deployment behavior. Select the database backend once during startup and keep shared metadata, history-row contracts and pure evidence summaries independent of the WASM transport. Native resource staging must select the installed manifest's dictionary, validate bytes and SHA-256, and package a stable native resource without regeneration or dependency changes.

This scope reuses the existing dictionary and Practice-history IPC handlers. Remaining personal storage and speech commands require separate scopes. No migration, data deletion or model integration is approved by this decision. Owners: [native database](../../architecture/modules/tauri-native-database.md) and [web assets](../../architecture/modules/web-assets.md).

## Confirmed Settings reset (2026-11-26)

The user explicitly approved **Settings > Reset app > Clear all data and start over** after reviewing the permanent data-deletion scope. Delete Zipeek-owned preferences, progress, Word Sets, exclusions, saved sessions and personal snapshots in browser/WebView storage and native Tauri learning history. Preserve dictionary/model files, exported backups and unrelated origin data. Require an explicit in-app confirmation, isolate cleanup from normal providers, block conflicting active app sessions, and return to fresh landing/onboarding only after complete success. Show partial failures with Retry. No dependency addition, migration or broader native storage adapter is approved by this change. Owner: [personal storage](../../dictionary-data/personal/personal-data-storage.md#confirmed-app-reset-2026-11-26).


## Desktop shutdown screen (2026-11-27)

Approved by the user: keep the desktop window visible while stopping Zipeek-owned local services. Intercept close/exit requests, show a themed spinner with Closing Zipeek and Stopping local services, reject duplicate shutdown and new service starts, and close after cleanup. A ten-second timeout or cleanup failure offers Close now; a later native close request also permits exit. External services remain untouched. This preserves the existing process termination approach and adds no dependency or personal-data format change. See [native services](../../architecture/modules/tauri-managed-services.md#desktop-shutdown-lifecycle) for implementation and pending acceptance.


## Desktop development GGUF path (2026-11-27)

The user approved setting `ZIPEEK_AI_MODEL_PATH` on every `desktop-dev` task invocation to the exact absolute LM Studio GGUF path supplied in the conversation. The owning value lives in Makefile.toml under `tasks.desktop-dev.env`; the selected model is Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf. Scope is the development launch and its inherited child-process environment, with no model copy, dependency change or installed application policy change. See [native services](../../architecture/modules/tauri-managed-services.md#approved-desktop-development-model-path).
