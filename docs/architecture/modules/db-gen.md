# db-gen module

Native generator: verified-input preparation, engine/storage guards, schema
creation, HSK import with CC-CEDICT fallback, and artifact export. Part of the
[architecture overview](../ARCHITECTURE.md). Approved dependency policy in
[D21](../../process/decisions/database-and-artifacts.md#d21-generator-dependencies-sources-and-engine-guard)
and [D23](../../process/decisions/database-and-artifacts.md#d23-native-neo-engine-and-compatible-storage);
import rules in [D16](../../process/decisions/database-and-artifacts.md#d16-primary-import-and-ids)
and [D18](../../process/decisions/database-and-artifacts.md#d18-source-comparison-and-serialization).

## Module map

The generator is a native library plus a thin binary, with no new dependencies
beyond D21 and the explicitly approved D37 Clap CLI addition.

| File/module | Responsibility |
| --- | --- |
| `crates/db-gen/src/main.rs` | Three-line entry point delegating to `run_cli` |
| `cli.rs` | Clap help/check/generate commands, logging, failures, exit status, default manifest and absolute output paths |
| `generator.rs` | `prepare(path)`: verify sources/notices, open guarded database, create empty schema; return `PreparedGeneration` |
| `database.rs` | Native connection, SQL-result handling, typed `DbError`, engine/storage checks |
| `schema.rs` and `schema/words.sql` | Exact approved words DDL and two ordinary HSK indexes |
| `policy.rs` | Approved native, storage, WASM-package and logical-reproducibility constants |
| `inputs/manifest.rs` | Typed TOML loading, required metadata policy, names and notice-reference validation |
| `inputs/verify.rs` | Local-file resolution, sizes, SHA-256, duplicate-file rejection and retained verified bytes |
| `inputs/error.rs` | `InputError` with file paths, expected/actual sizes or digests, and underlying causes |
| `tests/preparation.rs` | Cross-module fixtures and real command validation |
| `import/source.rs` | Typed primary JSON and exact original entry slices for conflict diagnostics |
| `import/validation.rs` and `levels.rs` | Required labels, PoS, rank bounds, independent memberships and conflict warnings |
| `import/glosses.rs` | Ordered trimmed meanings, pipe replacement and transformation diagnostics |
| `import/reading.rs` and `grouping.rs` | First-form retention, deterministic merging, display-key checks and sorted integer IDs |
| `import/diagnostics.rs` and `error.rs` | Structured reproducible warnings and actionable fatal errors |
| `import/storage.rs` | Prepared positional parameters, one transaction, rollback on insertion failure |
| `import/cedict.rs` and `fallback.rs` | Verified CC-CEDICT index, empty-primary matching, variants and outcome diagnostics |
| `zipeek-core/src/pinyin.rs` and `pinyin/syllables.rs` | Shared pure comparison-only pinyin keys |
| `artifact/writer.rs` | Staged directory export, guarded explicit storage, checkpoint/detach, read-only reopen and publication |
| `artifact/snapshot.rs` | Complete ordered rows, table/column/constraint/index definitions for logical equality |
| `artifact/report.rs` and `files.rs` | File hashes, manifest/notices, diagnostics, integrity report and failed-stage cleanup |
| `artifact/error.rs` | Actionable export, SQL, path, publication and logical-mismatch errors |
| `tests/artifact.rs` | Independent exports, checksums, refusal/failure cleanup, read-only enforcement and empty schema |
| `tests/hsk_import.rs` | Synthetic edge cases and actual verified snapshot import |

`VerifiedInputs` owns the exact bytes which passed hashing; future importers must
parse those buffers rather than reopen source paths. This keeps verification
and later parsing tied to the same data. Preparation creates the approved empty
schema in memory. `import_primary(prepared)` parses the retained hsk-primary bytes
and, only for readings with no primary English, applies fallback from the retained
CC-CEDICT bytes before inserting the rows in one transaction. The CLI runs both
steps and reports counts. With the `generate` command, artifact modules export the
result through the same guarded connection. No network access occurs.

## Verified-input preparation (A26)

`cargo run -p db-gen --locked -- check` performs real offline preparation. The three-line binary
delegates to CLI handling; library modules own manifest loading, file verification,
orchestration and database setup. Set `RUST_LOG=info` to see the verification
summary; an invalid explicit filter fails rather than hiding it.

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

## Engine and storage guards (D23)

Generator startup:

1. Query and log `SELECT version()`.
2. Fail clearly on anything other than native `v2.0.0`; on query failure, exit
   non-zero.
3. Set `storage_compatibility_version = v1.4.0`.
4. Query and log the setting; fail on mismatch.
5. Return the configured in-memory connection for later output work.

No output is created before these checks pass. Default neo output (storage
version 999) is rejected by the pinned WASM runtime; only explicit v1.4.0
storage is readable. This is a generator invocation check, not a Cargo build
script or an automatic application-build generation step.

## Schema creation (A02)

`schema/words.sql` is the exact approved D07 ten-column table; `schema.rs` adds
only `idx_words_hsk3` and `idx_words_hsk2`. Preparation creates it after input and
engine/storage checks. No CHECK constraints or schema changes were introduced:
HSK ranges, at least one membership and import-level text policies remain the
responsibility of typed conversions and the importer. Native fixtures assert
column types/nullability, primary/display-key enforcement, exact source text,
independent memberships and absent metadata. Schema preparation alone writes no
artifact; explicit A05 export does.

## Primary HSK import (A03)

`import_primary(prepared)` parses only retained verified hsk-primary bytes.
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
accepted-key and missing-gloss counts.

## CC-CEDICT fallback (A04)

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

The synthetic verified pipeline fills five readings, leaves two unresolved,
skips two variants, preserves primary glosses and inserts NULLs correctly. The
actual primary snapshot is unchanged: 12,100 readings and 514 primary
diagnostics, with no fallback needed.

## Artifact export (A05)

The exporter checks every ordered JSON row (all ten fields), table/column metadata,
constraints and both indexes against the in-memory import before and after
checkpoint/detach/read-only reopening. Failures leave no published bundle. It
never creates the file before engine/storage checks pass. Native read-only
attachment rejects DELETE; original rows remain unchanged. Existing outputs,
malformed/missing inputs, and quoted/absolute Windows paths have fixture coverage.
Empty dictionaries retain the complete schema and valid artifact checksums.

Two independent full-source exports matched 12,100 rows, 514 diagnostics, IDs,
schema/indexes and logical/diagnostic checksums. Their database hashes differed,
satisfying D22 (logical, not physical, reproducibility):

- Working-tree artifact SHA-256: `491fae6f84a96b5825c860163701f3e138cf72e5b50763848fde0e96ba03c546`
- Independent repeat SHA-256: `227e0ff5ecc4925c2d42e9686cc94aa2dedf0e9d42e4556dadddc6af6a9f1fb7`
- Logical SHA-256: `bf2865b52fd3e4343979cf2952cb42c01314042df489129fd22287d8d4c38f25`
- Diagnostics SHA-256: `d893b30b15121068c88047c8ce1d4c0ab0550093b37659555d4de80391251dfc`

Each generated file was natively compared against all logical content from its
independent import. Build output copies match source bytes, and development HTTP
serves the exact database hash. These tests cover file generation/native access/
static serving; browser query coverage is in
[testing/browser-db.md](../../testing/browser-db.md).

## Regeneration

The explicit cargo make dev workflow generates a fresh candidate, validates and
installs the complete bundle with backup/rollback, then starts Trunk. Separate
db-check/db-generate/db-install/serve tasks are available. No generation is added
to Trunk pre-build hooks; the task helper uses Node built-ins, not new dependencies.

Commands, manual review/replacement procedure, and sizing live in
[schema/regeneration.md](../schema/regeneration.md). Generation is explicit and
manual; never an automatic application build step.


## User-added local LLM scaffold

Observed during A53 on 2026-10-29: the user added aimux-core and aimux-providers
requirements 0.5.0 to crates/db-gen/Cargo.toml and exported a new llm module.
Its client wraps the LM Studio provider, normalizes a /v1 base URL and sketches
model listing/text generation with timeout/retry and streaming/error helpers.
This is scaffolding, not an implemented classifier or a verified working
generator integration. Types/helpers referenced by this scaffold are not yet
fully supplied locally; no compile/build/model call was run for this doc task.
The generator CLI/import/export do not invoke it.

The intended next feature is explicit local-model category enrichment and
entry-tag filtering, tracked by A56 and [future requirements](../../product/FUTURE.md#local-ai-category-enrichment).
No tag schema/output or new runtime AI endpoint exists. Preserve the user-added
manifest/lock/source changes; review the complete dependency/configuration and
classification/storage design before implementation. Root Cargo.lock resolves
aimux-core, aimux-providers, aimux-provider-utils and aimux-stream to 0.5.0.
The direct manifest requirements are semver ranges, not exact equals pins;
this observation does not add new AGENTS dependency-policy targets.
