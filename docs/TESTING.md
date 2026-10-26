# Testing and validation

## Checks by change type

| Change | Required checks |
| --- | --- |
| Documentation only | Markdown structure, local links, command accuracy, factual consistency |
| Rust/application/build/dependency changes | Formatting, Clippy, workspace tests, Pages-ready Trunk build |
| UI behaviour | Code gates plus Chrome e2e smoke covering changed behaviour |
| Test or CI changes | Relevant code gates and exercise affected harness/workflow where possible |

Always update [CURRENT](CURRENT.md) and [TASKS](TASKS.md), and append finished
outcomes to [COMPLETED](COMPLETED.md). For mixed changes, apply all relevant checks.
Unavailable gates are blocked, not passed; report prerequisites and failures.
Do not silently weaken gates to make the current scaffold appear complete.

## Commands

From the repository root:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --target wasm32-unknown-unknown -p zipeek
```

Use `cargo fmt --all` to apply formatting, then rerun the check. The wasm target
must be installed. The Cargo wasm check is a useful intermediate check, not a
replacement for the deployment build.

From `web/`:

```powershell
trunk build --public-url /zipeek/
trunk serve
```

Trunk must be installed; `web/Trunk.toml` selects Tailwind `4.3.3`. The configured
dev port is `1420`, not `8080`. Application builds require the approved Rust toolchain
and `wasm32-unknown-unknown` target. Installations and version changes are separate
from validation; dependency changes still need approval.

## Unit and wasm strategy

Use native unit tests for pure query and exercise logic where it can compile
natively. Browser-specific integration needs wasm checks and behaviour tests;
native workspace tests alone do not prove browser behaviour.

No dedicated wasm browser-test harness has been established. Introduce one only
as an approved task, including approval for any new dependencies. Match tests to
behaviour and risk rather than mirroring implementation details.

## Chrome e2e strategy (D09: approved 2026-10-26)

The existing manifest is named `zipeek-e2e` and specifies `thirtyfour = "0.38.1"`.
It is a workspace member with an executable and a headless integration smoke test.
Both require an external server on port 1420; shared lifecycle/readiness is absent.
The executable has headless mode commented out and suppresses the automation banner.
The smoke test asserts Home has an h1 reading Zipeek. The approved
target package name is `e2e` to support the invocation below; that manifest rename
has not been made in this documentation task.

Intended command after workspace/harness setup:

```powershell
# Repository root; currently unavailable
cargo test -p e2e -- --test-threads=1
```

Each test binary owns its own Trunk child and managed WebDriver session. Serial
execution avoids port and browser conflicts. Implement the shared setup/polling
helper once in the e2e crate and reuse it in every test binary.

### Chrome and driver ownership

Chrome must be installed locally and on CI. CI may use preinstalled Chrome on
`ubuntu-latest` or a Chrome setup action such as `browser-actions/setup-chrome`.
Create the browser session using:

```rust
let driver = WebDriver::managed(DesiredCapabilities::chrome()).await?;
```

Use thirtyfour's managed-driver support (the `manager` feature). It handles
matching/download of ChromeDriver, a free port, subprocess creation, and teardown
when the final connected WebDriver handle drops. Do not separately install
ChromeDriver, put it on PATH as a prerequisite, or manage its subprocess.
Driver download/cache availability is a prerequisite for the managed setup.
See the [thirtyfour managed-driver API](https://docs.rs/thirtyfour/latest/thirtyfour/struct.WebDriver.html#method.managed).

### Trunk lifecycle and readiness

From `web/`, the harness spawns `trunk serve --port <free-port>` as a child.
The harness owns a guard that kills and reaps the child on drop, including setup
failure and test failure. Trunk must already be installed.

Poll `http://127.0.0.1:<port>/` until any HTTP response is received, or use TCP
connection success to the same port. Use a total timeout of 30 seconds, starting
with 100ms backoff and doubling to a 2s cap. Bound each attempt/wait by the
remaining timeout. Do not use a fixed startup sleep. Report readiness failures
with the child exit status/output where available.

### Development base path and Pages validation

E2e starts Trunk **without** `--public-url /zipeek/` and navigates to
`http://127.0.0.1:<port>` at `/`. Do not add a `serve_base` configuration for v1.
The `/zipeek/` base path is a Pages deployment concern; validate its build with
`trunk build --public-url /zipeek/` separately from root-served e2e tests.

These are approved harness requirements, not existing implemented behaviour.

Planned smoke coverage includes the landing page, persistent theme switch, browse
queries/pagination, each of the three matching variants, Random, Word of the Day,
at the root-served development URL. The Pages build is checked separately with
`/zipeek/`. Resolve dependent product decisions first.

When these features are implemented, cover the approved requirements:

- HSK memberships can differ across systems or be missing in one; filtering must
  not treat equal numeric levels in different systems as equivalent.
- Round generation preserves four correct pairs, uses internal candidate reserves,
  widens HSK before dropping PoS, enforces visible right-label uniqueness, and
  reports insufficient pools. E2e sees exactly four cards per side.
- Endless offers Next round without a summary or persisted score. Fixed stops
  after N rounds and reports correct pairs, mistakes, and elapsed time with both
  exit actions. Test exactly 5/10/20 buttons, default 10, and no custom-N input.
- Random uses the shared generator, ignores setup filters, anchors to a seed,
  starts Endless, prefers HSK 3.0, and exercises ten invalid-membership retries,
  first-eligible fallback, and typed pool failure. Test D12/D13 contracts and
  use approved D18 eligibility and the resolved D19 call order for golden assertions.

During development, regenerate affected fixtures/artifacts after approved breaking
changes. Backward-compatibility tests for old generated DB formats are not required.

### Word of the Day coverage (D05)

- Navigate to `http://127.0.0.1:<port>/?wotd=2026-10-26` in debug e2e. Assert the
  specific expected word for the known fixture pool, approved hash, and ordering;
  reloading the same date/pool must reproduce it using D14's 64-bit FNV-1a.
- Check browser-local date handling, including a case where local and UTC calendar
  dates differ. The override represents a calendar date, not a UTC timestamp.
- Verify the full pool is used independently of practice filters, and offset
  selection uses deterministic ordering with tie handling.
- An empty pool shows "No word available" without a panic; a DB load failure
  shows the same card plus a working retry action.
- Verify release/production builds ignore `wotd` regardless of validity. Ordinary
  debug Trunk e2e alone cannot establish that release-build behaviour; use an
  appropriate separate release check when implementing the feature.
- Verify the exact FNV-1a basis/prime with fixed input/output vectors, wrapping
  multiplication, UTF-8 inputs, and `u64` arithmetic independent of pointer width.
  Word of the Day must never use DefaultHasher or persist hash state.
- In debug builds, wrong format, impossible dates such as `2026-13-40`, and dates
  outside the supported range silently fall back to real local date. No page error
  or panic; any warning is debug-only. No testing Cargo feature is needed for v1.

### Practice URL, exclusion, and offset coverage (D13)

- Round-trip mode, Fixed count, HSK context, exclusions, and unseeded offset or
  seed. Verify malformed/unknown values safely default to Endless, 10 rounds,
  HSK 3.0, level 1, and offset 0. Invalid level must never broaden to all levels.
- Test HSK 3.0's combined advanced tier as `level=7`, alias `7-9` normalization,
  UI display as one 7–9 band, and rejection of separate levels 8/9. HSK 2.0 has
  no level 7. PoS strings use the DB's canonical lowercase vocabulary; comma sets
  retain known entries and ignore unknown entries; empty/absent means no filter.
- Seeded Random emits the seed's PoS and HSK membership, including advanced tier
  7 where applicable. A later setup PoS picker must use the same URL contract.
- Require counts only for Fixed; ignore them for Endless. Seeded rounds use
  Endless and effective offset zero. Fixed+seed ignores seed; invalid/excluded
  seed becomes unseeded while preserving the entered filter and offset.
- Verify same-URL/same-DB replay across reloads and browsers using D12's canonical
  input and ordered candidate views. Use D17's rand 0.10.3/chacha provider and
  the approved D19 draw/shuffle sequence when recording full-round golden vectors.
- Store excluded WordIds in localStorage, serialize nonempty comma-separated IDs
  in links, omit empty exclude, and filter before all candidate selection/widening.
  Test setup's excluded count and Reset, and the insufficient-after-exclusions
  message/reset action. Generated URLs union saved/current-round IDs; loaded
  URLs use exactly their own set, and absent exclude means empty. Test sharing
  across browsers with different saved sets, no writes on link load, unknown IDs,
  deduplication, and noninteger entries dropped with debug-only warnings. Marking
  a new exclusion must not change current URL/cards; it applies to the next URL.
  Setup Reset clears storage/pending state and emits a URL without exclude.
- Verify offsets 0, 4, 8; distinct unchanged strict slices until exhaustion; wrap at
  offset + 4 > pool_size; one looping hint; Play again resets to zero. Fixed must
  still finish after N rounds even when wrapping occurs. Reload resets in-memory
  progress while honoring URL offset; do not infer progress from a wrapping offset.
  Sliding and widened offset-zero fallback may overlap correct sets before wrap.

### URL fallback and session coverage (D15; planned)

- Missing, malformed, out-of-range, or wrong-system levels use 1 in the selected
  HSK system, with debug-only warnings for invalid values. No panic or all-levels
  fallback; test level=99, level=abc, and HSK 2.0 level=7.
- Malformed/nonexistent seed, or a seed in the URL exclusion set, becomes unseeded
  with the original filter/offset. Exclusion only in another browser's saved set
  must not change that shared round. Validate seed before offset-zeroing.
- Fixed+seed preserves Fixed/count/filter/offset, ignores seed and warns only in
  debug. The next generated URL omits the ignored seed.
- Membership-less seed fixtures: two independent URL-normalization/build pipelines
  produce identical recovered seed and Round. Ten ineligible ChaCha draws must
  fall back to the first eligible word by id. No ambient RNG on URL reads and no
  URL replacement with the recovered seed. An excluded seed takes the unseeded
  path instead of membership recovery. Cover an empty recovery pool safely.
- Mark a word mid-round: saved/pending state changes, active URL/cards do not.
  Next URL includes the new exclusion; Setup Reset clears it and omits exclude.
- Reload Fixed at a nonzero offset: progress starts at zero while the pool offset
  is preserved. Wrap mid-session without resetting progress; finish after N rounds.
  Do not persist the counter or score.
- Seeded Next produces unseeded Endless with the same filter and offset 4 before
  normal pool handling. No seed or raw ignored offset survives into that round.

### Deterministic round builder coverage (D12; planned)

Use plain CandidatePool/Word fixtures for native unit tests; no wasm is needed
to exercise the pure builder. Later compare full golden outputs against wasm32
with the approved dependency versions/features.

Verify the generator is `rand::rngs::ChaCha12Rng` with rand 0.10.3's `chacha`
feature, seeded through `rand::SeedableRng`. No direct rand_chacha dependency or
StdRng substitution. The feature is approved and enabled in web/Cargo.toml;
the builder remains absent.
Use the current `sample` and `rng()` APIs rather than deprecated aliases, retaining
the existing `-D warnings` gate.

- Permute PoS insertion order and include duplicate PoS inputs: canonical bytes,
  FNV-1a, and round output must match. Empty PoS produces an empty field, and an
  absent seed is literal lowercase `none`, not Debug's `None` or `{}`.
- Pin the canonical encoding with the fixed ASCII string
  `zipeek-round-v1|eng-pinyin|hsk3|3|noun,verb|10|none` and expected `u64` hash
  `0xa0e531d76df62844`. Changed separators, tokens, ordering, or whitespace must fail.
- Vary raw URL offset for a seeded round: boundary normalization forces zero
  before canonical formatting, so the hash and round stay identical. Unseeded
  canonical input retains entered offset even after sliding or fallback.
- Verify each caller query applies exclusions and `ORDER BY id ASC` in SQL,
  and produces nested strict/widened/any-PoS views. Check same-system boundaries:
  HSK 2.0 level 6 -> {5,6}, level 1 -> {1,2}; HSK 3.0 level 7 -> {6,7},
  level 1 -> {1,2}. No level 8/9 or cross-system candidates.
- A fixture whose first clean window is at slide 3 must choose it. A view with
  no clean window must fall through to the next view, independently of the cap's
  numeric value. Each fallback starts at zero and has its own attempt cap.
  Do not assert `MAX_SLIDE == 16` or test the constant instead of the behaviour.
- Check exclusion-before-offset indexing, wrap when fewer than four entries
  remain at the entered position, first-clean-window selection, retained entered
  URL offset, and internal-only effective start.
- Seeded fixtures always retain the seed, select three distinct-label partners
  with ChaCha, and never slide. Distractors exclude all correct IDs/seed and
  normalized-label collisions; rejection is bounded by min(candidate_count,50),
  then falls through and reapplies dedup in the next view.
- Test actual right-column keys only: homophones collide for eng-pinyin;
  行 with different readings collides for pinyin-hanzi; English case/whitespace
  variants collide for hanzi-eng. `to eat` versus `eat` remains distinct.
- Empty/pathological bundles exhaust all views into InsufficientPool without
  panic. UI displays the distinct-words message and level/reset recovery controls.
- Random entry retries seeds lacking membership at most ten times, then uses
  the first eligible ordered word; if none exists, fail safely. Prefer HSK 3.0
  for dual memberships. Ambient RNG must not be used inside the pure builder.

See [ROUND_GENERATION](ROUND_GENERATION.md) for the complete contract and replay
limitations of changing dependencies, tuning, or the v1 algorithm.

### Single-stream and call-order vectors (D19; required, not implemented)

- Record original canonical bytes/hash, recovered ID, accepted/rejected candidate
  requests, selected correct IDs/reserves/options, and final label layout for
  native and wasm32 fixtures using rand 0.10.3. Do not call these tests passing
  until the builder and fixtures exist.
- Cover recovery succeeding on attempts 1 and 10, ten failures followed by the
  first eligible id, empty/no-eligible recovery pools, and D18 missing-English
  recovery. Verify early success consumes no further recovery requests and the
  by-id fallback consumes no RNG. Hash the original URL seed, not the recovered ID.
- Verify one RNG initialization and continuation into generation, including
  failed views; no reseeding, cloned-stream replay, or caller-owned second RNG.
- Match vectors must lock correct selection, four accepted reserve distractors,
  then left shuffle followed by right shuffle; unseeded windows use no RNG.
  Seeded partner draws remove attempted rows while preserving remaining id order.
- MCQ vectors must lock prompt selection, two accepted distractors, then one
  shuffle of initially ordered [correct, distractor_1, distractor_2].
- Include collisions, the bounded rejection limit, failed-view continuation
  without shuffling, repeated recovery candidates, and an unchanged amount-1
  slice `sample` primitive. API attempts are not counts of internal RNG words.
- Keep the existing v1 hash vector. Verify no `v=` selector/parser is added; a
  future replay-changing sampler/order/tuning change requires the approved
  prefix/URL-selection/compatibility path before replacing golden outputs.

## MCQ coverage (D20; planned)

Cover all three `/practice/mcq/<variant>` routes, one correct option and two
distinct-answer distractors, D02/D12 widening, exclusions and D18 eligibility.
Verify Endless/Fixed, counts 5/10/20 (default 10), and offset advancing by one
per question independently of Fixed progress. UI-detail assertions wait for A22;
exact sampling follows the resolved D19 sequence. Implement full-round golden
vectors with A07/A10; MCQ routes are not implemented.

## Startup loading coverage (planned)

- Delay app bootstrap to verify the static HTML loading shell is visible before
  Yew mounts. Check its theme/background/loading indicator without depending on
  the database or Yew having initialized.
- Delay database loading independently of bootstrap. Verify Yew's first render
  is Loading and that the loading view remains visible after mount while the
  database is fetching/opening; no blank or premature ready content.
- Verify Ready appears only once the read-only database is queryable, including
  when its valid word pool is empty (D05's neutral card, not a stuck spinner).
- Simulate a failed download/open, verify loading ends in the retry state, and
  verify Retry transitions through Loading to success or another handled failure.

Use controlled delays/failures when implementing this coverage; fixed sleeps do
not establish the loading-state lifecycle. The harness and loading flow remain
unimplemented, so these are test requirements rather than passing smoke tests.

## Theme, sidebar, and footer coverage (planned)

- With no saved selection, verify dark is the default even if the OS prefers light.
  Toggle to light and back to dark, reload, and navigate routes to verify persistence.
  A saved light choice must override the preferred dark default.
- Check the same theme during static HTML loading, Yew loading, and ready content;
  theme initialization must avoid a light flash during the startup handoff.
- Verify the shared left sidebar has the theme toggle and a bottom-anchored GitHub
  icon whose link is `https://github.com/Jozefpodlecki/zipeek`, with an accessible name.
- Verify the footer shows Jozef Podlecki, the copyright symbol and current year,
  and the required CC-CEDICT attribution when dictionary data is served.

These are future UI smoke requirements; this documentation task changes no UI.

## PoS mapping fixture tests (D18; required, not implemented)

The db-gen crate is a stub. Add native unit tests against fixture source entries
when implementing it; do not report these tests as already passing.

- Cover every one of DB.md's 42 source-code/canonical-value pairs with fixture rows.
- Lookup is case-insensitive: Mg/mg produce numeric_morpheme, Rg/rg produce
  pronoun_morpheme, and ordinary uppercase forms such as V produce verb. Stored
  pinyin and hanzi stay byte-identical; case folding applies to source PoS lookup.
- Include recognized multi-PoS entries and repeated case variants; produce only
  recognized canonical lowercase strings without duplicate canonical tokens.
- An unknown code alone rejects the entire word/all forms and emits a warning.
  Mixing a recognized code with an unknown code must still reject the whole word,
  not merely drop the bad tag or retain the word with NULL PoS.
- Include pinned-source unknowns cc/mq/qt/qv. They remain unknown, with no guessed
  expansions. The source audit predicts 64 rejected entries/105 associated forms
  before other import checks; this is not a generated-row count or a test result.
- Missing/empty PoS is NULL, distinct from a present unknown code. Verify skips
  occur before merging/ID assignment and are reflected in reproducible output counts.
- Verify q maps to classifier and y to modal_particle throughout DB/filter/seed
  tokens; the old illustrative measure_word/particle names are not canonical.
- Keep URL parsing separate: unknown pos query tokens follow D13's ignore policy,
  rather than triggering source-entry rejection.

## Data provenance and generation checks (D06/D16; planned)

Use complete.json from exact commit `a66fd30b9580da2c2af7eb19e4b9d8099a29c061`.
Retain its MIT notice, source attribution, retrieval/checksum information, and
transformation record. Verify notices for the separately pinned CC-CEDICT fallback.
The superseded leonsilicon packages must not be imported as substitutes.

## Schema and import validation (D07/D16; planned)

Validate one words table, its ten columns, nullable metadata, PK/display-key
UNIQUE constraint, and two ordinary HSK indexes. Do not add non-HSK dictionary
entries. Report actual source-entry, expanded-form, grouped-reading, and dropped
row counts; the audit's 11,494 entries/12,655 forms are not a generated DB size.

- Group by simplified plus exact numeric pinyin, retain the first traditional
  form, and warn on repeated keys. Test different readings remain separate and
  both directions of numeric/display-key correspondence hold for D07 uniqueness.
- Map old-1..6 and new-1..6 independently; both new-7 and new-7+ map to 7.
  Deduplicate identical tags; conflicting within-system levels choose the minimum
  and warn; unknown tokens are logged/skipped rather than inventing memberships.
- Preserve both source transcriptions exactly, including spaces, case, ü and
  unnumbered neutral syllables. Do not rederive display pinyin. Missing/blank
  required pinyin is dropped and logged.
- Trim nonempty meanings, preserve casing/order, and serialize with pipes.
  A nonempty primary list must never invoke CC-CEDICT. Empty primary meanings
  invoke reading-disambiguated fallback; ignore variant-of entries. A failed
  fallback writes NULL English and warns without dropping a usable pinyin row.
- Verify first-gloss labels for both English-bearing variants and variant-specific
  eligibility: missing English is allowed for pinyin-hanzi only. Rendering and
  normalized duplicate keys must use the same chosen gloss.
- Map raw PoS codes using D18's case-insensitive canonical map and whole-entry
  unknown-code rejection; preserve supplied
  frequency ranks and NULL missing ranks. Verify complete-token PoS filtering.
- Assign IDs with row numbers ordered by simplified/numeric pinyin after grouping.
  Repeat identical input to reproduce IDs; changed snapshots may shift them.
  Assert replay/exclusions only for the same DB snapshot.
- Exercise schema creation and file loading with the approved native generator
  and pinned DuckDB-WASM runtime when implemented. D18 fully approves comparison-key,
  literal-pipe handling, key-conflict and missing-label seed rules in DB.md. D19
  fixes the recovery/build stream; its golden vectors remain implementation work.

## Current CI and target coverage

- `ci.yml` currently runs `cargo test` on Windows using Rust `1.97.0`, triggered
  by pushes to `master` or manual dispatch.
- `deploy.yml` builds on Ubuntu using root-level npm/style/Trunk paths from the
  previous layout and no explicit `/zipeek/` public URL.
- Full fmt, Clippy, workspace, wasm, and Chrome smoke coverage is intended, but
  the exact CI matrix is not implemented. PR-triggered validation also needs work.

Track these gaps in TASKS/CURRENT. Editing workflows or fixing application build failures
requires a separate approved implementation plan.

## Generator foundation acceptance (D21/D22 approved)

A24 checked public release metadata and inspected crate source archives without
installing dependencies or generating a database. The exact dependency/input
accepted plan and evidence are in [DB](DB.md#a24-generator-dependencysource-audit-2026-10-26).
D21/D22 are approved. A25/A02/A05/A06 must validate:

- The accepted dependency pins/features and workspace lockfile policy, including
  the user-restored neo Git revision. At every generator invocation,
  query SELECT version() on the native engine, log the raw result, and fail
  unless it is v2.0.0 before any import/output (D23). Set and query-verify
  storage_compatibility_version=v1.4.0 on the output connection before creating
  any output. Test wrong releases, development suffixes, query failure and the
  actual configured storage; metadata alignment is not a runtime pass.
- SHA-256 mismatch/missing-file rejection before parsing any input; preserve the
  primary commit and fallback header/notices. Check byte-exact approved snapshots
  after Git checkout; the audited fallback has CRLF, so attributes must prevent
  line-ending conversion. Reject a changed moving upstream download.
- A02/A05: native schema, IDs, two HSK indexes, storage settings and checkpoint/close.
  A06: read-only browser queries against that same generated artifact with
  exactly WASM 1.32.0. A direct neo 2.0 synthetic fixture passed read-only queries in the actual
  WASM 1.32.0 Node engine with STORAGE_VERSION v1.4.0; default native storage
  failed to open. Final-dictionary/browser integration remains A06 coverage,
  not a prerequisite/blocker for A25. D23 now approves native v2.0.0 and
  mandatory compatible storage; its guard/storage native tests pass.
- Approved D22 logical reproducibility: compare every column in ORDER BY id
  order, schema/index definitions, and deterministic import/drop diagnostics
  across independent runs of verified inputs and pinned tooling. Record each
  final file checksum separately for integrity; do not require identical file
  bytes or equal file hashes across runs.

A25 preserves approved source files, notices, MANIFEST.toml and Git attributes,
and adds engine-version tests. A26 implements and tests runtime input/notice
verification and preparation. Source import and generated-database behavior
coverage remain A03/A04/A05/A06 work. Do not hand-edit
DuckDB bytes or mask physical differences to make a checksum comparison pass.

## PR checklist

- Scope and any dependency/schema/decision changes were explicitly approved.
- Applicable checks passed, or failures and missing prerequisites are reported.
- UI changes include Chrome smoke results when the harness is available.
- Pages builds retain `/zipeek/`.
- CURRENT/TASKS, completed outcomes, and affected architecture/product/DB/testing documents are updated.
- The PR describes resulting behaviour and validation, with relevant failure output.


## Generator preparation coverage (A26; implemented)

From the repository root, `cargo test -p db-gen` runs 19 tests: five database
unit tests and fourteen integration/CLI tests. Database tests cover actual
native/storage versions, rejected versions/storage, failing SQL context, and
NULL/empty scalar results. Preparation tests use copied local fixtures to
exercise source/notice tampering, missing files, sizes, malformed TOML/digests,
policy overrides, parent paths, duplicate files and undeclared notices, plus
successful preparation. A retained-buffer test proves a changed source file
cannot replace the bytes already verified. CLI tests verify the actual pinned
sources and reject invalid logging configuration. No network or generated
DuckDB artifact is needed for these tests; the database connection is in memory.

Fixtures live under crates/db-gen/tests/fixtures/inputs; native pure/database
unit tests stay beside their implementation. main.rs contains only CLI
entry-point delegation. No new test dependency or lint suppression is added.


## Schema and shared-domain coverage (A02; implemented)

cargo test -p zipeek-core -p db-gen --offline passes 29 tests: the prior 19
preparation/database tests, six shared-domain tests, and four native schema
integration tests. The shared fixture covers all 42 code/token pairs and case
aliases; unknown source codes reject the entire list. HSK tests reject separate
8/9 levels and check the combined tier. WordId tests cover SQL integer bounds.
The SQL fixture verifies exact columns/types/nullability, both HSK indexes,
primary/display uniqueness, distinct readings, Unicode and NULL metadata.
A preparation test confirms the real pipeline creates an empty words table.
No importer or final-dictionary reproducibility test is claimed yet.
