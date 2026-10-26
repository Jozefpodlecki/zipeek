# Current state

The reviewed A18 scope and amendments were approved on 2026-10-26. D18 is fully
approved in [DB §D18](DB.md#d18-source-comparison-and-serialization-approved);
D20 MCQ scope is recorded in DECISIONS/PRODUCT/UX. MCQ scope and D18 are no
longer blockers. TASKS, CURRENT, and append-only COMPLETED own live task state.

## Active

- A03 doing: verified HSK JSON parsing, diagnostics, reading grouping, deterministic
  IDs and transactional in-memory insertion. No fallback or file output yet.
- A02 is complete: exact approved words schema and two HSK
  indexes, checked WordId/HSK types, Word and all 42 canonical PoS mappings.
  Preparation creates the empty in-memory table after source/engine verification.
  All 29 domain/generator tests plus Chrome smoke pass; all five gates pass.
- A26 implemented modular generator preparation: thin CLI,
  orchestration, guarded DuckDB connection, typed manifest, verified sources and
  notices, retained input bytes, and actionable errors. All 19 generator tests
  and the Chrome landing smoke pass. All five quality gates pass.
- A20 formatting and app import repairs are complete under the user's direct
  coding-standard request. A19 lockfile policy remains independent and open.
- A25/D23 engine v2.0.0 and storage compatibility v1.4.0 are implemented and
  verified. D21 source/dependency and D22 logical-reproducibility policies apply.
- A23/D19 keeps v1, one recovery/build RNG stream and locked sampling order;
  full-round golden vectors remain A07/A10 implementation work.
- Workspace members: web, db-gen, zipeek-core and zipeek-e2e. The generator
  prepares verified inputs, a connection and the approved empty schema; import
  and artifact generation remain future work. Shared domain types are implemented;
  UI pages remain scaffolds.

## Blockers

- D21 dependency/source and D22 policy approvals are recorded. Input verification
  and schema creation are implemented; import/artifact generation remain unimplemented. The exact approved source
  bytes, provenance manifest and notices are present for review, not committed
  or imported. No generated DuckDB database exists.
- A19: root Cargo.lock remains ignored; transitive-lockfile tracking still needs
  an explicit policy choice. The approved exact direct pins do not freeze every
  transitive dependency.
- A22: 13 UX TBD markers remain; UX-MCQ01 now covers only feedback timing,
  number-key hints, and answer-button styling, not MCQ scope.
- A15: Chrome landing smoke passes with a separately started, ready Trunk
  server. The existing harness still requires an external server on port 1420;
  owned Trunk lifecycle/readiness and package rename to e2e remain open.
- A16: CI/deploy select Rust 1.97.0, while policy/manifests require 1.97.1;
  deployment still uses obsolete root paths and omits the Pages public URL.

## Other recorded follow-up

CODING_STANDARDS permits unreachable-branch comments at line 52 but declares
lint-suppression rationales its sole comment exception at lines 139–154. Resolve
that wording before adding exception comments; A26 introduced no Rust comments.
Local Rust/npm/wasm configuration already matches the declared targets; A19
verifies remaining configuration/pin-policy gaps without assuming dependency approval.
[BASELINE](BASELINE.md) remains unchanged historical evidence, not current results.

## A18 validation (previous task)

- Formatting: failed on the baseline core/e2e/page files (repaired in A20/A26).
- Clippy: failed on Timeout and components::* imports (repaired in A20/A26).
- Wasm check: passed with those two warnings.
- Pages build: passed with those two warnings; Tailwind 4.3.3 completed.
- `cargo test --workspace`: failed in `landing_page_loads` after 121.16s:
  `WebDriverError` status 500, `tab crashed` (Chrome 156.0.8078.47). Five
  unit-test targets had zero tests; the one integration smoke failed. No browser
  assertion pass is claimed. A Trunk launch attempt failed because port 1420 was
  occupied; the existing server responded HTTP 200 and was left running.
- Documentation checks passed: 16 Markdown documents, 86 local links/anchors,
  balanced fences, 21 unique open tasks in prerequisite order, and 13 UX TBDs.
- Scope checks passed: BASELINE unchanged; prior COMPLETED prefix preserved;
  only approved package-name/Home code changes; user practice.rs edit preserved;
  dependencies, workflows, app imports, and replay contract unchanged.
- `git diff --check`: passed.

## A23 validation

- Passed: 16 Markdown documents, 88 local links/anchors, balanced fences,
  20 unique open tasks in prerequisite order, and `git diff --check`.
- The existing v1 canonical hash remains `0xa0e531d76df62844`.
- Installed rand 0.10.3 source confirms slice IndexedRandom::sample and
  SliceRandom::shuffle API calls; no dependencies or application code changed.
- Prior COMPLETED history and BASELINE are preserved. No version URL parser
  was added. Rust/Trunk/Chrome gates were not rerun for documentation-only work;
  the A18 results above remain previous-task evidence.

## A24 validation

- Passed: 16 Markdown documents, 89 local links/anchors, balanced fences,
  parseable proposed dependency TOML, 20 unique open tasks in prerequisite order,
  preserved code/manifests/lockfiles/prior history, and `git diff --check`.
- Python hashlib and PowerShell Get-FileHash independently verified all three
  documented source checksums. Fallback gzip/UTF-8 validation and its 125,320-row
  count passed. Exact crate versions were non-yanked when checked; native crate
  archive checksums matched registry metadata. Inputs exist only in audit temp
  storage, not committed generator inputs or application assets.
- No dependencies were installed, no database was created/opened, and no native
  to WASM runtime compatibility test was run. Rust/Trunk/Chrome gates were not
  rerun for this documentation-only audit; earlier failures remain recorded.

## Earlier A25 bundled-build validation (superseded by neo restoration)

- Input/notice hashes, provenance manifest, attributes, scope and documentation
  checks are being finalized; sources are preserved in the working tree.
- `cargo fmt --all -- --check`: fails on existing core/e2e/page formatting;
  the changed db-gen main is formatted.
- `cargo clippy --workspace --all-targets -- -D warnings`: fails on native
  compilation and the existing two app.rs imports.
- `cargo test --workspace` and focused db-gen tests: native compilation fails;
  no tests/engine query pass is claimed and Chrome smoke was not reached.
- Wasm check: passed with two existing import warnings.
- Pages build with /zipeek/: passed with the same warnings and Tailwind 4.3.3.
- No generated dictionary artifact or native-to-WASM test exists. Root lockfile
  resolution changed locally through Cargo; tracking policy remains A19 work.

## Earlier neo restoration and requested compatibility check

- Native cargo check passes. Neo SELECT version() reports v2.0.0.
- Actual WASM 1.32.0 in Node reports engine v1.4.3. Its read-only open rejects
  the default native file (storage version 999, reader supports 64..67).
- An independent file written with STORAGE_VERSION v1.4.0 passes: eight full
  rows, ten columns and both HSK indexes match, including Unicode and NULLs;
  DELETE is rejected in read-only mode. Scratch tools/files/results are under
  target/a25-compat. This is a synthetic fixture, not the generated dictionary.
- Focused native tests: rejection test passes; required-version integration
  test fails because v2.0.0 differs from the unchanged v1.4.3 requirement.
  CLI prints a clear mismatch and exits 1. No successful generation is claimed.
- Formatting and Clippy retain baseline failures. Wasm check and Pages build
  pass with the two existing app.rs import warnings. Workspace tests remain
  blocked by the native required-version test; no Chrome pass is claimed.
- Documentation/scope checks passed: 17 Markdown files, 93 local links/anchors,
  balanced fences, 20 open tasks in prerequisite order, preserved source/notice
  bytes and prior history, actual/documented neo manifest agreement, and
  git diff --check.
- Browser worker/HTTP/full-dictionary integration remains A06 coverage. No
  client/npm app dependency or WASM pin was changed by the isolated probe.

## Final A25 / D23 validation

- Passed: three native generator tests and manual invocation, logging engine
  v2.0.0 and storage compatibility v1.4.0. Wrong-version rejection is covered.
- Passed: wasm check, /zipeek/ Pages build, 17 Markdown files, 93 local links/
  anchors, five source/notice hashes, metadata consistency, and git diff --check.
- Existing failures remain: workspace fmt; Clippy app.rs imports; Chrome smoke
  status 500/tab crashed on Chrome 156.0.8078.47 after 121.26s. The three
  generator tests passed within the workspace run before the browser failure.
- Native-to-WASM synthetic fixture compatibility passed with explicit storage;
  production dictionary and browser/HTTP integration remain future A06 work.

## A26 / A20 validation

- Passed: cargo fmt --all -- --check; cargo clippy --workspace --all-targets
  --offline -- -D warnings; cargo test --workspace --offline (19 generator tests
  and Chrome landing smoke); cargo check --target wasm32-unknown-unknown
  -p zipeek --offline; trunk build --public-url /zipeek/ from web/.
- Chrome passed outside the process sandbox with a temporary root-path Trunk
  server after HTTP readiness; the owned temporary server was stopped afterward.
  This does not implement the A15 harness lifecycle.
- Manual CLI preparation verified two sources, three notices and 19,567,208
  bytes, plus actual engine v2.0.0 and storage compatibility v1.4.0.
- No new dependency, schema, replay or source-byte changes. Earlier validation
  sections remain historical evidence; current formatting/Clippy/Chrome pass.

## A02 validation and next-session handoff

- Passed: fmt, workspace Clippy -D warnings, workspace tests (29 domain/generator
  tests plus Chrome smoke), wasm check for both zipeek and zipeek-core, and Trunk
  build --public-url /zipeek/ from web/. Logs: target/a02-workspace-tests.log.
- Chrome used the existing HTTP-200 port-1420 server outside the process sandbox;
  a new Trunk launch could not bind. Server ownership was not confirmed and no
  existing process was killed. The earlier A26 cleanup assertion was too strong;
  its corrective note is appended in COMPLETED. A15 remains open.
- No new dependencies, source snapshot modifications, schema-policy changes,
  generated dictionary, replay changes, staging or commits.

## Next up

**A03 ? import the pinned HSK source** is the next coding task. Read DB's source,
D18 and PoS sections before implementing. Reuse PreparedGeneration.inputs()
verified buffers; do not reopen files or fetch sources. Parse the preserved
primary JSON, map source codes through PartOfSpeech::from_source_codes, and
log/drop unknown-code entries including all forms before grouping/ID assignment.
Keep stored text exact; group by simplified/numeric reading, preserve first-form
traditional/display values, lowest within-system membership with warnings, and
check display-key conflicts. Assign positive INTEGER IDs only after deterministic
sorting. Source-level parser/grouping/serialization modules belong in db-gen;
shared types already live in zipeek-core/{hsk,part_of_speech,word}.rs. Word is a
transport record: A03 must validate membership and required labels itself.

Schema creation is already in db-gen/src/schema.rs and schema/words.sql, called
by prepare() after verification and engine/storage guards. It is empty today.
A04 adds empty-only CC-CEDICT fallback; A05 creates/checksums the actual artifact;
A06 tests full-file browser/HTTP compatibility. A07/A08 are independently ready.
A19 lockfile policy and A22 UX decisions remain open and do not block A03.
