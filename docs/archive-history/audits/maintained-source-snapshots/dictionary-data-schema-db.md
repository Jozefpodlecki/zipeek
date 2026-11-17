# Dictionary database: current implementation

This index describes the **checked-in R282 source and installed manifest**, not an original HSK-only proposal. The physical SQL definition in [`schema.sql`](../../../../crates/zipeek-db-gen/src/schema.sql), the generator in [`generator.rs`](../../../../crates/zipeek-db-gen/src/generator.rs), and [`hanzi.manifest.json`](../../../../crates/zipeek-web/static/hanzi.manifest.json) are the evidence for the installed layout. The database is generated natively and opened read-only by the browser; it is not a writable learning-history store.

## Installed artifact

| Manifest fact | Installed value |
| --- | --- |
| Reading rows | **12,188** |
| Identity candidates | 12,188 |
| Source entries | 11,494 |
| Diagnostic events | 467 |
| Native generator engine | `v2.0.0` |
| Storage compatibility | `v1.4.0` |
| DuckDB-WASM npm package | `@duckdb/duckdb-wasm@1.33.1-dev114.0` |
| Read-only database file | `hanzi-20261108-183418.duckdb` |
| Compressed browser asset | `hanzi-20261108-183418.duckdb.gz` |

The manifest includes checksums, source metadata, table names and generated-time properties. The original **12,100-row, 10-column HSK-only snapshot** is historical, not the installed database. The `words` compatibility projection remains ten columns, but normalized tables, senses, membership, classifications and identity candidates are part of the generated schema. Generator SQL also defines explicit character readings, though their presence in the installed artifact remains to be verified. See [schema](../../../dictionary-data/schema/schema.md).

## Generator and browser responsibilities

- [`zipeek-db-gen`](../../../../crates/zipeek-db-gen) verifies pinned inputs, imports HSK, applies source-backed CC-CEDICT enrichment/expansion, builds normalized relations, and publishes a staged, checked and reopened artifact. Database generation is explicit, not part of every Trunk build.
- [`zipeek-db`](../../../../crates/zipeek-db/src/wasm) owns the DuckDB-WASM transport, worker/connection and manifest verification. [`zipeek-web` database services](../../../../crates/zipeek-web/src/services/database) expose typed read-only queries to pages.
- The manifest's native engine and storage compatibility are required strings; the running DuckDB-WASM engine and `pragma_database_size()` results are queried independently by Debug.
- Local learner data uses browser storage and optional personal-data probes, **not the generated dictionary**. See [personal-data architecture](../../../dictionary-data/schema/personal-data.md).

## Maintained references

| Question | Owner |
| --- | --- |
| Actual SQL tables, constraints and HSK compatibility columns | [schema](../../../dictionary-data/schema/schema.md) |
| Original sources, parsing rules and reading identities | [sources](../../../dictionary-data/schema/sources.md), [import rules](../../../dictionary-data/schema/import-rules.md), [PoS mapping](../../../dictionary-data/schema/pos-mapping.md) |
| Generator stages, output and validation | [artifacts](../../../dictionary-data/schema/artifacts.md), [regeneration](../../../dictionary-data/schema/regeneration.md) |
| Actual package pins and compatibility boundary | [runtime](../../../dictionary-data/schema/runtime.md), [generator dependencies](../../../dictionary-data/schema/generator-deps.md) |
| Browser queries and Browse/Practice behavior | [queries](../../../dictionary-data/schema/queries.md), [browser DB architecture](../../../architecture/modules/browser-db.md) |
| Logical reproducibility and testing | [reproducibility](../../../dictionary-data/schema/reproducibility.md), [browser DB testing](../../../testing-quality/e2e/browser-db.md) |
| Unapproved FTS/index redesign | [review proposal](../../../dictionary-data/schema/review.md) |

Dependency governance: the older version target in [`AGENTS.md`](../../../../AGENTS.md) and historical decisions disagrees with the installed `package.json`/generator policy. This documentation records the **actual source pin**; any change to the approved policy requires an explicit decision, not a silent source or package change.
