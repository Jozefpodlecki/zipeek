# Source-to-documentation accuracy audit

## Scope and evidence

This review compares maintained documentation with the source snapshot in the same project archive. It is **not** a Rust compilation, a DuckDB data-file integrity test, a Chrome observation, or an exhaustive audit of every page. No dependencies or Rust application logic were changed. The current source manifest and manifest/policy code are authoritative for what this snapshot declares; approved UX documents define intended future behavior.

## Verified source facts and corrections

| Subject | Source evidence | Reconciled fact | Verification limit |
| --- | --- | --- | --- |
| Cargo workspace | [`Cargo.toml`](../../Cargo.toml) | 22 member crates, Rust 2024 workspace, resolver 2, root lockfile | Crates not compiled in this review |
| Dictionary rows | [`hanzi.manifest.json`](../../crates/zipeek-web/static/hanzi.manifest.json) | **120,970 `words` rows**, not 12,188 | Manifest declaration only |
| Dictionary identity candidates | Same manifest | **120,970** | Manifest declaration only |
| Dictionary source entries | Same manifest | **11,494** HSK source entries | Source count, not database rows |
| Character occurrences | Same manifest | **312,649** | Manifest declaration only |
| Explicit character readings | Same manifest | **11,846** | Manifest declaration only |
| Diagnostics | Same manifest and [`hanzi.diagnostics.json`](../../crates/zipeek-web/static/hanzi.diagnostics.json) | **10,187** events | Not independently replayed |
| Engine and storage | [`policy.rs`](../../crates/zipeek-db-gen/src/policy.rs), [`writer.rs`](../../crates/zipeek-db-gen/src/artifact/writer.rs), manifest | Native engine `v2.0.0`, explicit storage `v2.0.0` | Browser engine version still requires a live query |
| WASM package | [`package.json`](../../crates/zipeek-web/package.json), generator policy, manifest | `@duckdb/duckdb-wasm@1.33.1-dev114.0` | Package lock and runtime not executed |
| Generated file name | Manifest | `hanzi-20261115-215819.duckdb` and `.gz` | **Neither database binary is included in this source archive** |
| Dictionary tables | Manifest `tables`, [`schema.sql`](../../crates/zipeek-db-gen/src/schema.sql) | `character_readings` **is listed** in the checked-in manifest | Physical database still needs open/query verification |
| Browser startup | [`database_provider.rs`](../../crates/zipeek-web/src/components/providers/database_provider.rs), [`database_boundary.rs`](../../crates/zipeek-web/src/components/providers/database_boundary.rs), [`route.rs`](../../crates/zipeek-web/src/route.rs) | Loading/ready/failure/retry boundary exists, but several routes bypass it | Approved whole-app failure blocking is not implemented |
| Other-tab coordination | [`coordinator.rs`](../../crates/zipeek-web/src/components/shell/app_shell/other_tab/coordinator.rs), [`mod.rs`](../../crates/zipeek-web/src/components/shell/app_shell/other_tab/mod.rs) | BroadcastChannel heartbeats and a **bottom-right warning**, not exclusive first-tab ownership | Browser behavior and crash recovery unverified |
| Header | [`app_shell/mod.rs`](../../crates/zipeek-web/src/components/shell/app_shell/mod.rs) | Centered route title remains part of the general header | Approved dragon-shaped evidence header not implemented |
| Learner storage | [`preferences.rs`](../../crates/zipeek-web/src/preferences.rs), [`word_sets.rs`](../../crates/zipeek-web/src/services/word_sets.rs), [`exclusions.rs`](../../crates/zipeek-web/src/services/exclusions.rs), [`practice_session_store/mod.rs`](../../crates/zipeek-web/src/services/practice_session_store/mod.rs) | Separate browser keys exist for preferences, Word Sets, exclusions and Practice sessions | Persistence not browser-verified |

## Important distinctions

- **12,188** describes distinct reading keys in a historical HSK source grouping and can remain in clearly labeled source-coverage documents. It must not be used as the count of the current expanded `words` table.
- The manifest contains checksums for the generated `.duckdb` and `.gz` assets, but the source ZIP does not include those files. Their absence prevents inspection of actual on-disk rows, FTS index usability, `character_readings` queryability and hash agreement.
- Historical approved decisions may describe a previous storage format. They remain preserved as history; the current checked-in generator and manifest use `v2.0.0`. Any future change to dependencies, generator pins or format requires explicit approval.
- The UI specification's full-screen duplicate-tab block, original-tab ownership and long dragon progress are approved **requirements**, not behaviors shown by the inspected Rust source.
- The database provider already has a retryable failure state, but the route-level boundary is not whole-application blocking. Do not report compliance with the approved startup policy until implemented and browser-tested.

## Remaining gates

1. On the user's Windows toolchain, compile the workspace and WASM target with `--locked`; address compiler errors separately from this documentation audit.
2. Prepare the generated dictionary through the approved explicit workflow, verify manifest hashes, open the real database and inspect `words`, `character_readings`, normalized relations and FTS queries.
3. Test dictionary initialization, failure/retry and route gating in Chrome at root and `/zipeek/`.
4. Test multiple tabs, abrupt termination, stale ownership and write exclusion with actual browser sessions.
5. Verify browser storage round trips, personal OPFS persistence and session recovery in browser E2E before treating source presence as successful data durability.
6. Review remaining specialist implementation documents and E2E assertions beyond this focused architecture/data/runtime inventory.

## Audit discipline

Maintain source truth and approved future requirements separately. A source file proves that code or a manifest declaration exists, not that the deployed application performs correctly. A passed documentation validator proves links and index consistency, not runtime behavior.
