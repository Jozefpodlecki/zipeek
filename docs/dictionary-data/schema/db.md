# Dictionary database: current implementation

This index describes the **checked-in source and installed manifest**, not an original HSK-only proposal. The physical SQL definition in [`schema.sql`](../../../crates/zipeek-db-gen/src/schema.sql), the generator in [`generator.rs`](../../../crates/zipeek-db-gen/src/generator.rs), and [`hanzi.manifest.json`](../../../crates/zipeek-web/static/hanzi.manifest.json) are the evidence for the installed layout. The database is generated natively and opened read-only by the browser; it is not a writable learning-history store.

## Installed artifact

| Manifest fact | Installed value |
| --- | --- |
| Dictionary `words` rows | **120,970** |
| Identity candidates | 120,970 |
| Source entries | 11,494 |
| Diagnostic events | 10,187 |
| Native generator engine | `v2.0.0` |
| Storage compatibility | `v2.0.0` |
| DuckDB-WASM npm package | `@duckdb/duckdb-wasm@1.33.1-dev114.0` |
| Read-only database file | `hanzi-20261115-215819.duckdb` |
| Compressed browser asset | `hanzi-20261115-215819.duckdb.gz` |

The manifest includes checksums, source metadata, table names and generated-time properties. It declares 312,649 character occurrences and 11,846 explicit character readings. The 12,188 distinct HSK source reading keys described in the source audit are not the row count of this expanded dictionary. The original **12,100-row, 10-column HSK-only snapshot** is historical, not the installed database. The `words` compatibility projection remains ten columns, but normalized tables, senses, membership, classifications and identity candidates are part of the generated schema. The checked-in manifest lists the `character_readings` table, although its physical contents remain unverified until the database file is opened. See [schema](schema.md).


The generator copies the maintained [artifact attribution notice](artifact-attribution.md) into the published dictionary as `ATTRIBUTION.md`; the notice source is documentation, not Rust code.

## Generated dictionary search index (R706 pending regeneration)

The generator source now defines `word_search_documents` and `word_search_keys`, both keyed by the unchanged `words.id`. Search documents combine English headword glosses with lexical sense definitions. Search keys preserve Hanzi and precompute compact tone-folded pinyin and numeric pinyin. The DuckDB FTS index targets `word_search_documents.english_text` rather than just `words.english`.

**The installed manifest values above still describe the old generated artifact.** The new schema and FTS index must be generated through `cargo run -p zipeek-cli --locked -- db-gen generate --output target/db-workflow/search-r706`, verified after read-only reopen, tested on both native and WASM, and installed through the existing verified installer. No change to the user's personal database is required. Runtime dictionary autocomplete adoption is a separate milestone.

## Installer safety

`cargo make db-install` now publishes only verified dictionary artifacts (the database and gzip, manifest, diagnostics, pinned input metadata, attribution, and license notices). It leaves the existing `static/` directory and unrelated Practice, Courses, brand and Docs assets untouched. Obsolete timestamped dictionary files are moved to `target/db-workflow/backups/` for recovery. The preload in `index.html` is changed only after the dictionary files are published. The installer does not regenerate the database.

## Generator and browser responsibilities

- [`zipeek-db-gen`](../../../crates/zipeek-db-gen) verifies pinned inputs, imports HSK, applies source-backed CC-CEDICT enrichment/expansion, builds normalized relations, and publishes a staged, checked and reopened artifact. Database generation is explicit, not part of every Trunk build.
- [`zipeek-db`](../../../crates/zipeek-db/src/wasm) owns the DuckDB-WASM transport, worker/connection and manifest verification. [`zipeek-web` database services](../../../crates/zipeek-web/src/services/database) expose typed read-only queries to pages.
- The manifest's native engine and storage compatibility are required strings; the running DuckDB-WASM engine and `pragma_database_size()` results are queried independently by Debug.
- Local learner data uses browser storage and optional personal-data probes, **not the generated dictionary**. See [personal-data architecture](personal-data.md).

## Maintained references

| Question | Owner |
| --- | --- |
| Actual SQL tables, constraints and HSK compatibility columns | [schema](schema.md) |
| Original sources, parsing rules and reading identities | [sources](sources.md), [import rules](import-rules.md), [PoS mapping](pos-mapping.md) |
| Generator stages, output and validation | [artifacts](artifacts.md), [regeneration](regeneration.md) |
| Actual package pins and compatibility boundary | [runtime](runtime.md), [generator dependencies](generator-deps.md) |
| Browser queries and Browse/Practice behavior | [queries](queries.md), [browser DB architecture](../../architecture/modules/browser-db.md) |
| Logical reproducibility and testing | [reproducibility](reproducibility.md), [browser DB testing](../../testing-quality/e2e/browser-db.md) |
| Unapproved FTS/index redesign | [review proposal](review.md) |

Dependency governance: the installed npm package pin agrees with the generator policy and artifact manifest. Historical decisions about the storage compatibility version describe a prior artifact; do not treat them as source verification. Changes to dependencies or storage formats require explicit user approval.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/dictionary-data-schema-db.md). Dated implementation claims in that snapshot are not current build or browser verification.

## Generator bulk-import and lexical reference additions (R615 WIP)

The native generator now batches `sense_display`, `sense_qualifiers`, `sense_relations`, `characters`, `word_characters` and `vocabulary_identities` inserts. Performance changes require the same verified input set and fresh before/after timestamps; no measured speedup has been established in this authoring environment.

The bundled `crates/zipeek-db-gen/data/relationships.csv` is imported into `word_relations` during full artifact generation. The original spelling pairs and relationship type are preserved, with separate `source_resolution` and `target_resolution` statuses (`resolved`, `missing`, `ambiguous`). A canonical `word_id` is populated only when exactly one `words.simplified` row matches the spelling. Ambiguous forms are **not** expanded into all pronunciation combinations. These lexical relationships are separate from sense-level source-annotation relations.

`pinyin_syllable_candidates` groups numeric character-reading strings by attested characters. `character_gloss_candidates` associates each one-character dictionary entry's raw sense gloss with its character. Neither candidate table is an approved, curated R608 scoreable catalogue. Existing `vocabulary_identities.identity_status = 'unresolved'` also remains non-scoreable until explicit curation.

The CSV is resolved from the db-gen crate's compile-time source location. Running the generator from a source checkout requires the bundled CSV to remain present. It is now part of the dictionary snapshot fingerprint. Test the locked native generator and verify final artifact row counts, relation resolution totals and snapshot equality on the target machine.

## R616 WIP: relation import verification

The relationship importer verifies the row count against the bundled CSV before it reports completion. It records `total`, `fully_resolved`, `ambiguous_endpoint` and `missing_endpoint` in the native build log. An unresolved or ambiguous endpoint is retained in the table, never expanded into guessed word IDs. These last two status counts can overlap when a row has one ambiguous and one missing endpoint.

Focused generator tests cover unambiguous, ambiguous and missing endpoint resolution, plus an incomplete CSV import. The native test runner and full generator remain unverified in the R616 authoring environment.

## R617 WIP: artifact dependency ordering

The generated `character_gloss_candidates` table references `(word_id, sense_order)` in `word_senses`. The artifact writer now copies `word_senses` before this dependent table, preserving the schema's foreign key. A schema-driven unit test checks all foreign-key dependencies between tables in the artifact-copy sequence. The user's 2026-11-23 native generator run reached the artifact-copy stage after populating 267,447 word relationships but failed on the prior copy order; a completed artifact is not yet verified.

Relationship resolution counters are now a mutually exclusive partition: `fully_resolved`, `ambiguous_only`, `missing_only`, and `ambiguous_and_missing`. This avoids double-counting rows that have one ambiguous and one missing endpoint. The source spellings remain intact irrespective of resolution.

The database workflow is implemented by `zipeek-cli`. Use `cargo make db-generate`, `cargo make db-compress`, `cargo make db-install`, then `cargo make db-browse-check`. The final command verifies the installed dictionary manifest, bundled file checksums, pinned inputs, Browse tables and normalized search tables. No Node-based `db-workflow.cjs` implementation is retained.
