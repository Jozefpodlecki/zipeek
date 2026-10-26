# Architecture

## Current repository snapshot

Documented on 2026-10-26 during an existing, staged repository migration.
This describes files on disk, not the last committed release.

- Root `Cargo.toml` includes `web`, `crates/db-gen`, `crates/zipeek-core`, and `e2e`.
- `web` is the Rust 2024 Yew CSR package `zipeek`. `main.rs` initializes the
  environment, logging, and renderer; `lib.rs` declares the app modules.
- `web/src/` contains `app`, `components`, `contexts`, `env`, `models`, `pages`,
  `route`, and `services` modules. App mounts `HashRouter` and `Switch`; its
  test-panic callback is commented out. Home renders an h1 reading Zipeek;
  the intended landing content is not implemented.
- `route.rs` declares Home, Random, PracticeSetup, three match variants, Browse,
  and a `/404` fallback. MCQ routes are approved by D20 but not declared. Browse
  and setup are placeholders; Practice renders Loader. ErrorPage is props-driven
  and has no route or app-level error-state wiring.
- Trunk configuration and HTML live in `web/`. The configured development port
  is `1420`. Styles live in `web/styles/`; HTML copies `web/public/` assets.
- `zipeek-e2e` is a workspace package with `src/main.rs` and `tests/smoke.rs`.
  Both use managed Chrome and an external Trunk server on port 1420. Only the
  test enables headless mode; the executable suppresses the automation banner.
  Shared readiness, owned Trunk lifecycle, and the package rename remain open.
- Generator package `db-gen` has a modular verified-input preparation pipeline
  and the D21/D23 native-engine/storage guards. It creates the approved empty words table and two HSK indexes; importer/output
  remain unimplemented. `zipeek-core` now supplies Word/WordId, HSK and PoS types. DuckDB-WASM
  integration and the generated database file are absent. A06 will test read-only
  final-dictionary/browser compatibility. A neo 2.0 synthetic fixture is readable
  by WASM 1.32.0 with explicit v1.4.0 storage; default output is rejected.
  Neo is restored at its prior resolved Git revision. D23 requires actual engine
  v2.0.0 and verifies v1.4.0 storage compatibility on the output connection before
  any future dictionary file creation; the earlier v1.4.3 guard is superseded.

## Native generator modules

The generator is a native library plus a thin binary, with no new dependencies:

| File/module | Responsibility |
| --- | --- |
| crates/db-gen/src/main.rs | Three-line entry point delegating to run_cli |
| cli.rs | Logging configuration, human-readable failures, exit status, default manifest path |
| generator.rs | prepare(path): verify sources/notices, open guarded database, create empty schema; return PreparedGeneration |
| database.rs | Native connection, SQL-result handling, typed DbError, engine/storage checks |
| schema.rs and schema/words.sql | Exact approved words DDL and two ordinary HSK indexes |
| policy.rs | Approved native, storage, WASM-package and logical-reproducibility constants |
| inputs/manifest.rs | Typed TOML loading, required metadata policy, names and notice-reference validation |
| inputs/verify.rs | Local-file resolution, sizes, SHA-256, duplicate-file rejection and retained verified bytes |
| inputs/error.rs | InputError with file paths, expected/actual sizes or digests, and underlying causes |
| tests/preparation.rs | Cross-module fixtures and real command validation |

VerifiedInputs owns the exact bytes which passed hashing; future importers must
parse those buffers rather than reopen source paths. This keeps verification
and later parsing tied to the same data. Preparation creates the approved empty
schema in memory. It performs no source JSON/CC-CEDICT parsing, output writing,
or network access.

## Target architecture

### Startup: static loading shell and Yew loading state

The placeholder in `web/index.html` is an **initial loading shell** (also called
a static loading screen or bootstrap loader). It gives the browser useful HTML
to paint while the JavaScript/WebAssembly app downloads and initializes. Resource
prefetching describes fetching resources early; the loading shell describes the
visible placeholder during startup.

Current HTML puts an embedded background image and a `data-loading` animation
with the text "Loading" inside `#root`. It also applies the saved/system theme
before app rendering. `main.rs` mounts Yew into that same `#root`, and
`components/loader.rs` supplies a Yew loader with similar animation markup.
Loader has no loading text; LoadingShell is absent. Static HTML still contains
the background, spinner, and Loading text. App replaces it with route scaffolds
without a database readiness state; listeners log failures but do not render
ErrorPage.

Required handoff:

1. **Before Yew mounts:** the static HTML shell remains visible while the app
   boots. Keep its essential loading appearance available before Yew runs.
2. **Yew's first render:** initialize app/database state to `Loading` immediately,
   before any asynchronous effect starts. Render the Yew loading view as the
   shell is replaced, preserving a continuous loading experience without a blank
   frame or a flash of the ready UI.
3. **Database initialization:** remain `Loading` while initializing DuckDB-WASM,
   fetching `hanzi.duckdb`, and opening the read-only database for queries.
   Yew mounting or the fetch completing alone does not mean the database is ready.
4. **Ready:** show database-backed content only after initialization succeeds
   and the database is queryable. A valid empty database can be Ready; its Word
   of the Day card uses D05's empty placeholder.
5. **Load failure:** leave the loading state and expose retry rather than spinning
   forever. Apply D05's "No word available" plus Retry behaviour to the card;
   retry re-enters `Loading` until the database is ready or fails again.

This defines required states/ownership, not an implemented loader or a selected
Rust state type. The app owns database readiness; individual data queries may
still have their own loading states after startup.

### Runtime data flow

The static web app uses Rust/Yew compiled to WebAssembly, `yew-router` for the
[product routes](PRODUCT.md), and Tailwind for styling. It runs under `/zipeek/`.

Intended data flow:

1. A developer manually runs the native `crates/db-gen` binary against approved,
   licensed sources.
2. The generator produces `web/static/hanzi.duckdb`, which is committed and
   deployed with the static app.
3. DuckDB-WASM opens that file read-only in the browser.
4. Query results drive browse pagination, Word of the Day, and practice rounds.

D16 supersedes D06's initial source plan: use pinned jelleverheyen/hsk-vocabulary
complete.json for the HSK union, both hanzi/pinyin forms, memberships, PoS, rank,
and meanings. CC-CEDICT is consulted only for an empty primary gloss list.
The selected repository declares MIT; retain its notice and upstream data
attribution. The superseded unlicensed leonsilicon inputs are not used.
During db-gen import, D18 lowercases source PoS codes before the fixed 42-code
lookup; any unrecognized code rejects the entire source entry/all forms with a
warning before reading-key grouping and ID assignment. Stored transcriptions
remain untouched. These are required import rules, not implemented generator code.
See [DB](DB.md) for the immutable pin, audited formats, import rules, and notices.
The visible footer attribution remains implementation work; the current App
renders route scaffolds.

The approved D07 storage contract is one physical `words` table containing only
the HSK 2.0/3.0 union from the primary dataset, with optional CC-CEDICT gloss fallback.
No normalized membership/gloss/PoS tables are used in v1. db-gen groups readings by
simplified/numeric pinyin and validates correspondence to D07's display-pinyin
UNIQUE constraint. It preserves source display/numeric pinyin,
pipe-separated glosses, nullable comma-separated PoS, separate nullable HSK columns,
and nullable frequency. Two ordinary HSK indexes supplement implicit constraint
indexes. IDs use row numbers ordered by simplified/numeric pinyin; they reproduce
unchanged input but can shift after regeneration.
Word of the Day and Random use the full HSK-union table, not all of CC-CEDICT.

The empty generator schema is implemented; bindings, application database loading,
static-copy setup and query interfaces remain unimplemented. DuckDB-WASM is approved at the exact npm pin
`@duckdb/duckdb-wasm@1.32.0` (D08); integration is pending. See [DB](DB.md).

The approved e2e harness owns a Trunk child and a thirtyfour-managed Chrome
session per test binary, with shared readiness polling and drop cleanup. It tests
the development server at `/`; the mandatory `/zipeek/` Pages build is validated
separately. The current `zipeek-e2e` package still needs renaming to `e2e` to
support the approved smoke command. See [TESTING](TESTING.md).

Word of the Day reads browser-local calendar fields through `js_sys::Date`, hashes
the UTF-8 `yyyy-mm-dd` string with the approved 64-bit FNV-1a using `u64` arithmetic,
and requests the corresponding full-pool offset using
deterministic database ordering. Empty and failed loads produce the same neutral
card, with Retry on load failure. A `wotd` query override supplies a fixed date
only in debug builds; invalid overrides silently use the real local date. Release
ignores the override, with no testing feature in v1. No persisted hash state or
Clock trait is required. This data flow is approved but not yet implemented.

The target shared layout has a left sidebar with dark/light selection and a
bottom-anchored repository GitHub icon, plus a footer showing Jozef Podlecki's
copyright and required dictionary attribution. Theme state persists in localStorage
through existing gloo facilities; saved selection wins, otherwise default to dark.
Resolve it consistently before first paint and in Yew so loading and ready views
agree. Current index.html instead falls back to system preference, and the existing
footer's GitHub link targets the author's profile; both differ from the new target.
The current App does not mount the shared layout. See [PRODUCT](PRODUCT.md).

Practice state, query state, and component ownership should be documented here when
approved and implemented. Practice setup selects HSK system/level and Endless or Fixed
mode. PoS filtering is supported in URLs; its setup picker is future work.
Practice and Random share one round generator; Random supplies a
whole-DB seed and always uses Endless. Internal candidate reserves never render:
the visible round remains four matched cards per side. See [PRODUCT](PRODUCT.md).

D13 makes unseeded rounds deterministic ordered slices selected by URL offset;
seeded replay instead uses the seed and its HSK context. Canonical URL state carries
mode, Fixed round count when applicable, HSK context, optional canonical PoS set,
exclusions, and either offset or seed. Random emits the seed's PoS. The setup
count choices are 5/10/20 with default 10. Fixed completion
tracking must remain independent of offset wrapping.

Word exclusions are a localStorage-persisted set of WordIds through gloo storage,
serialized into round URLs and applied before generation. Creating a URL unions
saved/current-round exclusions; loading a URL uses only its set, with absence
meaning empty. Persist IDs only on explicit user actions, not on link load.
Marking a new exclusion changes saved/pending state for the next generated URL,
leaving current-round cards, active inputs, and URL unchanged. Setup Reset clears
saved/pending sets and generates a new URL without exclude. This state is distinct
from transient scores and forbidden persisted hash state.
D15 fixes level default 1, rejects malformed/missing/excluded seeds into unseeded
inputs with the same filter/offset, and ignores seeds for Fixed. Validate those
cases before zeroing the offset of a retained seed. Fixed progress is in-memory,
resets on reload, and is independent of the URL's pool position or wrap. Seeded
Next clears its anchor and advances by four as an unseeded Endless round.
[ROUND_GENERATION](ROUND_GENERATION.md) owns
the approved D12 algorithm and resolved D19 single-stream/v1 replay contract.
The pure build pipeline owns its one RNG; recovery helpers borrow that stream,
and generation/shuffling continue from its resulting position. The caller
supplies original URL metadata and ordered data, not an independently recovered
seed and newly seeded RNG. Full-round vectors remain implementation work.

The wasm caller builds three exclusion-clean CandidatePool views with SQL
`ORDER BY id ASC`; a pure native-testable builder performs all deterministic
selection. URL parsing normalizes seeded offsets to zero. A hand-written ASCII
seed string prefixed `zipeek-round-v1` feeds FNV-1a then `rand::rngs::ChaCha12Rng`
from the existing rand 0.10.3 dependency with its approved `chacha` feature (D17).
An existing URL whose seed lacks membership uses deterministic, bounded ChaCha
recovery followed by the first eligible id; never the ambient RNG used for a fresh
Random click. Preserve the original seed URL and retain recovery in effective state.
The builder does not touch database, localStorage, clock, or ambient RNG. Sliding
and fallback affect only internal window starts. The approved rand chacha
feature is enabled; the builder and portable full-round vectors remain
implementation work. Do not add
a separate rand_chacha dependency.

The intended workspace includes `web`, `crates/db-gen`, and `e2e`, with the approved
shared crate `zipeek-core` (D01) for Word, ExerciseVariant, HSK domain types, and
query types used by web and db-gen. Its scaffold and workspace/path wiring
exist. Word/WordId, HSK system/levels and all 42 PartOfSpeech values are implemented;
exercise variants and query types remain future work.

The HSK domain distinguishes system from level/tier and supports independent
memberships for HSK 2.0 and HSK 3.0. A single universal integer level cannot
represent that requirement. `Hsk3Level::SevenToNine` maps the advanced tier to
integer 7 in storage/canonical URLs; 8 and 9 are not separate levels. Remaining
exercise/query types remain to be designed; SQL storage uses D07's separate nullable
`hsk2_level` and `hsk3_level` columns in `words`.

During development, breaking code/data changes and regeneration are acceptable;
backward compatibility and application/database versioning are not required.
This does not authorize changing pinned dependencies without approval.


### Shared domain modules (A02)

zipeek-core has no dependencies or I/O. hsk.rs holds distinct Hsk2Level and
Hsk3Level enums, HskSystem and a tagged HskLevel. Checked integer conversions
reject invalid levels; SevenToNine stores 7. word.rs holds the ten-field Word
representation and WordId, checked against positive SQL INTEGER row-number
bounds. Optional metadata stays optional; empty PoS vectors represent SQL NULL.
Word is a transport record, not import validation: A03 must enforce nonblank
labels, at least one membership, grouping and conflict policies before insertion.

part_of_speech.rs exposes strict canonical-token parsing separately from
case-insensitive source-code mapping. from_source_codes rejects the entire list
on an unknown code, deduplicates recognized values and sorts by canonical token.
A03 must catch the error and log/drop the entire source entry before grouping
or IDs. URL tolerant filtering remains separate A08 work. No serde or database
library was added to the shared crate. Six unit tests include the 42-row mapping
fixture; native schema fixtures test exact columns, constraints and both indexes.
