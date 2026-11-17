# R184 - Source archival, web database distribution and Settings

## Included in this release

- `crates/zipeek-cli`: native Rust ZIP pack/unpack using the existing `zip 9.4.0` lockfile dependency. Tracks repository source, omits compiled/generated artifacts and symlinks, rejects traversal entries on extraction, stages extraction, backs up replaced source trees and overlays the remainder. Managed directories (including `crates/*/src`, `docs`, `scripts`, `crates/web/static`) are replaced so deleted source files do not linger.
- `cargo make pack` / `cargo make unpack`: default artifact `target/zipeek-source.zip`. Explicit paths supported through the CLI directly. Archives are **not signed**, so only unpack files from trusted sources.
- `cargo make db-compress`: Gzip-compresses the latest verified candidate to `hanzi-*.duckdb.gz`, records size/SHA-256, revalidates the complete bundle; never installs. `db-full` includes compression before installation. The installer renames raw and compressed artifacts in sync when timestamping.
- Compressed installed database shipped alongside raw DuckDB for development. With `23,080,960` uncompressed bytes the Gzip asset is `5,350,281` bytes (76.82% smaller), checked for exact decompression and SHA-256.
- Browser bridge prefers Gzip, checks the compressed hash and size, decompresses through native `DecompressionStream`, verifies original size and hash, then registers the raw bytes with DuckDB-WASM. An early manifest-aware `<link rel="preload" as="fetch" crossorigin>` hints the exact asset; measure browser network results because cache behavior varies.
- Complete version-policy update to `@duckdb/duckdb-wasm@1.33.1-dev114.0` in web JS preparation, database generator policy, bundle publisher, tests and installed bundle metadata.
- Independent Settings Image Generation panel with persistent API URL, reset and health probe. Native image-server CORS permits the two Trunk development origins, not arbitrary internet origins.
- Five focused web feature refactors: Settings image-server panel, Practice Setup route preselection, Cloud Match vocabulary-pool loader, Word Spotlight data loader, Practice Session round diagnostics.

## Manual commands

```powershell
cargo check -p zipeek-cli --locked
cargo run -p zipeek-cli --locked -- pack
cargo run -p zipeek-cli --locked -- unpack
cargo make db-generate
cargo make db-classify
cargo make db-compress
cargo make db-install
cargo check -p zipeek --target wasm32-unknown-unknown --locked
cargo check -p zipeek-image-server --locked
cargo make serve
```

**Unpack warning:** the unpack command intentionally replaces source-owned directories and overlays repository files. Keep a clean commit or external backup before using it; a timestamped backup is created under `.unpack-backups/`.

## Validation gates

- Node regression suite and new gzip tests must pass.
- Native Rust compile, WASM compile, Trunk browser startup and image-server CORS must be verified on Windows. Do not treat static Node contracts as substitutes for Cargo checks.
- Check that DuckDB-WASM `1.33.1-dev114.0` actually opens the installed native-v2.0.0 database, and inspect Network for duplicate preload transfers.
- Physical DuckDB rebuilding/compaction (`COPY FROM DATABASE`) is a separate optional follow-up, requiring comparisons of schemas, indices and query results. `VACUUM` alone does not guarantee file shrinkage.
- Image exercises, their scheduling and image approval workflow are not included in R184.
