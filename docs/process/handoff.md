# Next-session handoff

Per-task continuation notes moved out of the live [CURRENT](CURRENT.md) state.
These are reference prose for the next session or a new contributor picking up a
task; they are not live task state. Live work is in [CURRENT](CURRENT.md) and
[TASKS](TASKS.md); completed outcomes in [completed/](completed/).

Preserved verbatim from the former single-file `CURRENT.md`. Do not rewrite.

---

## A34 handoff

Implement A15 first using [testing/e2e.md](../testing/e2e.md)'s explicit
directory/ownership and cleanup plan; then review A31's scoped state plan before
expanding feature UI. A32 motion profile and A33 crate extraction remain separate
proposed scopes. Preserve current source files, staged changes and A06 test
evidence; doc-only validation did not rerun Rust/Trunk/Chrome. No implementation
completion is claimed.

## A19 handoff

Root `Cargo.lock` is authoritative and staged; only its new index entry was added.
Existing staged files were preserved. The stale `web/Cargo.lock` is deleted,
`.gitignore` no longer ignores the root lock, and `Trunk.toml` sets `locked=true`
to prevent silent resolution changes. Watch paths include `web`, shared core and
root `Cargo.toml`/`Cargo.lock`/`rust-toolchain.toml`. Shared-core watch proof
passed without changing bytes; original timestamp restored after owned-server
shutdown. All five gates pass with locked resolution; log
`target/a19-workspace-tests.log`.

Tailwind 4.3.3 npm packages are absent locally, but Trunk's managed CLI works;
global wasm-bindgen is absent from PATH, but its Trunk pin is 0.2.114 and build
passes. Do not install/upgrade tooling to correct these observations. Local
Trunk 0.22.0 is tested, not an approved exact pin. A16 still owns CI Rust 1.97.0
and obsolete root-path/Pages workflow fixes.

A19 scope checks: working diff and newly staged `Cargo.lock` pass whitespace
checks. The full pre-existing staged diff reports whitespace in fallback
`NOTICE.txt` and `hsk.json`, and static `CC-BY-SA-4.0.txt`/`HSK-SOURCE-README.txt`
notices. These protected fixture/source bytes were preserved; no normalization
was attempted. Details: `target/a19-existing-staged-whitespace.log`. This does
not fail a Rust/build gate.

## A06 handoff

Run `npm ci --ignore-scripts` from `web/` before Trunk. The `pre_build` hook
verifies package 1.32.0 and copies matching local MVP/EH workers/WASM, Arrow
bundle/adapter and licenses to ignored `web/vendor/duckdb`. HTML copies it; watch
ignores vendor to avoid loops. No CDN, automatic `npm install` or DB generation.

`bridge.js` resolves assets/dictionary via `document.baseURI` for `/` and
`/zipeek/`, opens a full fetched buffer `READ_ONLY`, prepares/binds statements
and closes them. `runtime.rs` owns async Rust handles/cleanup; `row.rs` validates
`WordId`s/levels/PoS without changing source display text; `queries.rs` provides
lookup, full-pool count/ordered offset/rows, pagination and the three clamped
HSK/PoS views.

`DatabaseProvider` inside `AppShell` starts Loading before effects, keeps
navigation usable, renders routes only after a typed probe, and exposes concise
failure plus Retry. `contexts/database.rs` supplies `DatabaseContext`/
`DatabaseState` for future route queries. The existing practice `Loader` is still
a feature placeholder; do not interpret it as DB readiness. No Browse/daily/
exercise UI was implemented.

Tests/logs: `target/a06-workspace-tests.log` (88 domain tests + seven Chrome
tests; one Pages test ignored there), `target/a06-pages-chrome.log` (that Pages
test passed separately). Node probe `target/a06-compat.cjs` also passed. Both
owned servers stop after validation. Dictionary and background bytes, Cargo graph
and replay remain unchanged. No commit/staging changes were made in A06.

`npm audit` reports existing tooling picomatch 2.3.1 with a high-severity
advisory (`target/a06-npm-audit.json`). It was already in the previous npm
lockfile; existing package records were preserved and Picomatch is not copied
into browser assets. A16 tracks review; dependency upgrades require specific
approval. Do not run `npm audit fix` as an unapproved follow-up to this task.

## A07 handoff

`core/round` exports `build_round`, `RoundCandidates`, `CandidatePool`,
`ContextPool`, `Round`/`RoundContent`, `Card`, `BuildTrace` and typed
`BuildError`. Modules separate orchestration, pool validation, labels, recovery,
selection and final layout. Pass normalized A08 state, ordered views and original
seed metadata. For Ready seeds, initial views use the seed's effective context.
For recovery, provide ordered exclusion-clean rows and `ContextPool` entries for
possible recovered contexts; missing views fail explicitly. Canonical input
retains the original URL seed/filter; never recover with a second caller-owned
RNG. A06 now supplies SQL ordering/exclusion binding and DB readiness; A21 must
wire those APIs to route normalization and the builder.

Matching reserves stay in trace only; visible columns have four cards, MCQ has
one prompt and three options. English cards use the first pipe-separated gloss.
No gameplay/score controls are connected to the placeholder pages yet.

`tests/fixtures/round-golden.txt` and the shared `replay_vectors` example lock 18
complete rounds; `tests/check-round-vectors.mjs` runs the wasm export with host
calls trapped. See [testing/round-builder.md](../testing/round-builder.md) for
exact build/run commands and why web participates in wasm feature unification.
All five gates and wasm replay pass. Latest log:
`target/a07-workspace-tests.log`; the owned server is stopped. No Git commit made.

## A08 handoff

`crates/zipeek-core/src/practice` separates types, query parsing, normalization,
canonical encoding and session state. `ParsedPracticeQuery` takes already-decoded
pairs. `normalize` takes an explicit `Word` lookup, drops unknown exclusion IDs
and returns `Unseeded`, `Ready` or `RecoveryRequired`. The latter keeps the
original bad seed/filter at offset zero for canonical hashing; A07 owns all RNG
initialization, recovery, selection and shuffling. Do not resolve seeds against
an empty placeholder database. The web adapter only parses today; A21 must
provide rows using A06 APIs and call normalization. `query_pairs` returns decoded
values for one normal router serialization; no additional percent-encoding.

Saved/pending exclusions are unioned only for explicit URL generation. Offset
helpers advance match by four and MCQ by one; `FixedProgress` is separate from
offset and resets in memory. Match pages still render `Loader`, not a game.
The published canonical golden remains `0xa0e531d76df62844`. Checks/logs are
recorded in [completed/](completed/) and `target/a08-workspace-tests.log`; no
owned server remains running.

## A06 asset packaging notes

`npm ci --ignore-scripts` from `web/` runs the pre-build hook that packages exact
DuckDB-WASM 1.32.0 and locked Arrow assets from `node_modules` into ignored
`vendor/`. It verifies the runtime version, copies matching workers/WASM and
licenses and does not fetch/install anything itself.

The npm tarball lacks DuckDB's MIT license file; the pinned upstream MIT text is
included alongside Arrow's shipped license/NOTICE. Do not substitute `latest`,
`next`, or development builds. If integration exposes a genuine wasm toolchain
compatibility blocker, stop and report commands, versions, and error evidence.
See [architecture/schema/DB.md](../architecture/schema/DB.md) and
[architecture/schema/runtime.md](../architecture/schema/runtime.md) for the pin
rationale.