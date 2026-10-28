# Decisions and backlog

Approved decisions below record the user's requirements on 2026-10-26 and
2026-10-27 and 2026-10-28; they do not mean the feature is implemented. Proposed details in
the open backlog remain unapproved. Ask when an open choice blocks an approved
task. Record resolutions with their date and update the owning document while
retaining decision history.

This file is the index. Each decision is elaborated in one of the group files:

| Group file | Decisions |
| --- | --- |
| [foundations.md](foundations.md) | D01–D05, D10, D11 (product and shared domain) |
| [rounds-and-replay.md](rounds-and-replay.md) | D12–D15, D17, D19 (round construction, URLs, RNG) |
| [database-and-artifacts.md](database-and-artifacts.md) | D06–D08, D16, D18, D20–D23 (sources, schema, generator, runtime) |
| [web-and-ui.md](web-and-ui.md) | D24–D26, D29, D31–D32, D34 (shell, motion, styling, pronunciation, social cards) |
| [process-and-infra.md](process-and-infra.md) | D09, D27, D28, D30, D33 (e2e, lockfile, crate extraction, tooling acceptance) |

## Approved decisions

| ID | Decision | Approved requirement | Owner document | Group |
| --- | --- | --- | --- | --- |
| D01 | Shared crate | Use `zipeek-core` for shared domain/query types. The old `zipseek-core` reference is a typo; implementation and dependency changes remain separate work. | ARCHITECTURE | [foundations](foundations.md#d01-shared-crate) |
| D02 | Round generation | Originally random correct four from HSK + PoS; D13 supersedes unseeded selection with deterministic ordered slices. Same-pool reserve candidates, eight-unique-candidate target, HSK +/-1 then drop PoS, bounded resampling, insufficient-pool message, four visible cards per side still apply. | PRODUCT | [foundations](foundations.md#d02-round-generation) |
| D03 | Practice progression | Offer Endless and Fixed on setup. Endless has Next round, no summary or persisted scores. Fixed has N rounds and a summary of correct pairs, mistakes, and time, plus Play again / Back to setup. | PRODUCT | [foundations](foundations.md#d03-practice-progression) |
| D04 | Random tile | Shared generator, random variant and whole-DB seed, seed-anchored HSK/PoS, ignore remembered filters, always Endless. D12 refines membership retries/preference and replaces the original unfiltered escape hatch with typed pool failure. | PRODUCT / ROUND_GENERATION | [foundations](foundations.md#d04-random-tile) |
| D05 | Word of the Day | Full-pool pick using browser-local `yyyy-mm-dd`, date hash modulo count, deterministic SQL ordering, neutral empty state, load-failure retry, and debug-only `wotd` override. D14 fixes FNV-1a and forbids DefaultHasher here. | PRODUCT / DB / TESTING | [foundations](foundations.md#d05-word-of-the-day) |
| D06 | Sources and licensing | Original CC-CEDICT/leonsilicon import selection is superseded by D16. Retain CC-CEDICT attribution/derived-data terms, preserve source notices, and exclude Wiktionary/noncommercial Anki. Unlicensed leonsilicon inputs are no longer used; their audit remains historical. | DB / PRODUCT / README | [database-and-artifacts](database-and-artifacts.md) |
| D07 | Physical schema and pool | One denormalized `words` table containing only the HSK 2.0/3.0 union, supplied by D16's primary source with empty-gloss fallback. Integer PK, unique simplified/display-pinyin pair, both hanzi/pinyin forms, pipe-separated English, nullable comma-separated PoS, separate nullable HSK columns, nullable frequency rank. Two ordinary HSK indexes replace unsupported partial indexes. See DB for exact DDL and D16 import rules. | DB / ARCHITECTURE / TESTING | [database-and-artifacts](database-and-artifacts.md) |
| D08 | DuckDB-WASM version | Exact npm pin `@duckdb/duckdb-wasm@1.32.0`. No `latest`, `next`, or dev-build substitution. Stop and report evidence for genuine wasm toolchain compatibility blockers. | AGENTS / DB | [database-and-artifacts](database-and-artifacts.md) |
| D09 | E2e lifecycle | Harness owns a Trunk child and one managed Chrome WebDriver session per test binary. Shared readiness helper: 30s timeout, 100ms exponential backoff capped at 2s, HTTP-response or TCP-connect readiness, no fixed sleep. Serve/test at `/`, retain `/zipeek/` for Pages builds. Target command: `cargo test -p e2e -- --test-threads=1`; A15 implements the package rename, owned lifecycle and root/Pages coverage. | TESTING | [process-and-infra](process-and-infra.md#d09-e2e-lifecycle) |
| D10 | Development lifecycle | Breaking changes are acceptable during development; regenerate data/artifacts. No app/database versioning or backward-compatibility requirement. Existing dependency controls still apply. | AGENTS / DB | [foundations](foundations.md#d10-development-lifecycle) |
| D11 | HSK abstraction | Distinct HSK 2.0 (1-6) and HSK 3.0 (1-6 plus advanced tier 7-9) memberships. A word may be in both at different levels, or only one. | PRODUCT / DB / ARCHITECTURE | [foundations](foundations.md#d11-hsk-abstraction) |
| D12 | Deterministic round builder | Pure CandidatePool function; three exclusion-clean SQL-ordered nested views; canonical zipeek-round-v1 string and FNV-1a to named ChaCha12Rng; no StdRng. Deterministic sliding/sampling, seeded offset zero, bounded rejection, clamped widening, membership retries/preference, right-label normalization and typed failure. D17 selects rand's provider; D19 fixes the v1 stream and draw order. | ROUND_GENERATION / DB / TESTING | [rounds-and-replay](rounds-and-replay.md#d12-deterministic-round-builder) |
| D13 | Counts, URLs, exclusions, offsets | Fixed 5/10/20, default 10, no custom count. Canonical HSK/level and optional PoS set, exclusions, offset or Endless seed in URLs. Generate exclusions from saved/current/pending union; load exactly the URL set (absent means empty), never merge saved state. D15 defers new exclusions until the next URL. Deterministic replay and four-step nominal offsets. Advanced tier uses 7 (7-9 alias). | PRODUCT / DB / TESTING | [rounds-and-replay](rounds-and-replay.md#d13-counts-urls-exclusions-offsets) |
| D14 | Reproducible hashing and date override | 64-bit FNV-1a over UTF-8 date bytes: basis 0xcbf29ce484222325, prime 0x100000001b3, XOR then wrapping multiply, u64 arithmetic. Zero dependencies. No DefaultHasher for WotD/reproducible values, no persisted hash state. Invalid debug override silently uses real local date; release ignores it; no testing feature in v1. | AGENTS / PRODUCT / TESTING | [rounds-and-replay](rounds-and-replay.md#d14-reproducible-hashing-and-date-override) |
| D15 | URL defaults and session lifecycle | Level defaults to 1 in selected system; malformed/missing/excluded seed becomes unseeded with same filter/offset; Fixed ignores seed. Membership-less URL seeds use deterministic ten-draw ChaCha recovery then first eligible id, retaining original URL. New exclusions are pending until the next URL; setup Reset clears storage and omits exclude. Fixed counter is memory-only, reload resets progress but honors offset. Seeded Next clears seed and advances by four. | PRODUCT / ROUND_GENERATION / TESTING | [rounds-and-replay](rounds-and-replay.md#d15-url-defaults-and-session-lifecycle) |
| D16 | Primary import and IDs | Pin jelleverheyen/hsk-vocabulary complete.json at a66fd30b9580da2c2af7eb19e4b9d8099a29c061; CC-CEDICT only for empty glosses. Exact source text, numeric reading key, first traditional form plus duplicate warning, old/new levels (new-7/new-7+ -> 7), minimum conflicting level with warning, first-gloss exercise labels, drop missing pinyin, NULL missing English. IDs are row numbers ordered by simplified/numeric pinyin, snapshot-local. D18 fully specifies PoS mapping, comparison, serialization, conflicts, and seed eligibility. | DB / TESTING / PRODUCT | [database-and-artifacts](database-and-artifacts.md) |
| D17 | RNG provider | Use existing rand 0.10.3 with the chacha feature, explicitly approved. Import rand::rngs::ChaCha12Rng and rand::SeedableRng; do not add rand_chacha. Use current rand 0.10 API names (sample/rng). This supersedes D12's direct-crate proposal; D19 fixes the algorithm/replay call order and sole v1 format. | AGENTS / ROUND_GENERATION / TESTING | [rounds-and-replay](rounds-and-replay.md#d17-rng-provider) |
| D18 | Source comparison and serialization: fully approved | Approved 2026-10-26: exact 42-code case-insensitive PoS mapping and whole-entry unknown-code rejection; comparison-only pinyin normalization; literal ASCII gloss pipes replaced with U+FF5C and logged; numeric/display-key conflict policy; deterministic recovery of seeds missing required English. Preserve stored source transcriptions. See [DB §D18](../../architecture/schema/import-rules.md) for the full contract. Generator/fixture implementation remains open. | DB / TESTING / PRODUCT | [database-and-artifacts](database-and-artifacts.md) |
| D19 | V1 replay stream and draw order | Approved 2026-10-26: keep v1 as the only format, with no version URL parameter/parser today. One ChaCha12Rng is seeded once from original canonical URL inputs; recovery makes up to ten amount-1 slice sample calls, stops at first eligible word, and uses first-eligible-by-id fallback without RNG. Generation continues on that stream: correct content, four matching reserves or two MCQ distractors, then final shuffles in the exact ROUND_GENERATION order. Lock full native/wasm golden outputs during implementation. Future replay breaks require a v2 prefix, a selecting URL parameter, documented compatibility, and explicit approval. | ROUND_GENERATION / TESTING / AGENTS | [rounds-and-replay](rounds-and-replay.md#d19-v1-replay-stream-and-draw-order) |
| D20 | MCQ practice | Approved 2026-10-26: hanzi-to-meaning, meaning-to-hanzi, and meaning-to-pinyin at `/practice/mcq/<variant>`; Endless and Fixed with 5/10/20 questions, default 10. Three options: one correct answer plus two distractors (N=2), using D02/D12 pool widening and visible-answer uniqueness rules. Offset advances by one per question. UX-MCQ01 records immediate feedback, locked selection, number-key hints and adaptive answer layout for later A10 implementation. | PRODUCT / UX / TESTING | [database-and-artifacts](database-and-artifacts.md) |
| D21 | Generator dependencies, sources, and engine guard | Approved 2026-10-26: replace moving duckdb-neo Git with duckdb =1.4.3 and bundled, apply the exact DB.md dependency pins/removals and SHA-256/TOML support, and preserve checksum-identified primary/CC-CEDICT inputs and notices. At generator invocation query SELECT version() or PRAGMA version against the bundled engine, log it, and fail clearly unless it reports v1.4.3. This is generation-time validation, not an automatic Cargo build/generation hook. Native-to-WASM compatibility remains an open A06 test, not an A25 blocker. | DB / TESTING | [database-and-artifacts](database-and-artifacts.md) |
| D22 | Logical database reproducibility | Approved 2026-10-26: identical verified inputs and pinned tooling must reproduce ordered logical rows, IDs, schema/index definitions, and import/drop diagnostics. Record SHA-256 for each final artifact as an integrity check; byte-identical .duckdb files across runs are not required. Do not patch or mask physical database bytes to force equality. | DB / TESTING / CODING_STANDARDS | [database-and-artifacts](database-and-artifacts.md) |
| D23 | Native neo engine and compatible storage | Approved 2026-10-26 after the measured probe: keep restored neo at prior Git revision, require actual native v2.0.0, and require storage compatibility v1.4.0 before output creation. Query/log engine and storage settings and fail on mismatch; future output files must use explicit compatible storage. Supersedes the earlier v1.4.3 native guard. WASM stays exactly 1.32.0; default neo output is rejected. Synthetic read-only WASM engine coverage passed; final dictionary/browser integration remains A06. | DB / TESTING / AGENTS | [database-and-artifacts](database-and-artifacts.md) |
| D24 | Web shell and landing direction | Approved by the user on 2026-10-27: neutral slate dark/light with blue accents; desktop left rail and compact mobile header; shared Home/Practice/Browse navigation, saved theme control, repository link and attribution. Landing uses a prominent daily-word area followed by Practice/Browse/Random tiles. Build navigation/shell first; dictionary-backed content remains A06/A12 and no daily word is fabricated. A27 implements this direction; A22 records daily-card fields; unrelated UX decisions remain open. | UX / PRODUCT / ARCHITECTURE | [web-and-ui](web-and-ui.md#d24-web-shell-and-landing-direction) |
| D25 | Compact collapsible shell | Approved by direct user request on 2026-10-27: remove decorative headers such as Chinese vocabulary and A fresh starting point; move theme to the top-right of a permanent top panel; make the drawer collapsible; remove unnecessary vertical scrolling. Supersedes D24's theme-in-sidebar placement. Preserve functional headings, access to drawer/theme controls and natural scrolling for content that genuinely exceeds the viewport. A28 implements this amendment without data/dependency changes. | UX / PRODUCT / ARCHITECTURE | [web-and-ui](web-and-ui.md#d25-compact-collapsible-shell) |
| D26 | Persistent landscape and loading presentation | Approved by direct user request on 2026-10-27: reuse the existing unmodified landscape as the shared loading/app backdrop, compose light/dark surfaces with it, and replace the clipped loading treatment. A30 keeps the image outside Yew's mount point, adds theme-specific overlays/translucent surfaces and a centered shared loading panel. This is presentation work; A06/A21 still own database initialization/readiness/retry. | UX / PRODUCT / ARCHITECTURE | [web-and-ui](web-and-ui.md#d26-persistent-landscape-and-loading-presentation) |
| D27 | Workspace lockfile | Approved 2026-10-27: track the existing root Cargo.lock and remove its ignore entry, preserving the current resolved graph. A19 removes the obsolete web-local lockfile and enables locked Trunk builds. Direct version requirements are unchanged; dependency updates still require explicit approval. No exact Trunk version has been selected; the tested local installation is 0.22.0. | AGENTS / TESTING / CI | [process-and-infra](process-and-infra.md#d27-workspace-lockfile) |
| D28 | Modular e2e infrastructure | Requested 2026-10-27, approved and implemented by A15 on 2026-10-28: owned harness, named suites, page helpers and explicit browser fixtures; D09 single session per binary, root/Pages separation and failure cleanup verified. | TESTING | [process-and-infra](process-and-infra.md#d28-modular-e2e-infrastructure) |
| D29 | Scoped UI state and motion (proposed) | Requested by the user on 2026-10-27: scoped read/command contexts and reducers; persistent database/theme providers; motion preference provider, small Animato adapter and interruptible route host. Preserve current Animato 1.7.3/path/physics/color; choose motion profiles before enabling them. A31 implements theme/database state boundaries; A37 selects 150 ms fade-only route/drawer motion; A32 implementation remains separate. | UI_ARCHITECTURE / UX | [web-and-ui](web-and-ui.md#d29-scoped-ui-state-and-motion-proposed) |
| D30 | Extract browser database adapter (proposed) | Requested by the user on 2026-10-27: extract transport, typed rows and private SQL to proposed zipeek-db-wasm; retain Yew providers and npm/asset deployment in web. Approve name, native-target strategy and exact dependency moves first. A33 owns implementation. | ARCHITECTURE | [process-and-infra](process-and-infra.md#d30-extract-browser-database-adapter-proposed) |
| D31 | Tailwind-first component styling | Required by the user on 2026-10-27: correct the drift from Tailwind to bespoke component CSS. Tailwind 4.3.3 utilities remain the default for component styling; preserve shared theme tokens and focused bootstrap/keyframe exceptions. A35 owns migration with visual/behaviour parity. A35 implements the correction on 2026-10-28 with root/Pages visual and behaviour coverage; no dependency upgrades. | CODING_STANDARDS / UI_ARCHITECTURE | [web-and-ui](web-and-ui.md#d31-tailwind-first-component-styling) |
| D32 | Web Speech API pronunciation | Daily/match/MCQ hanzi or pinyin surfaces offer Listen using native speech synthesis with zh-CN; supplied fallback speaks the represented word's simplified hanzi. Voice support varies; API-spy tests only. Requirement recorded, not implemented. | UX / PRODUCT / web-shell | [web-and-ui](web-and-ui.md#d32-web-speech-api-for-pronunciation) |
| D33 | Accept build-tool Picomatch advisories | Accept CVE-2026-33671 and CVE-2026-33672 while patterns remain developer-controlled build input; no override or audit fix. Reassess if the boundary changes. No package changes. | CI | [process-and-infra](process-and-infra.md#d33-accept-build-tool-picomatch-advisories) |
| D34 | Static social cards and preview image | Approved site-wide 1200×630 landscape/slate/wordmark/blue preview and exact share copy for Open Graph and Twitter/X large-image cards. No word/score-specific previews; A40 implements before v1 with /zipeek/ URLs. | UX / PRODUCT | [web-and-ui](web-and-ui.md#d34-static-social-cards-and-preview-image) |

D27 (approved 2026-10-27): track the existing root Cargo.lock and remove its
ignore entry, preserving the current resolved graph. A19 removes the obsolete
web-local lockfile and enables locked Trunk builds. Direct version requirements
are unchanged; dependency updates still require explicit approval. No exact Trunk
version has been selected; the tested local installation is 0.22.0.

D31 (required by the user on 2026-10-27): correct the drift from Tailwind to
bespoke component CSS. Tailwind 4.3.3 utilities remain the default for component
styling; preserve shared theme tokens and focused bootstrap/keyframe exceptions.
A35 owns migration with visual/behaviour parity. This direction does not approve
dependency upgrades or a redesign; implementation has not started.

D24/D25 and D13 have dated A22 amendments on 2026-10-27: label-only navigation,
footer repository placement and deferred mid-round exclusion control. The daily
card and answered UX requirements are recorded; pending summary/traditional/
Browse details and motion profiles remain open. A22 adds no dependency pin.

## Open decisions

The user requested the following architecture work on 2026-10-27. The need for
modular e2e infrastructure and scoped UI state is explicit; the detailed plans
below are proposals, not approval to add crates or change dependency features.

D28 has implemented A15 scope. D29 has approved A31 theme/database state scope;
its motion work and D30 extraction remain proposals requiring their recorded
scope/feature/dependency review. The approved order is A15 → A35 → A31
→ A21; motion and extraction retain their separate requirements.

D21/D22/D23 are approved. No native-engine/storage decision remains open.
D23 supersedes the earlier D21 bundled-client/native-version choice; other D21
helper pins and source preservation remain approved. Implementation and the
future regenerated-artifact compatibility checks are still tracked; A06 has
validated the current full dictionary in actual browser workers.

No replay-format decision remains open: D19 selected v1. A07 implements the pure
builder and full-round native/wasm golden fixtures for all six variants.
UX detail and generator dependency/source work remain tracked in TASKS/CURRENT.

The original guide referred to a schema draft in DB.md, but that file did not
exist when this backlog was created. D11 clarified the HSK domain, and D07 later
selected the single-table SQL schema. D16 selected/pinned the primary import and
resolved its rules; D18's full comparison/serialization contract is now approved.
D19 was subsequently resolved as Option 1 (keep v1). The earlier v2 forecast
was speculative, not a commitment; preserve its history with an appended correction.
