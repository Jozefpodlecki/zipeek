# Reproducibility

The logical database reproducibility rule. Part of the
[database documentation](DB.md). Approved in
[D22](../../process/decisions/database-and-artifacts.md#d22-logical-database-reproducibility).

## Logical reproducibility (D22: approved 2026-10-26)

D22 replaces the former byte-identical-output requirement. The inspected
[DuckDB 1.4.3 file writer](https://raw.githubusercontent.com/duckdb/duckdb/v1.4.3/src/storage/single_file_block_manager.cpp)
contains `GenerateDBIdentifier` using `RandomEngine`; `CreateNewDatabase` calls it
when encryption is enabled or the selected file storage version is at least 67.
The identifier is serialized into the main header. This is conditional: it does
not prove every unencrypted older-format file is nondeterministic. Exact storage
settings have not been selected or exercised, and byte identity is not established.

**Approved D22 rule:** require identical ordered logical rows, IDs, schema/index
definitions, and import/drop diagnostics for identical verified inputs and pinned
tooling. Record SHA-256 of each final artifact for integrity without requiring
different builds to have the same file hash.

CODING_STANDARDS and TESTING now use that logical-content rule. Do not patch, mask,
or hand-edit database headers to force checksums to match. A05 must compare
independent runs at the logical level and record each artifact checksum; physical
checksum equality is not the reproducibility acceptance criterion.

## What "logical" means

The reproducibility contract compares:

- Every column of every row, in `ORDER BY id ASC` order.
- Assigned IDs 1..N, in that order.
- Table, column, constraint, and index definitions.
- The ordered `ImportDiagnostic` list (applied/ignored/unresolved warnings,
  rejections, conflicts, pipe transformations).

It does **not** require:

- Identical `.duckdb` file SHA-256 across runs.
- Identical checkpoint metadata, page ordering, or header bytes.
- Identical file sizes.

## Why files may differ

DuckDB's file writer can generate a random database identifier and serialize it
into the main header when encryption is enabled or storage version ≥ 67. This is
why D22 is logical-only. Any future requirement for byte identity would need a
different storage configuration, which is not approved.

Do not:

- Patch or mask header bytes to make checksums match.
- Hand-edit a database to make logical comparisons pass.
- Declare reproducibility satisfied because one run's SHA-256 happens to match
  another's.

## Where it is verified

- A05 artifact tests: independent exports compared row-by-row, schema-by-schema.
- A05 evidence: `target/a05-repeat` vs `web/static/`, with matching logical hash
  and differing physical hash.
- D22 policy is enforced in the generator at every full-source export.

See [artifacts.md](artifacts.md) for the concrete hashes from the A05 run and
[../../testing/provenance.md](../../testing/provenance.md) for the required
reproducibility checks in any future regeneration.

## Related

- Artifact export and evidence: [artifacts.md](artifacts.md).
- Coding standards reference: [../../process/CODING_STANDARDS.md](../../process/CODING_STANDARDS.md).
- Testing plan: [../../testing/provenance.md](../../testing/provenance.md).