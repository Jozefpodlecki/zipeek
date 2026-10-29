# Browser database checks

DuckDB-WASM integration checks, the Pages-prefix test, and the native/wasm
replay runner. Part of the [testing index](TESTING.md). Approved runtime pin in
[D08](../process/decisions/database-and-artifacts.md);
required engine/storage in [D23](../process/decisions/database-and-artifacts.md).

## A06 browser database checks

A15 now owns the root/Pages servers and managed Chrome. Run the commands in
[e2e](e2e.md#run-commands-and-prerequisites), without a separately started server.
The preserved A06 scenarios verify:

- Mounted-app Loading while a controlled fetch is held; no early Home render.
- Download failure and successful Retry.
- Actual 12,100-row reads against `web/static/hanzi.duckdb`.
- Typed parameter binding for ID lookup.
- Read-only DELETE rejection (mutation refused by the opened database).

`DatabaseInfo` reports engine v1.4.3 for package 1.32.0. Four native tests cover
typed row decoding/corruption and bound SQL with HSK endpoint clamping, PoS and
exclusion-before-offset rules.

### Pages-specific test

A15's Pages binary is included in normal workspace tests and owns its prefix
build/static server. No fixed 1421 port or ignored-test opt-in is required:

```powershell
cargo test -p e2e --test pages --locked -- --test-threads=1
```

The A06 results below are historical evidence from the earlier external-server
setup; they are not the current harness commands or counts.

The test passes against the real built assets, checks Ready with 12,100 rows, and
verifies same-origin resource URLs stay under /zipeek/. A06 ran this separately:
one pass. The ordinary workspace suite passes 88 domain/generator/query tests and
seven Chrome tests, with this one Pages test explicitly ignored there. Logs:
target/a06-workspace-tests.log and target/a06-pages-chrome.log. All five gates
pass; no deployed-site validation is claimed. A15 now implements server lifecycle; see [e2e](e2e.md).

Browser database operational notes
Run npm ci --ignore-scripts from web/ before Trunk. The pre-build hook verifies
package 1.32.0 and copies matching local MVP/EH workers/WASM, Arrow bundle/adapter
and licenses to ignored web/vendor/duckdb. HTML copies it; watch ignores vendor
to avoid loops. No CDN, automatic npm install or DB generation.

bridge.js resolves assets/dictionary via document.baseURI for / and
/zipeek/, opens a full fetched buffer READ_ONLY, prepares/binds statements
and closes them. runtime.rs owns async Rust handles/cleanup; row.rs validates
WordIds/levels/PoS without changing source display text; queries.rs provides
lookup, full-pool count/ordered offset/rows, pagination and the three clamped
HSK/PoS views.

A31 places DatabaseProvider above the router/shell; DatabaseBoundary inside
AppShell starts Loading before effects and renders routes only after a typed
probe, with concise failure plus Retry. contexts/database.rs supplies separate
Dictionary, DatabaseStatus and DatabaseCommands contexts through focused hooks.
Native tests reject stale/duplicate completions and cover retry/unmount gates.
Chrome verifies navigation/theme changes keep the same download and worker count. The existing practice Loader is still a feature
placeholder; do not interpret it as DB readiness. No Browse/daily/exercise UI
was implemented.

The pinned package reports engine v1.4.3 and reads the native neo v2.0.0 artifact
with v1.4.0 storage. This resolves compatibility for this snapshot. The
dictionary checksum at A06 was
491fae6f84a96b5825c860163701f3e138cf72e5b50763848fde0e96ba03c546.
Future regenerated artifacts must repeat compatibility checks.

npm audit reports existing tooling picomatch 2.3.1 with a high-severity advisory
(target/a06-npm-audit.json). It was already in the previous npm lockfile;
existing package records were preserved and Picomatch is not copied into browser
assets. A16 tracks review; dependency upgrades require specific approval. Do not
run npm audit fix as an unapproved follow-up to this task.

## Executable native/wasm replay check (A07)
From the repository root, with Node and the wasm target installed:

powershell
cargo test -p zipeek-core --offline
cargo build --offline --target wasm32-unknown-unknown -p zipeek -p zipeek-core --lib --example replay_vectors
node crates/zipeek-core/tests/check-round-vectors.mjs
Including web in the wasm build unifies its existing getrandom/wasm_js
feature. The example shares the native test fixtures; the runner installs
throwing host stubs for unused bindgen imports and invokes the exported digest
function. Any host call fails the check. This executes the wasm builder;
compilation alone does not establish replay. Node built-ins add no npm or Cargo
dependencies.

18 vectors cover seeded/unseeded layouts for six variants, recovery success on
draws 1/10, ten-failure by-id fallback, missing-English recovery, sliding/widening
and failed distractor-view stream continuation. The checked-in textual fixture
contains original canonical bytes/hash, requests, accepted IDs and final labels;
its digest is a94efad5496532a0. Native tests also pin that digest literally.
Additional tests cover first-gloss keys, right-only dedup, exclusion-before-offset,
HSK/PoS view validation, bounded failure and no extra recovery RNG requests.
The latest workspace run passes 84 domain/generator tests plus five Chrome tests.
All five gates pass; log: target/a07-workspace-tests.log. The new code is pure;
existing Chrome tests validate the unchanged app after the dependency move.


## A44/A46 regenerated dictionary smoke - 2026-10-29

The cargo make dev sequence generated and installed a complete 12,100-reading
bundle, then served it through Trunk. Downloaded database bytes match web/static
exactly. One managed headless Chrome session loaded the read-only dictionary and
prepared three MCQ options. New SHA-256:
728dca2400ba8f23877043c6216a433e9c0adfad539097f3081cfb200605eb02.
Logical rows/schema/indexes/input and diagnostic hashes match the prior bundle.
Original complete bundle is retained in target/db-workflow/backups. Server and
browser cleanup passed. This is a focused development-path compatibility sample;
full root/Pages regressions were not rerun. Evidence: target/a46-workflow.
