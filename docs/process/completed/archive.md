# Completed — archive

Frozen historical record migrated once from the former single-file
`docs/COMPLETED.md`. Do not append to this file. New outcomes go to
`YYYY-MM-DD.md` for their outcome date.

This file preserves:
- The one-time STATUS.md reconciliation performed during A18.
- The "Preserved documentation history" list of earlier documentation-only
  outcomes whose validation belonged to their original snapshots.
- Corrections and supersession notes that were historically appended between
  task rows on 2026-10-26.

Links and wording are preserved verbatim. Do not rewrite them even when the
target file has since moved.

---

## One-time STATUS reconciliation — A18

All nine legacy TODOs are represented by TASKS: scaffold/loading in A20/A21,
configuration in A19, workflows in A16, source contracts in A18/A02/A03,
generation/querying in A01/A04/A05/A06, e2e in A15, product implementation in
A07/A08/A09/A11/A12/A13, sidebar/theme/attribution in A14, and RNG/replay in
A18/A07. No unique open task was discarded. Legacy Working contained no active
implementation; CURRENT now records the approved work instead.

The 15 historical Done entries follow verbatim. Their validation claims belong
to their original snapshots; superseded proposals and retired blockers are history.
The original log did not record branches for these entries.

## Preserved documentation history

- 2026-10-26: Revamped AGENTS.md as the contributor process guide; separated
  product, architecture, DB, testing, and open decisions into linked documents.
  Recorded plan-and-approval workflow, checks by change type, current repository
  gaps, and unapproved defaults without resolving product or DB choices.
  Validation passed: structure/encoding/whitespace checks for eight documents,
  27 relative links, nine retained open decisions, command/configuration checks,
  and `git diff --check`. Rust builds and e2e were not required for this docs-only task.

- 2026-10-26: Recorded the development/regeneration policy and approved
  `zipeek-core`, independent HSK 2.0/3.0 memberships, D02 generation with exactly
  four visible cards per side, Endless/Fixed practice modes, and the shared
  seed-based Random flow. Kept physical schema, retry limits, advanced-tier
  widening, Fixed count options, and query encoding open. Documentation only;
  no application, schema, or dependency implementation changes.
  Validation passed: eight Markdown documents, 26 local links, unique D01-D13
  decision records, requirement consistency checks, and `git diff --check`.
  Rust builds and e2e were not required for this documentation-only update.

- 2026-10-26: Approved D08's exact `@duckdb/duckdb-wasm@1.32.0` pin and
  recorded it in AGENTS section 6; prohibit silent tag/dev-build substitutions
  and require evidence for compatibility blockers. Approved D09's managed
  ChromeDriver and harness-owned Trunk lifecycle, shared 30s readiness polling
  with 100ms-to-2s backoff, root-served e2e, and target `cargo test -p e2e` invocation.
  Documented the pending `zipeek-e2e` package rename and separate Pages build check.
  Validation passed: eight Markdown files, 28 relative links, decision/policy
  consistency, and `git diff --check`. No dependency installation, harness code,
  runtime compatibility test, or Rust build was performed for this docs-only task.

- 2026-10-26: Recorded approved D05: browser-local calendar date, deterministic
  full-pool count/ordered-offset selection, neutral empty card, load-failure retry,
  and debug/testing-only `wotd` date override. Added DB, architecture, and test
  contracts; kept the proposed FNV-1a choice and its exact parameters open in D14,
  alongside DefaultHasher's no-persistence/cross-Rust-version limitations.
  Validation passed: eight Markdown documents, 29 relative links, unique D01-D14
  decision records, D05 requirement consistency, and `git diff --check`.
  Documentation only; no Rust/browser tests or application changes were required.

- 2026-10-26: Approved D13/D14: exact 64-bit FNV-1a with debug-only validated
  date overrides; Fixed counts 5/10/20 (default 10); canonical practice/seed URLs,
  persistent WordId exclusions, deterministic offset progression, and looping.
  D13 supersedes random unseeded selection in D02. Added optional canonical PoS
  sets (seed emits its PoS), HSK 3.0 advanced value 7 / SevenToNine, and exclusion
  lifecycle: generation unions saved/current sets, loading trusts only the URL.
  Recorded specified PoS labels without inventing an exhaustive source mapping.
  Kept generator details in D12 and remaining URL/session gaps in D15. This is
  documentation of approved requirements; no application or schema code changed.
  Validation passed: eight Markdown documents, 30 relative links, unique D01-D15
  decision records, requirement/precedence consistency, and `git diff --check`.
  No Rust/browser tests were required for this documentation-only update.

- 2026-10-26: Recorded D06's CC-CEDICT + HSK source selection, CC BY-SA 4.0
  attribution/derived-data policy, raw pinyin/both-hanzi requirements, sparse PoS
  and absent frequency, and the exclusions of Wiktionary/noncommercial Anki data.
  Added README attribution and documented pending visible-footer work. Audited
  both complete HSK repository trees and package manifests: no licence evidence
  found; redistribution remains blocked. No upstream issue filed or official-PDF
  extraction performed in this documentation task. CC-CEDICT's homepage and
  download page both currently declare 4.0; recorded the reported earlier 3.0
  discrepancy without treating it as a current verified fact.
  Validation passed: nine Markdown documents, 33 relative links, D06/audit
  consistency, and `git diff --check`. No data was imported/committed; application
  code was unchanged, so Rust builds and browser smoke were not required.

- 2026-10-26: Recorded approved D07's single `words` table, HSK-union-only pool,
  display/numeric pinyin, nullable metadata, separate HSK columns, surrogate PK,
  and unique simplified/display-pinyin key. Verified official DuckDB regression
  tests reject partial indexes; the user selected two ordinary HSK indexes instead.
  Clarified deterministic row-number IDs can shift across changed inputs and
  frequency remains NULL without a verified rank. Added D16 for unresolved key
  merges, multiple within-system levels, and unmatched words needing pinyin.
  Source redistribution remains blocked by D06. Documentation only; no database
  was generated, dependencies installed, or schema/runtime implementation changed.
  Mapped Word of the Day's generic hanzi ordering to `ORDER BY simplified, pinyin`.
  Validation passed: nine Markdown files, 33 relative links, documented ten-column
  DDL/constraints/two ordinary indexes, decision consistency, and `git diff --check`.
  DDL was reviewed as documentation; no DuckDB engine or browser runtime was run.

- 2026-10-26: Documented the initial static loading shell in index.html and the
  required handoff to Yew's first-render Loading state. Loading continues through
  DuckDB-WASM initialization, database fetch, and read-only open/query readiness;
  failures leave loading for retry, and valid empty data uses D05's placeholder.
  Added startup behaviour and planned delayed-bootstrap/database/failure coverage.
  Documentation only; existing HTML, components, and database code were unchanged.
  Validation passed: nine Markdown documents, 34 relative links, startup-state
  consistency checks, and `git diff --check`. Browser tests were not run for this
  documentation-only task; the actual loading flow remains implementation work.

- 2026-10-26: Recorded approved D12 in ROUND_GENERATION.md: pure CandidatePool
  builder, three SQL-ordered/exclusion-clean views, portable ChaCha12Rng requirement,
  canonical zipeek-round-v1 seed input, bounded first-clean-window sliding,
  seeded offset-zero/no-slide rules, actual right-label normalization, clamped
  same-system widening, membership retries/preference, and typed distinct-pool
  failure. Corrected earlier left-column duplicate examples and unconditional
  no-repeat claims. MAX_SLIDE starts at 16 as a tunable per-view guard; tuning
  can change fallback rounds. Added native/wasm replay and encoding coverage,
  including the computed canonical FNV-1a documentation vector.
  Required rand_chacha is absent; addition/exact version need approval. Recorded
  rand 0.10's rng/sample API names and D17 dependency/draw-order/version transport
  gaps. No dependencies, application code, or database artifacts changed.
  Validation passed: ten Markdown files, 47 relative links, unique D01-D17 records,
  D12 requirement checks, canonical FNV-1a vector calculation, and `git diff --check`.
  The documented native/wasm round tests are planned, not executed by this docs task.

- 2026-10-26: Documented the shared left sidebar, persistent dark/light theme
  selection with dark as the preferred default, bottom-anchored GitHub icon linking
  to the zipeek repository, and footer copyright for Jozef Podlecki alongside data
  attribution. Recorded current system-theme/profile-link differences and planned
  shell/theme smoke coverage. Documentation only; implementation remains pending.
  Validation passed: ten Markdown files, 48 relative links, shell requirement
  consistency, and `git diff --check`. No application code changed or UI tests ran.

- 2026-10-26: Recorded approved D16 primary import from jelleverheyen/hsk-vocabulary
  complete.json, pinned a66fd30b9580da2c2af7eb19e4b9d8099a29c061; audited MIT notice,
  11,494 entries / 12,655 forms, exact reading-key counts and tag/pinyin differences.
  Superseded the leonsilicon source plan while preserving its historical audit.
  Recorded first-form traditional handling with warnings, level mapping/minimum
  conflicts, exact pinyin/hanzi preservation, empty-only gloss fallback, first-gloss
  labels and variant eligibility, and numeric-key row-number IDs valid only within
  a DB snapshot. Added D18 for remaining PoS/comparison/serialization details.
  Documentation only; no dataset file, generator, schema code, or dependency changed.
  Validation passed: ten Markdown documents, 48 relative links, table/fence checks,
  unique D01-D18 records, source/import consistency, and `git diff --check`.
  Source inspection was read-only; no DuckDB generation or application tests ran.

- 2026-10-26: Recorded approved D15: level-1 fallback; malformed/nonexistent/URL-
  excluded seed to unseeded with the same filter/offset; Fixed ignores seed;
  validate before zeroing retained-seed offset. Fresh Random selection may use
  ambient randomness, but membership-less seed recovery on URL loads uses up to
  ten deterministic ChaCha draws then first eligible id, preserving the original
  seed URL. New exclusions affect only the next generated URL; Setup Reset clears
  storage/pending state and omits exclude. Fixed progress is in-memory and resets
  on reload while offset is honored; wrapping preserves progress. Seeded Next
  clears the seed and advances by four for all v1 match variants. Added planned
  fallback/recovery/session tests; no application or dependency implementation changed.
  Validation passed: ten Markdown documents, 48 relative links, table/fence checks,
  unique D01-D18 records, approved D15 and fallback/session consistency, and
  `git diff --check`. No browser/runtime tests were run for this docs-only update.

- 2026-10-26: Recorded approved D17: use existing rand 0.10.3 with chacha enabled
  and rand::rngs::ChaCha12Rng / rand::SeedableRng; no rand_chacha dependency.
  Superseded the earlier direct-crate requirement throughout active guide/specs
  while preserving history. Recorded current sample/rng API names and moved
  outstanding draw-order/replay-version/tuning details to D19. Feature selection
  is approved and must not be re-requested; the manifest edit remains pending
  implementation. No Cargo manifest or application code changed in this docs task.
  Validation passed: ten Markdown documents, 47 relative links, table/fence checks,
  unique D01-D19 records, D17 provider/configuration consistency, and `git diff --check`.
  No builds or runtime tests were required for this documentation-only update.

- 2026-10-26: Recorded D18's exact 42-code PoS mapping and case-insensitive lookup
  (including Mg/Rg aliases), whole-entry/all-form rejection with warnings for
  unknown codes, and required native fixture tests. Replaced the earlier
  illustrative measure_word/particle vocabulary with classifier/modal_particle.
  Audited the pinned dataset: unknown cc/mq/qt/qv affect 64 entries and 105 forms;
  lowercasing recognizes six entries with case aliases. Remaining D18 comparison/
  serialization/missing-label-seed details stay open. Documentation only; db-gen
  does not yet exist and no generator/unit-test implementation was claimed.
  Validation passed: ten Markdown documents, 47 relative links, all 42 mapping
  pairs, source-policy/audit consistency, unique D01-D19 records, and
  `git diff --check`. Native fixture tests remain implementation work.

- 2026-10-26: Bootstrapped TASKS/CURRENT/COMPLETED/UX as requested before any
  implementation. TASKS contains only open todo work in prerequisite order;
  CURRENT has empty Active/Blockers and points to ready A00; COMPLETED starts
  with an empty log. UX transcribes settled behaviour and marks missing decisions
  explicitly, including unconfirmed MCQ scope. Recorded the user's D02-D18
  resolution milestone while retaining residual-note reconciliation as A18.
  The actual guide has a repository map in section 3 and gates/policies in 6/4/5,
  not the requested Current state table or sections 8/9. Legacy history/TODOs and
  the guide are preserved. No application code, dependency, or database changed.
  Validation passed: fourteen Markdown files, 64 relative links, 23 unique todo
  tasks in prerequisite order, ready markers A00/A18/A22, matching CURRENT next
  task, empty Active/Blockers and completion log, thirteen UX sections/TBDs, and
  `git diff --check`. No Rust/browser gates were run; the proposed A00 audit is
  still awaiting approval, and no implementation task has started.

---

## Historical correction notes

The following correction and supersession notes were originally interleaved with
the 2026-10-26 task rows. They are preserved here as a frozen record.

### A18 accepted review reconciliation — 2026-10-26

These entries append corrections to the earlier snapshot claims; all prior
outcomes and the migrated history above remain unchanged.

- **A01 correction:** scaffold verified on disk and in workspace. Corrects the
  earlier dependency summary: manifest requirements are serde 1.0.171 and
  serde_json 1.0.99, with additional never-approved dependencies and an unpinned
  DuckDB Git source. Approved package rename zipeek-db-gen → db-gen applied;
  main remains a stub. Dependency/source audit is A24; no generation claimed.
- **A17 correction:** empty zipeek-core library and workspace/path wiring verified.
  Scaffold task is closed; shared domain/query implementation remains A02.
- **A15 correction:** corrects the earlier partial row: e2e is already a workspace
  member with src/main.rs and tests/smoke.rs. Only the integration test enables
  headless Chrome; the executable has it commented out and suppresses the
  automation banner. Package rename, owned Trunk lifecycle/readiness, and reliable
  smoke execution remain open.
- **A21 partial scaffold inventory:** route switch is mounted; setup, three match
  variants, Browse, and NotFound exist. ErrorPage is props-driven with no
  route/state wiring. Loader exists; LoadingShell is absent and the static HTML
  loader remains. Practice renders Loader. No database readiness, MCQ routes, or
  complete UI behavior is claimed.
- **A18 accepted review/amended scope:** full D18 approval and DB cross-reference
  recorded; D20 three-variant MCQ routes/rules, three options, N=2, Endless/Fixed,
  and offset +1 documented; UX-MCQ01 retains UI details only and A10 is no longer
  provisional. Corrected factual guide/README/architecture/testing/CI/task-state
  drift. Added Home h1 Zipeek and renamed generator package to db-gen. Earlier
  AGENTS-update claims were not all on disk at review; factual/process pointers
  now reconciled without changing D12 replay policy. BASELINE and prior COMPLETED
  history preserved. D19 remains A23; generator dependencies/source remain A24;
  fmt/Clippy repairs remain A20.

A18 validation: all five required commands were executed. Formatting failed on
the recorded scaffold/page files; Clippy failed on unused Timeout and
components imports in app.rs. Wasm check and Pages build passed with those two
warnings (Tailwind 4.3.3). Workspace tests failed in the single Chrome smoke:
`tab crashed`, status 500, Chrome 156.0.8078.47, after 121.16s; five unit targets
contained zero tests. The Home assertion was not established in-browser. A local
Trunk start attempt found port 1420 occupied; its existing server returned HTTP
200 and was not stopped. No gate failure was repaired outside approved scope.
Markdown/link/task-state and append-only history checks are recorded in CURRENT.
The scope is implemented with known validation failures; this is not an all-gates-pass claim.

Final A18 checks passed: 16 Markdown documents, 86 local links/anchors, balanced
fences, 21 unique open tasks in prerequisite order, 13 retained UX TBDs, unchanged
BASELINE/prior COMPLETED prefix, preserved user practice.rs change, unchanged
dependencies/workflows/replay contract, and `git diff --check`.

### A23 / D19 v1 correction — 2026-10-26

D19 resolved as Option 1: keep v1 as the only replay format. The earlier A18
note forecasting a pending v2 prefix was speculative, not a commitment; no v2
prefix, version-selection URL parameter, or compatibility parser exists or is
added. ROUND_GENERATION now fixes one ChaCha12Rng seeded once from original
canonical inputs, up to ten amount-1 slice sample recovery requests with early
success, zero-RNG first-eligible fallback, continuation into correct selection,
four matching reserves or two MCQ distractors, then the specified final shuffles.
Caller/helper ownership agrees with authoritative AGENTS §5; §2 requires explicit
approval, prefix plus URL selector, and documented compatibility for a future
break. DECISIONS and affected architecture/product/DB/testing references
reconciled; CURRENT replay blocker removed. Full native/wasm golden rounds remain
A07/A10 implementation work, not passing tests. No application, dependency,
schema, URL parser, or historical-log edits.

Validation passed: 16 Markdown documents, 88 local links/anchors, balanced code
fences, unchanged v1 canonical hash `0xa0e531d76df62844`, task prerequisite order,
unchanged tracked non-Markdown files, preserved prior COMPLETED prefix, and
`git diff --check`. The sample/shuffle APIs were checked against installed rand
0.10.3 source. Rust/Trunk/Chrome gates were not rerun for this documentation-only
task; earlier A18 failures remain recorded. Next open task: A24 generator audit;
no follow-up implementation was started.

### A26 server-cleanup correction

Correction to A26's server-cleanup assertion recorded under A02: Ctrl-C was sent,
but process termination was not confirmed; do not infer owned lifecycle cleanup
from that run. A15 still owns the harness lifecycle fix.

---

## End of archive

This file is complete and closed. Subsequent outcomes belong in the date file for
their outcome date.