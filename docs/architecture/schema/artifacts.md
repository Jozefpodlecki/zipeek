# Artifacts

Implemented evidence for input preparation, primary import, fallback, artifact
export, and browser integration. Part of the [database documentation](DB.md).
Source rules in [sources.md](sources.md) and [import-rules.md](import-rules.md);
logical reproducibility in [reproducibility.md](reproducibility.md).

This file records what each generator stage *does* and what evidence currently
exists. Historical evidence from each task is preserved; a passing run in one
task is not a current gate claim.

## Implemented input preparation (A26)

`cargo run -p db-gen --locked -- check` performs real offline preparation. The three-line
binary delegates to CLI handling; library modules own manifest loading, file
verification, orchestration and database setup. Set `RUST_LOG=info` to see the
verification summary; an invalid explicit filter fails rather than hiding it.

The manifest's engine, storage, WASM package and reproducibility fields must
match approved policy. Source names must be unique, notice references declared,
and each input/notice path must be a nonempty relative path resolving inside
the manifest directory. Duplicate resolved files, malformed hashes, missing
files, incorrect source sizes and SHA-256 mismatches return typed errors with
paths and expected/actual values. No database connection is prepared until all
files pass. Verified source and notice buffers are retained in memory; future
importers parse those bytes instead of reopening paths after verification.

The current preserved manifest verifies two sources and three notices, totaling
19,567,208 bytes.

## Implemented schema and domain boundary (A02)

`schema/words.sql` is the exact approved D07 ten-column table; `schema.rs` adds
only `idx_words_hsk3` and `idx_words_hsk2`. Preparation creates it after input and
engine/storage checks. No CHECK constraints or schema changes were introduced:
HSK ranges, at least one membership and import-level text policies remain the
responsibility of typed conversions and the upcoming importer. Native fixtures
assert column types/nullability, primary/display-key enforcement, exact source
text, independent memberships and absent metadata. Schema preparation alone writes
no artifact; explicit A05 export does.

The shared types preserve both pinyin strings and nullable traditional, numeric
pinyin, English and frequency fields. `PartOfSpeech` accepts exactly the 42 source
codes above, case-insensitively, with strict canonical strings for DB/URL use.
Source lists are canonical-token sorted and deduplicated; empty maps to NULL
when inserted. `WordId` permits 1 through `i32::MAX`. A03 assigns IDs after
accepted-entry grouping in the documented simplified/numeric-pinyin order.

## Implemented primary HSK import (A03)

`cargo run -p db-gen --locked -- check` verifies all inputs/notices, checks native v2.0.0 and storage
compatibility v1.4.0, creates the schema and imports the primary source into
memory. `import_primary(prepared)` parses only retained verified hsk-primary bytes.
`prepare()` alone still returns an empty schema for fixtures and later pipelines.
Primary import alone writes no file. A05 exports it explicitly. A04 adds CC-CEDICT
parsing only for readings with empty primary glosses.

The importer rejects unknown-code entries and all their forms before grouping,
logs every invalid level or dropped form, validates SQL INTEGER rank/ID bounds,
and preserves exact headword/transcription text. Missing required labels or
all memberships reject a reading/entry. Repeated readings retain first-form
traditional/display text and first nonempty primary glosses. Recognized PoS is
unioned and canonical-token sorted; independent HSK memberships keep the lowest
within-system value with warnings. Duplicate-entry rank conflicts retain the
first available rank and warn. Stored glosses preserve order/case after trimming
and escaping literal ASCII pipes to U+FF5C. Display-key collisions fail with both
original JSON entry slices, including their original whitespace and escaping.

A `BTreeMap` orders exact simplified/numeric keys before assigning IDs 1..N;
SQL `row_number` ordering is checked against these IDs in the real-snapshot test.
One prepared positional `INSERT` is reused inside a transaction; failures roll
back earlier inserts and preserve pre-existing rows. Warnings are also returned
as ordered `ImportDiagnostic` values for logical reproducibility checks.

Measured on the preserved snapshot: 11,494 entries, 64 rejected unknown-PoS
entries, 12,100 grouped readings, and 514 diagnostics (64 rejections plus 450
repeated-reading warnings). Every resulting reading has primary English; no
fallback is needed for this snapshot. An independent Python audit matched the
accepted-key and missing-gloss counts. Fixtures still exercise missing English,
pipe replacement, invalid labels/ranks, display drift/conflicts and level
conflicts.

## Implemented CC-CEDICT fallback (A04)

`import_primary` applies fallback before SQL insertion, using only retained verified
`cc-cedict-fallback` bytes. All source and notice hashes are still verified even
when no fallback is needed. A malformed fallback source is a clear UTF-8 or
line-numbered parsing error when consulted. Blank/comment lines and LF/CRLF are
supported; parsed entries preserve input order within each normalized reading.
The complete pinned snapshot parses as 125,320 dictionary entries.

Primary English is retained whenever present. Empty-primary readings match exact
simplified hanzi plus comparison-normalized numeric pinyin, or display pinyin if
numeric is absent. No spelling-only match selects a different reading. Entries
with `variant of` glosses are skipped; the first remaining entry with nonempty
meanings wins. Gloss trimming, source order, casing and pipe escaping follow
D18 for both sources. Unresolved words retain NULL English and remain in the
pool. Diagnostics record applied/ignored/unresolved outcomes and transformations.
No source text, headword, pinyin, membership, rank or ID is changed by fallback.

The shared normalizer covers the approved neutral-tone examples, tone marks,
ASCII case, `u:`/umlaut/`v`, common decomposed marks and concatenated display
syllables. It changes comparison keys only. Both real pinned sources separate
syllables with whitespace; synthetic tests also cover concatenated numeric and
marked display forms. A07 reuses this helper for pinyin labels; it does not
implement the round builder or alter the v1 seed encoding.

The synthetic verified pipeline fills five readings, leaves two unresolved,
skips two variants, preserves primary glosses and inserts NULLs correctly.
The actual primary snapshot is unchanged: 12,100 readings and 514 primary
diagnostics, with no fallback needed.

## Generated dictionary artifact (A05)

`web/static/` contains `hanzi.duckdb`, `hanzi.manifest.json`,
`hanzi.diagnostics.json`, `INPUTS.toml`, `ATTRIBUTION.md` and `notices/`. The
data licence remains CC BY-SA 4.0; source MIT/upstream notices are copied without
modification. The input manifest records source URLs/commits, acquisition dates,
exact bytes, licences and changes. The report records actual native engine
v2.0.0, storage compatibility v1.4.0, WASM target 1.32.0, generator version/
manifest hash, input checksums, counts, complete schema/index definitions and all
distributed file checksums. It excludes its own checksum to avoid self-reference.
A19's ignored transitive lockfile is still a policy gap; direct pins alone are not
a complete frozen-toolchain claim.

The exporter checks every ordered JSON row (all ten fields), table/column metadata,
constraints and both indexes against the in-memory import before and after
checkpoint/detach/read-only reopening. Failures leave no published bundle. It
never creates the file before engine/storage checks pass. Native read-only
attachment rejects DELETE; original rows remain unchanged. Existing outputs,
malformed/missing inputs, and quoted/absolute Windows paths have fixture coverage.
Empty dictionaries retain the complete schema and valid artifact checksums.

Two independent full-source exports matched 12,100 rows, 514 diagnostics, IDs,
schema/indexes and logical/diagnostic checksums. Their database hashes differed,
satisfying D22 (see [reproducibility.md](reproducibility.md)):

- Working-tree artifact SHA-256: `491fae6f84a96b5825c860163701f3e138cf72e5b50763848fde0e96ba03c546`
- Independent repeat SHA-256: `227e0ff5ecc4925c2d42e9686cc94aa2dedf0e9d42e4556dadddc6af6a9f1fb7`
- Logical SHA-256: `bf2865b52fd3e4343979cf2952cb42c01314042df489129fd22287d8d4c38f25`
- Diagnostics SHA-256: `d893b30b15121068c88047c8ce1d4c0ab0550093b37659555d4de80391251dfc`

Both file checksums were independently checked against closed-file bytes; they
identify their own outputs and are not required to match. Each generated file
was natively compared against all logical content from its independent import.
Build output copies match source bytes, and development HTTP serves the exact
database hash. These tests cover file generation/native access/static serving,
not the DuckDB-WASM browser query layer.

## A06 final dictionary/browser integration - 2026-10-27

The user explicitly approved A06 and the exact npm dependency, prioritizing this
integration over the A15 harness refactor. The existing external-server Chrome
setup was used. The npm lockfile retains previous packages and adds the pinned
runtime with its Arrow dependency. No Cargo dependency, native engine, source
snapshot, schema, generated artifact or replay contract changed.

The actual `web/static/hanzi.duckdb` (12,100 rows, ten columns, 3,158,016 bytes)
now passes both Node and Chrome with package 1.32.0 / engine v1.4.3. Every ordered
row was read, IDs and bound ID lookup verified, and DELETE rejected as read-only.
The native writer remains neo v2.0.0 with v1.4.0 storage. This resolves the earlier
open final-artifact compatibility question for this exact snapshot/runtime. The
database SHA-256 remains
`491fae6f84a96b5825c860163701f3e138cf72e5b50763848fde0e96ba03c546`.
Future regenerated artifacts must repeat compatibility checks.

The browser fetches the complete file into a registered buffer and opens it with
`DuckDBAccessMode.READ_ONLY`. There is no OPFS persistence or automatic generation.
Worker/WASM/Arrow assets are served locally; Chrome also passed the actual Pages
build under `/zipeek/` without cross-origin isolation headers. Loading remains
visible until a typed query succeeds. Blocked-download failure and Retry are
tested independently of bootstrap; route pages still await feature UI wiring.

The npm tarball omits DuckDB's MIT license file, so packaging includes the exact
[v1.32.0 upstream license](https://raw.githubusercontent.com/duckdb/duckdb-wasm/v1.32.0/LICENSE)
from `web/scripts/duckdb-LICENSE.txt`. Arrow's packaged Apache license and NOTICE
are copied alongside assets. No source notices or dictionary attribution changed.

The 2026-10-27 review proposes extracting the A06 browser adapter into a dedicated
crate while preserving native generator ownership and the current schema/runtime
contract. See [target-architecture.md](../modules/target-architecture.md#proposed-browser-database-crate-extraction-a33)
and [../../process/decisions/process-and-infra.md#d30-extract-browser-database-adapter-proposed](../../process/decisions/process-and-infra.md#d30-extract-browser-database-adapter-proposed).
This is a proposal, not an implemented crate or approval of dependency moves.

## Testing evidence

Test counts and logs for each stage are recorded per task in
[../../testing/provenance.md](../../testing/provenance.md) (A02–A05) and
[../../testing/browser-db.md](../../testing/browser-db.md) (A06). This file
records the *operational contract*; the test evidence records what was verified.

## Related

- Input preparation rules: [sources.md](sources.md), [import-rules.md](import-rules.md).
- PoS mapping: [pos-mapping.md](pos-mapping.md).
- Reproducibility rule: [reproducibility.md](reproducibility.md).
- Regeneration commands: [regeneration.md](regeneration.md).
- Browser module: [../modules/browser-db.md](../modules/browser-db.md).


## A44 regenerated bundle and A46 workflow

On 2026-10-29 the explicit cargo make dev workflow regenerated the pinned
snapshot and installed a complete verified bundle. Current database SHA-256 is
728dca2400ba8f23877043c6216a433e9c0adfad539097f3081cfb200605eb02 (3,158,016 bytes).
All 12,100 rows, schema/indexes, input records, logical hash and 514 diagnostics
match the prior A05/A06 snapshot. Physical bytes may differ under D22. Original
bundle bytes are preserved together in the workflow backup. The A05/A06 hashes
above remain historical evidence, not the current file checksum.

Native export/read-only verification, complete-file checksums, exact development
HTTP bytes and one managed Chrome dictionary/MCQ load pass. Four focused Node
checks cover complete installation/backup, checksum rejection, path/missing-notice
rejection and rollback. No full application or Pages regression was rerun under
D36. See regeneration.md for cargo-make commands and testing/evidence.md for logs.
