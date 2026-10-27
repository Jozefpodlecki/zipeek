# Provenance and import checks

Data source hashing, schema, import, and PoS fixture coverage. Part of the
[testing index](TESTING.md). Approved sources in
[D06](../process/decisions/database-and-artifacts.md#d06-sources-and-licensing)
and [D16](../process/decisions/database-and-artifacts.md#d16-primary-import-and-ids);
schema in [D07](../process/decisions/database-and-artifacts.md#d07-physical-schema-and-pool);
PoS mapping in [D18](../process/decisions/database-and-artifacts.md#d18-source-comparison-and-serialization).

## Data provenance and generation checks (D06/D16)

Use `complete.json` from exact commit
`a66fd30b9580da2c2af7eb19e4b9d8099a29c061`. Retain its MIT notice, source
attribution, retrieval/checksum information, and transformation record. Verify
notices for the separately pinned CC-CEDICT fallback. The superseded leonsilicon
packages must not be imported as substitutes.

## Schema and import validation (D07/D16)

Validate one `words` table, its ten columns, nullable metadata, PK/display-key
`UNIQUE` constraint, and two ordinary HSK indexes. Do not add non-HSK dictionary
entries. Report actual source-entry, expanded-form, grouped-reading, and dropped
row counts; the audit's 11,494 entries / 12,655 forms are not a generated DB size.

- Group by simplified plus exact numeric pinyin, retain the first traditional
  form, and warn on repeated keys. Test different readings remain separate and
  both directions of numeric/display-key correspondence hold for D07 uniqueness.
- Map `old-1..6` and `new-1..6` independently; both `new-7` and `new-7+` map to 7.
  Deduplicate identical tags; conflicting within-system levels choose the minimum
  and warn; unknown tokens are logged/skipped rather than inventing memberships.
- Preserve both source transcriptions exactly, including spaces, case, ü and
  unnumbered neutral syllables. Do not rederive display pinyin. Missing/blank
  required pinyin is dropped and logged.
- Trim nonempty meanings, preserve casing/order, and serialize with pipes.
  A nonempty primary list must never invoke CC-CEDICT. Empty primary meanings
  invoke reading-disambiguated fallback; ignore `variant of` entries. A failed
  fallback writes NULL English and warns without dropping a usable pinyin row.
- Verify first-gloss labels for both English-bearing variants and variant-specific
  eligibility: missing English is allowed for `pinyin-hanzi` only. Rendering and
  normalized duplicate keys must use the same chosen gloss.
- Map raw PoS codes using D18's case-insensitive canonical map and whole-entry
  unknown-code rejection; preserve supplied frequency ranks and NULL missing ranks.
  Verify complete-token PoS filtering.
- Assign IDs with row numbers ordered by simplified/numeric pinyin after grouping.
  Repeat identical input to reproduce IDs; changed snapshots may shift them.
  Assert replay/exclusions only for the same DB snapshot.
- Exercise schema creation and file loading with the approved native generator
  and pinned DuckDB-WASM runtime when implemented. D18 fully approves comparison-key,
  literal-pipe handling, key-conflict and missing-label seed rules in
  [DB.md](../architecture/schema/DB.md). D19 fixes the recovery/build stream;
  its golden vectors remain implementation work.

## PoS mapping fixture tests (D18)

Add native unit tests against fixture source entries when implementing the
generator; do not report these tests as already passing without a corresponding
implementation.

- Cover every one of [DB.md](../architecture/schema/DB.md)'s 42 source-code/canonical-value
  pairs with fixture rows.
- Lookup is case-insensitive: `Mg`/`mg` produce `numeric_morpheme`, `Rg`/`rg`
  produce `pronoun_morpheme`, and ordinary uppercase forms such as `V` produce
  `verb`. Stored pinyin and hanzi stay byte-identical; case folding applies to
  source PoS lookup.
- Include recognized multi-PoS entries and repeated case variants; produce only
  recognized canonical lowercase strings without duplicate canonical tokens.
- An unknown code alone rejects the entire word/all forms and emits a warning.
  Mixing a recognized code with an unknown code must still reject the whole word,
  not merely drop the bad tag or retain the word with NULL PoS.
- Include pinned-source unknowns `cc`/`mq`/`qt`/`qv`. They remain unknown, with
  no guessed expansions. The source audit predicts 64 rejected entries/105
  associated forms before other import checks; this is not a generated-row
  count or a test result.
- Missing/empty PoS is NULL, distinct from a present unknown code. Verify skips
  occur before merging/ID assignment and are reflected in reproducible output counts.
- Verify `q` maps to `classifier` and `y` to `modal_particle` throughout
  DB/filter/seed tokens; the old illustrative `measure_word`/`particle` names
  are not canonical.
- Keep URL parsing separate: unknown `pos` query tokens follow D13's ignore
  policy, rather than triggering source-entry rejection.

## Generator foundation acceptance (D21/D22/D23)

At every generator invocation:

- Query `SELECT version()` on the native engine, log the raw result, and fail
  unless it is `v2.0.0` before any import/output (D23).
- Set and query-verify `storage_compatibility_version=v1.4.0` on the output
  connection before creating any output.
- Test wrong releases, development suffixes, query failure and the actual
  configured storage; metadata alignment is not a runtime pass.
- SHA-256 mismatch/missing-file rejection before parsing any input; preserve the
  primary commit and fallback header/notices. Check byte-exact approved snapshots
  after Git checkout; the audited fallback has CRLF, so attributes must prevent
  line-ending conversion. Reject a changed moving upstream download.

D22 logical reproducibility: compare every column in `ORDER BY id` order, schema/
index definitions, and deterministic import/drop diagnostics across independent
runs of verified inputs and pinned tooling. Record each final file checksum
separately for integrity; do not require identical file bytes or equal file
hashes across runs.

Do not hand-edit DuckDB bytes or mask physical differences to make a checksum
comparison pass.

## A26 preparation evidence (implemented)

`cargo test -p db-gen` runs 19 tests: five database unit tests and fourteen
integration/CLI tests. Database tests cover actual native/storage versions,
rejected versions/storage, failing SQL context, and NULL/empty scalar results.
Preparation tests use copied local fixtures to exercise source/notice tampering,
missing files, sizes, malformed TOML/digests, policy overrides, parent paths,
duplicate files and undeclared notices, plus successful preparation. A
retained-buffer test proves a changed source file cannot replace the bytes already
verified. CLI tests verify the actual pinned sources and reject invalid logging
configuration. No network or generated DuckDB artifact is needed for these tests;
the database connection is in memory.

Fixtures live under `crates/db-gen/tests/fixtures/inputs`; native pure/database
unit tests stay beside their implementation. `main.rs` contains only CLI
entry-point delegation. No new test dependency or lint suppression is added.

## A02 schema and shared-domain evidence (implemented)

`cargo test -p zipeek-core -p db-gen --offline` passes 29 tests: the prior 19
preparation/database tests, six shared-domain tests, and four native schema
integration tests. The shared fixture covers all 42 code/token pairs and case
aliases; unknown source codes reject the entire list. HSK tests reject separate
8/9 levels and check the combined tier. `WordId` tests cover SQL integer bounds.
The SQL fixture verifies exact columns/types/nullability, both HSK indexes,
primary/display uniqueness, distinct readings, Unicode and NULL metadata. A
preparation test confirms the real pipeline creates an empty `words` table.

## A03 primary import evidence (implemented)

`cargo test -p db-gen -p zipeek-core --offline` now passes 39 tests: the prior 29
plus ten importer integration tests in `tests/hsk_import.rs`. JSON fixtures under
`tests/fixtures/hsk` cover rejected entries/all forms, invalid labels and ranks,
level conflicts, first-form retention, canonical PoS, exact numeric keys,
first-nonempty gloss selection and pipe escaping. Fatal display collisions
retain both original source rows verbatim. Native insertion tests cover quotes,
Unicode, NULLs, distinct readings and rollback after a partially inserted batch.
The verified real snapshot is parsed twice to compare every Word and diagnostic;
counts are 11,494 entries, 64 rejections, 12,100 rows and 514 diagnostics. SQL
checks verify row count, exact ID ordering and at least one HSK membership.
The actual CLI test asserts import counts as well as source/engine verification.

## A04 fallback evidence (implemented)

Workspace tests pass 49 domain/generator tests plus Chrome smoke. Three new
shared normalizer tests cover tone/umlaut aliases, whitespace, marked display
syllables and the six approved neutral-tone examples, including retained `r5`/`ng5`.
Seven tests in `tests/cedict_fallback.rs` cover primary-gloss retention, variant
skipping, source-order selection, matching by reading, pipe escaping, unchanged
row metadata, deterministic diagnostics, unresolved NULLs and malformed UTF-8 or
line syntax. A verified manifest fixture proves fallback precedes SQL insertion.
The actual verified CC-CEDICT source parses all 125,320 entries, and the real
primary import remains logically identical. Synthetic checksum fixtures carry
path-specific `-text` attributes so Git cannot invalidate their pinned hashes.
Chrome, wasm and Pages gates also pass.

## A05 artifact evidence (implemented)

Workspace tests pass 55 domain/generator tests plus Chrome landing smoke. Six
artifact integration tests compare independent complete row/schema/index
snapshots and diagnostics, verify every file checksum and report roundtrip,
retain empty schema, reject existing output/unknown CLI arguments, clean failed
staging, handle quoted absolute Windows paths, reject missing read-only files
without creating them, and prove read-only DELETE fails without modifying rows.
Fixtures exercise empty-primary fallback before database export. Tests write only
temporary bundles; normal test/build commands do not regenerate the
checked-in-path dictionary.

Manual full-source generation produced `web/static/` and an independent ignored
`target/a05-repeat/` bundle. Both outputs were compared against all imported rows
and schema before and after read-only reopen. Logical/diagnostic hashes match;
physical hashes differ, satisfying D22. The Pages dist copy and HTTP download
match the working-tree bytes. Input/notice hashes, complete metadata and checksum
records also pass independent Python checks.

## Current status

D06/D07/D16/D18/D21/D22/D23 are approved and implemented at the generator level
(A02–A05) with the evidence above passing. Runtime browser compatibility is
covered by [browser-db.md](browser-db.md). The PoS fixture checklist is required
coverage; A02's shared-domain fixture already exercises all 42 pairs, and A03's
importer tests exercise whole-entry rejection. Any new source mutation or
schema change requires regenerating and re-running this coverage; past passing
logs are evidence of that run, not a current gate claim.