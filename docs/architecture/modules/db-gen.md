# Native dictionary generator

The implemented native generator is [`crates/zipeek-db-gen`](../../../crates/zipeek-db-gen), not the retired `crates/db-gen` path. Its library API and generator policy are defined in the crate; command parsing is owned by [`zipeek-cli/src/db_gen.rs`](../../../crates/zipeek-cli/src/db_gen.rs). No `zipeek-db-gen` executable is built.

## Source-verified pipeline

1. `inputs/` verifies the pinned HSK and CC-CEDICT bytes and associated notices and hashes.
2. `zipeek-db::native::GeneratorDatabase` owns the native DuckDB connection and session/transaction lifecycle; generator code uses that boundary, requiring engine `v2.0.0` and explicit writable compatibility `v2.0.0`.
3. `schema.sql` creates compatibility `words` plus normalized senses, memberships, characters, identity candidates, controlled categories/PoS and classification relations.
4. [`generator.rs`](../../../crates/zipeek-db-gen/src/generator.rs) parses HSK, applies CC-CEDICT gloss fallback and expands additional CC-CEDICT readings, retaining explicit character-reading data. `import/storage.rs` writes the relational representation.
5. `artifact/writer.rs` stages the native output, copies tables, validates logical snapshots and a reopened read-only database, and publishes only on success. Optional approved classification cache data can be applied before snapshotting.
6. `artifact/report.rs` writes the manifest, provenance, diagnostic counts and per-file hashes. The installed manifest is found under [`zipeek-web/static`](../../../crates/zipeek-web/static/hanzi.manifest.json).

The output is read-only browser dictionary data. It is not a writable personal store or a pedagogical observation ledger. Do not treat an old HSK-only A02/A03 count as the installed dictionary.

## Source and manifest consistency

The installed manifest declares 120,970 `words` rows, engine `v2.0.0`, storage `v2.0.0`, and DuckDB-WASM `1.33.1-dev114.0`. These match `src/policy.rs`, `artifact/writer.rs` and web `package.json`. Older storage-version decisions are historical, not a description of this checked-in generator. Do not change dependency or format pins without explicit approval.

**Artifact verification gate:** `character_readings` is included in the manifest table list and generated SQL, but the source ZIP omits the actual DuckDB binary. Do not claim the table was physically opened or queried until the artifact is present and verified with the native/browser runtime. Source inspection does not certify the user's current Windows build.

## Entry points and further detail

- [Regeneration and installation](../../dictionary-data/schema/regeneration.md)
- [Physical schema](../../dictionary-data/schema/schema.md)
- [Artifact evidence](../../dictionary-data/schema/artifacts.md)
- [Inputs and provenance](../../dictionary-data/schema/sources.md)
- [Current acceptance](../../project-management/current.md)

## Generation performance review (R528 WIP)

The artifact writer currently specifies `BLOCK_SIZE 262144` (256 KiB). Increasing the storage block size is not an established optimization for this workload. The import operates in a transaction with reusable prepared statements but performs individual executes for words, senses and character readings; the artifact writer then bulk-copies the in-memory tables, builds English FTS, checkpoints and verifies the reopened file. Benchmark these phases separately before changing bulk ingestion, schema/index ordering or database layout. Expected gloss pipe normalization diagnostics remain preserved in the structured artifact, but no longer emit WARN per occurrence.

## Batched ingestion (R529 WIP)

The main `words`, `word_senses`, and `character_readings` imports now use parameterized multi-row inserts in chunks of 256, reducing query executions for large dictionaries. Sense display derivation, qualifier and relation extraction, and character enrichment keep their existing per-row logic and transactional boundaries. Small word inserts retain their individual error attribution. The artifact still uses DuckDB's maximum supported 256 KiB block size, and logical snapshot/reopen verification remains mandatory. Benchmark and run the generator's import tests locally before considering the optimization accepted.
