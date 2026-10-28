# browser-db module

Browser DuckDB-WASM query layer: worker/Arrow bridge, runtime, row decoding, and
typed queries. Part of the [architecture overview](../ARCHITECTURE.md). Approved
runtime pin in [D08](../../process/decisions/database-and-artifacts.md);
engine/storage in [D23](../../process/decisions/database-and-artifacts.md).

## Browser query modules (A06)

`services/database` separates:

| File | Responsibility |
| --- | --- |
| `bridge.js` | Worker/Arrow/prepared-statement interop |
| `runtime.rs` | Rust async handle, cleanup and metadata |
| `row.rs` | Validated `Word` conversion |
| `queries.rs` | Bound SQL, including A21 ordered batch ID lookup |
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

A31 places DatabaseProvider above HashRouter and AppShell. It owns the client,
a pure readiness reducer and separate stable Dictionary/readiness/Retry contexts.
DatabaseBoundary inside AppShell presents Loading/Ready/Failed+Retry. Navigation
and theme stay usable while routes wait. Only a successful typed probe permits
Ready. Technical errors are logged; the UI keeps its concise unavailable message.
The separate ErrorPage remains unused; no OPFS writes are added.

Retry invalidates the previous generation synchronously and releases the owned
client. Old effect cleanup cannot cancel a newer generation. The JS open API
cannot be aborted: pending opens settle, then stale successful handles are dropped
and asynchronously closed with their worker. Reducers reject stale/duplicate
completions too. Unmount invalidates the generation and releases the owned handle;
existing query clones retain normal last-handle cleanup. See
[UI state boundaries](../UI_ARCHITECTURE.md#a31-implemented-state-boundary--2026-10-28)
for the hooks and cancellation limits.

## Proposed extraction (A33/D30)

The 2026-10-27 review proposes extracting the browser adapter into
`crates/zipeek-db-wasm` (package `zipeek-db-wasm`). See
[target-architecture.md](target-architecture.md)
for the proposed boundary, dependency direction, and migration order. This is a
proposal, not an implemented crate or approved dependency move.

The following mapping is proposed for the eventual extraction:

| Current module/responsibility | Proposed owner |
| --- | --- |
| `services/database/runtime.rs` and JS bridge | `zipeek-db-wasm` browser backend and connection lifecycle |
| `services/database/row.rs` and `error.rs` | `zipeek-db-wasm` decoding and actionable query/runtime errors |
| `services/database/queries.rs` | `zipeek-db-wasm` typed read API and private SQL planning/binding |
| Word, WordId, HSK/PoS, candidate and round types | Existing `zipeek-core`, unchanged |
| Dictionary/status/command contexts, reducer, Retry and readiness presentation | web providers/hooks and UI state |
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
are documented in [testing/browser-db.md](../../testing/browser-db.md). A15 now owns
Trunk/fixture servers, readiness, scenario isolation and managed Chrome at dynamic
ports, including an explicit adapter fixture entry; see
[testing/e2e.md](../../testing/e2e.md).

## Planned Browse query extension (A13, A22 requirements)

Proposed typed facade: browse(query, filters, strategy, page, page_size), returning
rows and a total count for the same predicate. No such method is implemented by
A22. Keep existing ID-ordered practice queries unchanged.

Build a bound predicate with AND between chips: complete canonical PoS token,
selected HSK column membership/exact level/inclusive range, frequency rank or
inclusive range, WordId, exact simplified/traditional hanzi or exact pinyin.
Select columns/strategy only through typed enums; bind every user value, including
pagination. Numeric/display pinyin lookup must support the approved forms.

OR the free-text field matches, then AND that group with chip predicates:
comparison-only pinyin search uses tone-stripped/whitespace-free/case-folded values
and exact/prefix/substring strategy; English always uses case-insensitive substring;
simplified hanzi always uses prefix. Keep search normalization separate from
D18's tone-preserving replay key and never rewrite stored text. Plan literal search
escaping/binding during implementation; do not expose arbitrary SQL to UI callers.

Browse alone orders by pinyin ASC, simplified ASC as the bytewise tie-break;
run the companion count without pagination. Page sizes are 5/10/15/20, default 5.
Debounce/cancellation, late-result rejection and delayed-spinner presentation stay
in the feature effect/reducer boundary, not the database transport. A37 resolves URL-token mapping, malformed-range behavior and the five result
fields in UX; A13 owns their implementation.

## A21 practice consumer

words_by_ids binds all requested IDs and orders by id ASC; empty sets return
without a query. The practice loader uses it to validate URL exclusions and the
non-Fixed seed, preserving typed Word conversion. It then uses existing
candidate_pool/hsk_union queries and the pure core builder. These are read-only
queries with no schema or artifact changes. DatabaseBoundary is composed per
recognized route; the unknown-route fallback does not need database readiness.
Feature effects discard late results and abort only their Rust orchestration;
the app-owned client survives route changes.
