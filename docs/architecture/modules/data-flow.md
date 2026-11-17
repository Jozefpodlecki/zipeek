# Dictionary and browser data flow

The native generator (`crates/zipeek-db-gen`) ingests verified lexical inputs, produces and checks a compatible DuckDB file, and publishes a manifest and source notices. The installed files are under `crates/zipeek-web/static/`, including timestamped `hanzi-*.duckdb` and compressed `hanzi-*.duckdb.gz` assets. Generation and installation are explicit native tasks, not automatic Trunk build steps.

The browser uses the pinned DuckDB-WASM assets to load the packaged dictionary read-only. Manifest metadata distinguishes the generator's engine/storage format from the running WASM engine. Typed Rust database services serve dictionary search, Word Sets lookup, Practice vocabulary queries and Debug storage statistics. A failed load must expose a recovery state; mounting the Yew component alone does not mean database readiness.

`crates/zipeek-web/index.html`, Trunk and `crates/zipeek-web/public/` provide the static page and bundled assets. The GitHub Pages mount is `/zipeek/`; local development serves the same application from `/`. E2E tests cover root and Pages URLs with a common route catalog.

Learner-generated information is separate from the read-only dictionary. Word Sets, Pinyin progress, exclusions and preferences have browser storage keys and explicit backup semantics. Temporary UUID Practice records can be resumed but are removed on completion. Private OPFS/DuckDB work is not an active pedagogical-history sink.

For the exact pinned sources, regeneration steps, schema and current storage-version requirements, read [database schema](../../dictionary-data/schema/db.md), [generator](db-gen.md), [regeneration](../../dictionary-data/schema/regeneration.md) and [runtime](../../dictionary-data/schema/runtime.md). This overview is intentionally not another historical manifest or schema-version contract.
