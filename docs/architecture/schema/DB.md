# Database

Overview and index for the database documentation. Each section here is
elaborated in a dedicated file below. Source and licensing decisions live in
[D06](../process/decisions/database-and-artifacts.md#d06-sources-and-licensing)
and [D16](../process/decisions/database-and-artifacts.md#d16-primary-import-and-ids);
schema in [D07](../process/decisions/database-and-artifacts.md#d07-physical-schema-and-pool);
import rules in [D18](../process/decisions/database-and-artifacts.md#d18-source-comparison-and-serialization);
runtime pin in [D08](../process/decisions/database-and-artifacts.md#d08-duckdb-wasm-version);
reproducibility in [D22](../process/decisions/database-and-artifacts.md#d22-logical-database-reproducibility).

## Requirements and current state

The planned runtime database is a committed, pregenerated file at
`web/static/hanzi.duckdb`, read-only through DuckDB-WASM in the browser.

The generated database file and provenance bundle are present. `crates/db-gen`
exists as workspace package `db-gen` with a generation-time native-engine guard.
The user restored `duckdb-neo` at Git revision
`4acbea84794582c4cd962ee971a3d7a5dcb72e39` after the bundled 1.4.3 build failed.
Other D21 dependencies are applied and checksum-identified inputs/notices are
preserved in `crates/db-gen/data`. A26 implements typed manifest loading and
verifies every source/notice file (size/SHA-256) before preparing the guarded
in-memory database connection. A02 creates the approved schema and two HSK
indexes in memory. A03 imports 12,100 accepted primary readings in one
transaction from verified source bytes. A04 supplies empty-primary fallback;
A05 exports and verifies the compatible artifact, integrity report, diagnostics
and notices under `web/static/`.

Database generation is a manual native operation, separate from web builds.
Never hand-edit a DuckDB file. Regenerate it after an approved source or schema
change and include the generated output in that change.

The app is in development: breaking data changes are acceptable and affected
databases are regenerated. There is no application/database versioning or
backward-compatibility requirement, and no obligation to migrate old generated
files. Source provenance, licensing, and dependency version controls still apply.

## Files

| File | Contents |
| --- | --- |
| [sources.md](sources.md) | Primary HSK snapshot, CC-CEDICT fallback, licensing, excluded sources |
| [schema.md](schema.md) | `words` DDL, columns, HSK abstraction, index policy |
| [import-rules.md](import-rules.md) | Reading identity, level mapping, text preservation, gloss rules, key conflicts |
| [pos-mapping.md](pos-mapping.md) | 42-code canonical PoS mapping and audit counts |
| [queries.md](queries.md) | Word of the Day query, practice pool queries, right-label normalization, tuning |
| [runtime.md](runtime.md) | DuckDB-WASM pin, engine facts, OPFS rationale |
| [generator-deps.md](generator-deps.md) | D21 dependency pins/removals, neo restoration, compatibility probe |
| [reproducibility.md](reproducibility.md) | D22 logical reproducibility rule |
| [artifacts.md](artifacts.md) | Input preparation, primary import, fallback, artifact export, browser integration |
| [regeneration.md](regeneration.md) | Commands, review/replacement procedure, measured sizing |

## Reading order

For a new contributor:

1. [sources.md](sources.md) — what data is used and why.
2. [schema.md](schema.md) — what the output looks like.
3. [import-rules.md](import-rules.md) + [pos-mapping.md](pos-mapping.md) — how source rows become `words` rows.
4. [queries.md](queries.md) — what runtime asks of the table.
5. [runtime.md](runtime.md) + [generator-deps.md](generator-deps.md) — pinned toolchain facts.
6. [reproducibility.md](reproducibility.md) + [artifacts.md](artifacts.md) — what "generated" means and how it is verified.
7. [regeneration.md](regeneration.md) — how to produce a new file.

## Related

- Architecture overview: [../ARCHITECTURE.md](../ARCHITECTURE.md).
- Generator module: [../modules/db-gen.md](../modules/db-gen.md).
- Browser database: [../modules/browser-db.md](../modules/browser-db.md).
- Decisions: [../../process/decisions/database-and-artifacts.md](../../process/decisions/database-and-artifacts.md).
- Testing: [../../testing/provenance.md](../../testing/provenance.md) and [../../testing/browser-db.md](../../testing/browser-db.md).