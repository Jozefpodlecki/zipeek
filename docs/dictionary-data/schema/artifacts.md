# Dictionary generation and published artifacts

This is the **current** generator pipeline, as implemented in [`crates/zipeek-db-gen`](../../../crates/zipeek-db-gen) and described by the installed [`hanzi.manifest.json`](../../../crates/zipeek-web/static/hanzi.manifest.json). Task-by-task historical snapshot sizes, temporary checksums and former pre-expansion counts belong in [release history](../../archive-history/RELEASE_HISTORY.md), not in current generator specifications.

## Inputs and preparation

`inputs/` validates the pinned HSK and CC-CEDICT source files, declared license notices, paths, lengths and SHA-256 hashes. [`generator.rs`](../../../crates/zipeek-db-gen/src/generator.rs) calls `prepare()` to verify sources and open the engine-guarded in-memory schema. Native `duckdb-neo` must report `v2.0.0`; the writer explicitly uses storage compatibility `v2.0.0`.

`import_primary()` parses HSK, applies CC-CEDICT gloss fallback where required, expands valid CC-CEDICT-only readings and preserves explicit character readings. This is no longer an HSK-only, fallback-on-missing-glosses-only pipeline. See [import rules](import-rules.md) and the actual [`import/`](../../../crates/zipeek-db-gen/src/import) modules.

## Schema, enrichment and output

[`schema.sql`](../../../crates/zipeek-db-gen/src/schema.sql) defines the `words` compatibility table, normalized senses/characters/identities, classification tables, memberships, constraints, indexes and controlled taxonomy. The generator may apply validated AI classification cache data if supplied; a table existing does not mean every sense is classified.

[`artifact/writer.rs`](../../../crates/zipeek-db-gen/src/artifact/writer.rs) writes a staging directory, attaches an explicitly compatible native DuckDB file, bulk-copies generated tables, compares logical snapshots, checkpoints/detaches, reopens read-only and validates again before publishing. Failed stages do not publish a partial output. [`artifact/report.rs`](../../../crates/zipeek-db-gen/src/artifact/report.rs) records manifest, diagnostics, source/artifact provenance and table metadata.

## Current installed bundle

| Property | Value from manifest |
| --- | --- |
| Dictionary `words` rows | **120,970** |
| Character occurrences | **312,649** |
| Identity candidates | **120,970** |
| Diagnostic events | **10,187** |
| Native engine | `v2.0.0` |
| Storage version | `v2.0.0` |
| Browser package | `@duckdb/duckdb-wasm@1.33.1-dev114.0` |
| Data artifact | `hanzi-20261115-215819.duckdb` (plus compressed `.gz`) |

These are manifest **declarations** read from the supplied source, not an independent native replay or validation of every row. The source ZIP does not bundle the generated database bytes; binary hash verification and Chrome loading remain outstanding. To inspect the actual asset filenames and hashes, read the installed manifest directly. Do not use the original 12,100-row, 514-diagnostic HSK-only export as the current count.

## Browser integration and boundaries

The [`zipeek-db` WASM adapter](../../../crates/zipeek-db/src/wasm) verifies and opens the installed read-only dictionary; the app supplies a local worker and asset bundle. The manifest's required generator/storage fields are distinct from the live browser engine version and Debug's runtime storage metrics. A new generated dictionary requires native output verification plus browser/Pages tests. It is **not** a writable user database.

See [regeneration](regeneration.md), [runtime](runtime.md), [reproducibility](reproducibility.md) and [provenance testing](../../testing-quality/e2e/provenance.md).

## English full-text indexing

Generation explicitly sets `BLOCK_SIZE 262144` on the new `artifact` database.
After copying dictionary data and validating its logical snapshot, the native
generator installs and loads DuckDB `fts` and creates an immutable BM25 index
on `words.id` / `words.english`. This requires the DuckDB 2.0 `fts` extension
to be available to the native generator. A missing extension fails generation
rather than publishing an artifact without the declared index. The generator
verifies the index schema and executes a BM25 query during generation and
read-only reopening. FTS-owned auxiliary tables are excluded from the canonical
logical dictionary snapshot, which still compares all dictionary relations.

The read-only `Database::english_full_text_search` provides a standalone ranked
query. Browse also exposes `English relevance (FTS / BM25)` as an explicit
field selection. It uses the persisted `fts_main_words` index, sorts by BM25
score, and retains HSK, part-of-speech, category, Word Set and pagination
filters. Ordinary English substring search and Hanzi/Pinyin search remain
separate modes.

On browser dictionary initialization, the DuckDB-WASM connection attempts
`INSTALL AND LOAD fts;`. The extension is fetched dynamically by the versioned
WASM runtime. If it cannot be loaded, the dictionary remains available but
ranked queries will report an error instead of quietly changing to substring
search. The browser compatibility and offline behavior require actual Chrome
verification with a regenerated DuckDB 2.0 bundle at both `/` and `/zipeek/`.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/dictionary-data-schema-artifacts.md). Dated implementation claims in that snapshot are not current build or browser verification.
