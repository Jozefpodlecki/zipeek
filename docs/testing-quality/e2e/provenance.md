# Dictionary source, import and provenance verification

Source contract: [`zipeek-db-gen/inputs`](../../../crates/zipeek-db-gen/src/inputs), [`generator.rs`](../../../crates/zipeek-db-gen/src/generator.rs), [`schema.sql`](../../../crates/zipeek-db-gen/src/schema.sql), [`artifact/writer.rs`](../../../crates/zipeek-db-gen/src/artifact/writer.rs). The installed [manifest](../../../crates/zipeek-web/static/hanzi.manifest.json) is the authority for current counts, source hashes and output assets.

## Current checks

1. Validate declared HSK and CC-CEDICT input paths, sizes, SHA-256, license notices and engine/storage guards before import.
2. Verify primary HSK reading grouping, original Hanzi/pinyin preservation, missing-gloss handling, PoS and HSK memberships. Verify CC-CEDICT **expansion** as well as fallback; the old “no extra non-HSK readings” rule is no longer current.
3. Exercise source-matching by simplified and normalized reading key, variant rejection, duplicate-key diagnostics, typed transactions and rollback.
4. Validate normalized schema relations in addition to the ten-column `words` compatibility projection: senses, HSK/PoS membership, categories, identities, characters, source-explicit character readings and sense relationships.
5. Verify an exported candidate against the in-memory snapshot, then reopen it read-only and compare again. Never hand-edit database bytes.
6. Verify the generated manifest, diagnostic report, native engine `v2.0.0`, storage format `v2.0.0`, browser package `1.33.1-dev114.0` and installed bundle file hashes.
7. **Open validation gate:** the source manifest lists `character_readings`, but the source archive omits the binary `.duckdb` and gzip assets. Verify actual file hashes, open the database and query the relation before asserting installed data integrity.

Run the generator's locked Rust tests on the native toolchain and follow [regeneration](../../dictionary-data/schema/regeneration.md) for explicit, approved output generation and installation. This is not triggered by ordinary website serving.

Historical D06/D16 HSK-only counts, source audit arithmetic and A06 browser results are preserved in [release history](../../archive-history/RELEASE_HISTORY.md) and [completed](../../archive-history/completed/); they are not the current artifact acceptance totals. See [current DB](../../dictionary-data/schema/db.md) and [source reconciliation](../../archive-history/audits/SOURCE_RECONCILIATION.md).
