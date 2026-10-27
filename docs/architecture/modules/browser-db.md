# browser-db module

Browser DuckDB-WASM query layer: worker/Arrow bridge, runtime, row decoding, and
typed queries. Part of the [architecture overview](../ARCHITECTURE.md). Approved
runtime pin in [D08](../../process/decisions/database-and-artifacts.md#d08-duckdb-wasm-version);
engine/storage in [D23](../../process/decisions/database-and-artifacts.md#d23-native-neo-engine-and-compatible-storage).

## Browser query modules (A06)

`services/database` separates:

| File | Responsibility |
| --- | --- |
| `bridge.js` | Worker/Arrow/prepared-statement interop |
| `runtime.rs` | Rust async handle, cleanup and metadata |
| `row.rs` | Validated `Word` conversion |
| `queries.rs` | Bound SQL |
| `error.rs` | Typed query/runtime errors |

`Database` exposes ID lookup, exact simplified/traditional and display/numeric-
pinyin lookup, full HSK-union rows/count/ordered offset, HSK/PoS pagination and
the three `CandidatePool` views. Widening clamps within the chosen system. Every
word query uses `ORDER BY id ASC`; SQL removes bound exclusions before offsets.
No user value becomes an identifier.

## Runtime facts

- Pin: `@duckdb/duckdb-wasm@1.32.0` exact (npm).
- Engine: `v1.4.3`.
- Reads the native neo v2.0.0 artifact with explicit v1.4.0 storage.
- Actual `web/static/hanzi.duckdb`: 12,100 rows, ten columns, 3,158,016 bytes,
  SHA-256 `491fae6f84a96b5825c860163701f3e138cf72e5b50763848fde0e96ba03c546`.
- Opens the full fetched buffer `READ_ONLY`. No OPFS persistence, no automatic
  DB generation.
- Worker/WASM/Arrow assets are served locally, not from a CDN.
- The `/zipeek/` build passes Chrome verification without cross-origin isolation
  headers.

Future regenerated artifacts must repeat compatibility checks; no default neo
storage compatibility is implied.

## Asset packaging

`web/scripts/prepare-duckdb.cjs` runs before Trunk, verifies npm version 1.32.0,
copies matching single-threaded MVP/EH workers/WASM, Arrow's packaged browser
bundle and library notices into ignored `vendor/duckdb`. It localizes DuckDB's
bare Arrow import to a small local adapter; no extra bundler dependency or CDN.
The adapter loads Arrow's unmodified classic bundle and supplies the three
exports used by the pinned DuckDB module. `document.baseURI` resolves all runtime
assets and the dictionary for `/` and `/zipeek/`. Generated vendor files are
excluded from watch paths to prevent rebuild loops. npm installation is a
prerequisite.

The npm tarball lacks DuckDB's MIT license file; packaging includes the exact
[v1.32.0 upstream license](https://raw.githubusercontent.com/duckdb/duckdb-wasm/v1.32.0/LICENSE)
from `web/scripts/duckdb-LICENSE.txt`. Arrow's packaged Apache license and NOTICE
are copied alongside assets. No source notices or dictionary attribution changed.

## Provider lifecycle

`contexts/database.rs` exposes `DatabaseState` and retry; `DatabaseProvider`
initializes `Loading` on its first render, opens/fetches the DB and validates a
typed probe before `Ready`. It persists across route changes. Failed attempts
terminate their worker; dropped handles close connections and terminate workers.
No OPFS writes.

`DatabaseProvider` inside `AppShell` owns Loading/Ready/Failed and Retry.
Navigation/theme stay usable while routes wait for the dictionary. Failures are
logged with technical detail; the UI uses a concise unavailable message. The
separate `ErrorPage` remains unused.

## Proposed extraction (A33/D30)

The 2026-10-27 review proposes extracting the browser adapter into
`crates/zipeek-db-wasm` (package `zipeek-db-wasm`). See
[target-architecture.md](target-architecture.md#proposed-browser-database-crate-extraction-a33)
for the proposed boundary, dependency direction, and migration order. This is a
proposal, not an implemented crate or approved dependency move.

The following mapping is proposed for the eventual extraction:

| Current module/responsibility | Proposed owner |
| --- | --- |
| `services/database/runtime.rs` and JS bridge | `zipeek-db-wasm` browser backend and connection lifecycle |
| `services/database/row.rs` and `error.rs` | `zipeek-db-wasm` decoding and actionable query/runtime errors |
| `services/database/queries.rs` | `zipeek-db-wasm` typed read API and private SQL planning/binding |
| Word, WordId, HSK/PoS, candidate and round types | Existing `zipeek-core`, unchanged |
| `DatabaseContext`, reducer, Retry and readiness presentation | web providers/hooks and UI state |
| Fetch options/base URL selection | web constructs explicit database options |
| npm manifest/lockfile, Trunk hook and deployed asset copying | web build/application deployment |
| Low-level browser adapter tests | Dedicated adapter fixture; user journeys remain in e2e suites |

Retain the existing narrow typed `Database` facade for initialization, metadata,
ID and hanzi/pinyin lookups, counts, ordered offsets, pages and candidate views.
Return core types and typed errors. Keep `JsValue`, Arrow tables, SQL strings and
worker handles private; consumers must not execute arbitrary SQL through a public
API. Move the existing API first, without redesigning query semantics or replay.

## Testing

Browser database tests, the Pages-prefix test, and the native/wasm replay runner
are documented in [testing/browser-db.md](../../testing/browser-db.md). The e2e
harness currently requires an external Trunk server on port 1420; owned
lifecycle/readiness belongs to A15 and is tracked in
[testing/e2e.md](../../testing/e2e.md).