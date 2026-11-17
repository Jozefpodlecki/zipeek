# Browser dictionary adapter and query ownership

## Implemented boundaries

- [`zipeek-db/src/wasm/`](../../../../crates/zipeek-db/src/wasm) owns the DuckDB-WASM bridge, local worker asset resolution, manifest validation, transport and connection lifetime. It is not a separate `zipeek-db-wasm` crate.
- [`zipeek-web/src/services/database/`](../../../../crates/zipeek-web/src/services/database) owns application-facing typed read-only queries, result projection and metadata. Its runtime SQL remains private; Debug uses `Database::storage_statistics()`.
- [`zipeek-web/src/components/browse/`](../../../../crates/zipeek-web/src/components/browse) drives Browse searches and pagination; [`services/database/browse_queries.rs`](../../../../crates/zipeek-web/src/services/database/browse_queries.rs) implements the active normalized Browse query. This is **not** an unconnected future query proposal.
- [`zipeek-db/src/wasm/probe.rs`](../../../../crates/zipeek-db/src/wasm/probe.rs) is a separate writable personal-data **probe**, not a production learner-history database.

## Runtime and artifact facts

Current web npm pin: **`@duckdb/duckdb-wasm@1.33.1-dev114.0`**; native generator engine `v2.0.0`, explicit disk compatibility `v1.4.0`. The browser engine must be read from the live runtime, not inferred from native versions. The installed manifest declares **12,188** reading rows, not the original 12,100-row snapshot. See [runtime](../../../dictionary-data/schema/runtime.md) and [installed artifact](../../../dictionary-data/schema/db.md) for evidence and the outstanding approval-policy mismatch.

The dictionary uses the packaged local JS/WASM worker and Arrow assets. Source preparation is explicit (`zipeek-cli`/Cargo Make); Trunk deploys prepared assets and loads the timestamped compressed dictionary below the correct `/` or `/zipeek/` base. The dictionary itself is opened read-only and is never the destination for learner outcomes. Browser readiness is owned by the database provider and boundary, with a retry path for failures.

## Real Browse behavior and test boundaries

`BrowseSearchForm` and `BrowseResults` render the active query UI. The service applies typed filters to normalized dictionary tables and orders pages by word ID. Pagination uses the presence of an extra result to establish Next rather than inventing a total count. User-reported Windows E2E results passed Browse word-set discovery, membership, accessibility, empty search, live search and pagination on the earlier R282 source; they do not certify every combination of category/PoS filters in the latest UUID translation build.

## Not yet a supported contract

An optional future adapter-crate extraction and index/FTS redesign are proposals in [target architecture](../../../architecture/modules/target-architecture.md) and [schema review](../../../dictionary-data/schema/review.md). Do not present historical A13/A22 or R12 plans as current limitations in conflict with an implemented Browse query.

See [database schema](../../../dictionary-data/schema/schema.md), [E2E](../e2e.md), [current validation state](../../../project-management/current.md).
