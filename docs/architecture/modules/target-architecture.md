# Target architecture

Current and proposed workspace/crate structure, the A33 crate extraction
proposal, and the runtime data-flow summary. Part of the
[architecture overview](../ARCHITECTURE.md). Proposed extraction in
[D30](../../process/decisions/process-and-infra.md#d30-extract-browser-database-adapter-proposed).

## Workspace members

The intended workspace:

```text
web
crates/db-gen
crates/zipeek-core
e2e
```

web is the Yew CSR package zipeek.

crates/db-gen is the native generator (db-gen).

crates/zipeek-core is the shared domain crate (D01).

e2e is the browser harness package; currently named zipeek-e2e, pending
rename to e2e in A15 (D09).

Do not add nonexistent crates merely to match this map.

Current crate graph
text
web ------------------> zipeek-core
db-gen ---------------> zipeek-core
db-gen -> native duckdb-neo
web -> @duckdb/duckdb-wasm (npm)
zipeek-core has no I/O and no wasm-only dependencies (D01).

db-gen is a native-only binary and library; it never imports the browser
runtime.

web bundles DuckDB-WASM locally via the A06 pre-build hook; assets live under
ignored vendor/duckdb.

Proposed crate graph (A33 / D30)
Proposal only, not approved or implemented:

text
web -> zipeek-db-wasm -> zipeek-core
web ------------------> zipeek-core
db-gen ---------------> zipeek-core
db-gen -> native duckdb-neo
Never make zipeek-core depend on the browser adapter or either DuckDB runtime.
The generator keeps its native connection and engine/storage guards; it does not
import the browser crate. Web continues to call A07 directly for deterministic
round construction. No new framework-neutral domain crate is needed.

Proposed browser database crate extraction (A33)
The A06 services/database module is a reasonable extraction boundary now that
it owns transport, typed decoding and SQL independently of Yew. Unlike an earlier
suggestion of another generic shared-logic crate, this isolates a concrete browser
runtime dependency. Proposed crate: crates/zipeek-db-wasm, package
zipeek-db-wasm. The name, manifest moves and extraction are proposed, not
approved or implemented.

The detailed ownership mapping (current module → proposed owner) lives at
browser-db.md. Summary constraints
from D30:

Retain the existing narrow typed Database facade; move the API first, without
redesigning query semantics or replay.

DatabaseOptions explicitly carries database URL, asset base and relevant
timeouts; the library must not hard-code the app's hosting prefix, inspect
routes or silently select a CDN.

The crate owns resource cleanup and failed-open disposal; the app provider owns
the lifetime of one client and its retry policy. Review an explicit awaited
shutdown operation; Drop alone cannot await cleanup. Do not silently close a
live shared client when one consumer releases its reference.

Keep the library free of Yew, Animato, localStorage, page loading components
and application-global context.

Native gates must still work: worker/JS code is wasm-only, and native open
returns a typed unsupported-target error. Never silently fake a working DuckDB
connection.

The npm dependency stays exactly @duckdb/duckdb-wasm@1.32.0 in web. The crate
does not run npm or copy assets from build.rs. Keep application-controlled
packaging and license notices, and verify wasm-bindgen resolves the relocated
bridge from the new crate's manifest directory.

The existing browser smoke probe's private snippet discovery must be replaced
by an explicit adapter-test entry point, as described in A15.

Before implementation, inventory the exact dependencies needed by the new crate:
zipeek-core, serde, serde_json, thiserror, log, wasm-bindgen,
wasm-bindgen-futures, js-sys and serde-wasm-bindgen as applicable. Keep the
current resolved versions/features. Some remain direct dependencies of web for
other uses; do not blindly move or delete them. Prefer target-specific JS
dependencies where the chosen facade allows it. Any testing/mock dependency must
be named in the plan; no async-trait or general DI framework is approved by
this proposal.

Migration order (proposed)
Complete A15 isolation and preserve a reproducible browser baseline.

Approve the crate name, public facade, exact dependency additions/moves and
native-target strategy; add workspace/path wiring and update Cargo.lock.

Move transport/decoding/private SQL with tests; leave a temporary web re-export
only if useful while callers migrate. Avoid maintaining two implementations.

Keep providers and feature commands in web, injecting the extracted client.
Remove the old service modules after callers use the new boundary.

Run native gates, typed query tests, real worker/read-only/full-file checks,
loading/retry, and actual /zipeek/ build tests. Preserve artifact checksums,
package pins, query ordering and all A07 replay vectors.

The benefit is dependency/test isolation and a stable client boundary; it is not
needed to share pure logic already in core. The cost is another manifest, target
configuration and JS asset integration boundary. Keep it proposed until the user
approves those concrete changes; feature work can use the existing facade meanwhile.

Practice and rounds boundary
Practice state, query state, and component ownership should be documented here
when approved and implemented. Practice setup selects HSK system/level and
Endless or Fixed mode. PoS filtering is supported in URLs; its setup picker is
future work. Practice and Random share one round generator; Random supplies a
whole-DB seed and always uses Endless. Internal candidate reserves never render:
the visible round remains four matched cards per side. See
PRODUCT.

D13 makes unseeded rounds deterministic ordered slices selected by URL offset;
seeded replay instead uses the seed and its HSK context. Canonical URL state carries
mode, Fixed round count when applicable, HSK context, optional canonical PoS set,
exclusions, and either offset or seed. Random emits the seed's PoS. The setup
count choices are 5/10/20 with default 10. Fixed completion tracking must remain
independent of offset wrapping.

Word exclusions are a localStorage-persisted set of WordIds through gloo
storage, serialized into round URLs and applied before generation. Creating a
URL unions saved/current-round exclusions; loading a URL uses only its set, with
absence meaning empty. Persist IDs only on explicit user actions, not on link
load. Marking a new exclusion changes saved/pending state for the next generated
URL, leaving current-round cards, active inputs, and URL unchanged. Setup Reset
clears saved/pending sets and generates a new URL without exclude. This state
is distinct from transient scores and forbidden persisted hash state.

D15 fixes level default 1, rejects malformed/missing/excluded seeds into unseeded
inputs with the same filter/offset, and ignores seeds for Fixed. Validate those
cases before zeroing the offset of a retained seed. Fixed progress is in-memory,
resets on reload, and is independent of the URL's pool position or wrap. Seeded
Next clears its anchor and advances by four as an unseeded Endless round.

ROUND_GENERATION owns the approved D12
algorithm and resolved D19 single-stream/v1 replay contract. The pure build
pipeline owns its one RNG; recovery helpers borrow that stream, and
generation/shuffling continue from its resulting position. The caller supplies
original URL metadata and ordered data, not an independently recovered seed and
newly seeded RNG. A07 supplies full-round native/wasm vectors.

The wasm caller builds three exclusion-clean CandidatePool views with SQL
ORDER BY id ASC; a pure native-testable builder performs all deterministic
selection. URL parsing normalizes seeded offsets to zero. A hand-written ASCII
seed string prefixed zipeek-round-v1 feeds FNV-1a then rand::rngs::ChaCha12Rng
from the existing rand 0.10.3 dependency with its approved chacha feature (D17).
An existing URL whose seed lacks membership uses deterministic, bounded ChaCha
recovery followed by the first eligible id; never the ambient RNG used for a fresh
Random click. Preserve the original seed URL and retain recovery in effective state.
The builder does not touch database, localStorage, clock, or ambient RNG. Sliding
and fallback affect only internal window starts. The approved rand chacha
feature is enabled in zipeek-core after the approved A07 dependency move; the
builder and portable full-round vectors are implemented. Do not add
a separate rand_chacha dependency.

Shared domain crate in the workspace
zipeek-core (D01) provides Word, ExerciseVariant, HSK domain types, and
query types used by web and db-gen. Its scaffold and workspace/path wiring
exist. Word/WordId, HSK system/levels and all 42 PartOfSpeech values are
implemented; practice variants/URL state and the pure round builder are
implemented. SQL query and playable exercise integration remain future work.

The HSK domain distinguishes system from level/tier and supports independent
memberships for HSK 2.0 and HSK 3.0. A single universal integer level cannot
represent that requirement. Hsk3Level::SevenToNine maps the advanced tier to
integer 7 in storage/canonical URLs; 8 and 9 are not separate levels. Remaining
database-query types remain to be designed; SQL storage uses D07's separate
nullable hsk2_level and hsk3_level columns in words.

During development, breaking code/data changes and regeneration are acceptable;
backward compatibility and application/database versioning are not required.
This does not authorize changing pinned dependencies without approval (D10).

Related
Browser database module: browser-db.md.

Shared crate: zipeek-core.md.

Runtime data flow: data-flow.md.

Architecture overview: ARCHITECTURE.md.