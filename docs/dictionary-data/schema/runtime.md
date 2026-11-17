# DuckDB browser runtime and artifact compatibility

## Current, source-verified facts

- Browser npm dependency: **`@duckdb/duckdb-wasm@1.33.1-dev114.0`**, from [`package.json`](../../../crates/zipeek-web/package.json) and its lockfile. The installed [`hanzi.manifest.json`](../../../crates/zipeek-web/static/hanzi.manifest.json) and [`db-gen policy.rs`](../../../crates/zipeek-db-gen/src/policy.rs) agree.
- Native generator engine: **`v2.0.0`**; explicitly written disk storage: **`v2.0.0`**. Neither is the browser's running engine version; Debug obtains the latter from the runtime connection.
- The browser is designed to fetch the timestamped compressed `.duckdb.gz` asset and opens the dictionary read-only through the [`zipeek-db` WASM adapter](../../../crates/zipeek-db/src/wasm). Assets must be prepared locally for Trunk and `/zipeek/` deployment; the app does not generate a database on load. The source ZIP contains the manifest and diagnostics but not the generated `.duckdb` or `.duckdb.gz` files. Their existence, hashes and runtime load are not verified by inspecting this source archive.
- The browser-facing dictionary service in [`services/database/runtime.rs`](../../../crates/zipeek-web/src/services/database/runtime.rs) owns typed SQL metadata queries, including `pragma_database_size()`. UI code does not receive a public generic SQL runner.
- The personal OPFS probe and JSON backup flow are **not** an authoritative learning-results database. See [personal data](personal-data.md).

## Approved dependency policy

The user approved **`@duckdb/duckdb-wasm@1.33.1-dev114.0` on 2026-11-09**, amending historical D08. The exact package manifest, lockfile, generator policy and installed artifact agree with this pin. Future substitutions still require approval. This is a policy reconciliation, not a dependency upgrade or a new Chrome compatibility test; retain actual browser acceptance separately from a source-version match. The older `1.32.0`/engine-`1.4.3` combination is historical, not current runtime fact.

## Installed artifact inspection

The checked-in manifest reports **120,970** `words` rows, **120,970** identity candidates and **10,187** diagnostic events, includes normalized tables and lists `hanzi-20261115-215819.duckdb` and its gzip asset. It is the authority for artifact name, checksums and sizes. Historical 12,100-row and 3.1 MB examples belong to release history, not runtime acceptance.

See [database index](db.md), [regeneration](regeneration.md) and [browser DB testing](../../testing-quality/e2e/browser-db.md). Source inspection does not substitute for a fresh Chrome run.
