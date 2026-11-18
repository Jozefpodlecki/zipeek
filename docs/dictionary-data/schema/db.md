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
