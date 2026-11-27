R706 WIP: Root `scripts/` removed; active automation commands route through Rust CLI or Rust tests, with no hidden Cargo fallback for pack/unpack. Desktop staging uses the prebuilt CLI. Native build and task validation pending.

R706 WIP: `zipeek-db-gen` now supplies library-only generation APIs. `zipeek-cli db-gen` owns Clap commands and invokes the generator library; the native DuckDB dependency, scoped session, and transaction implementations are owned by `zipeek-db`. Cargo version requirements use normal semver syntax in touched manifests. Locked CLI/generator tests and regeneration remain unverified.

R706 WIP: Artifact source hygiene pass moves the published attribution source into maintained dictionary docs, centralizes 25 snapshot SQL statements in `artifact/queries.rs`, and constructs reports using `ArtifactReport::new(ArtifactReportOptions)` with chrono UTC formatting. Locked `chrono` 0.4.45 is added as a direct generator dependency; cargo formatting, tests and Clippy remain unverified in this environment.

R706 WIP: Generator SQL/transaction boundaries now use a scoped `DatabaseSession` over the existing owned `GeneratorDatabase`. Artifact writer, FTS, dictionary import, and classification-cache transactions use it; the old artifact SQL execute helper and standalone transaction module are retired. Source package verification pending. Local `cargo fmt --all`, `cargo test -p zipeek-db-gen --locked`, Clippy, generator regeneration and DuckDB-WASM checks remain unverified.

R706 fixture-policy reconciliation: aligned three generator test manifests and policy-override fixture with approved exact DuckDB-WASM 1.33.1-dev114.0. User's 78 unit tests passed; four artifact integration tests previously failed at fixture preflight. Full rerun pending.

R706 WIP generator refactor: consolidated transaction lifecycles, owned staging/publish cleanup, streaming artifact SHA-256. Source-only review; Cargo/test and database generation require local validation. R706 dictionary search regeneration and runtime adoption remain pending.

R706 WIP: Dictionary generator quality checkpoint: extracted FTS and SQL helpers from artifact writer, replaced brittle relationship CSV test probes with exact aggregate assertions. User reported 69/70 db-gen tests passing before these changes; native retest and binary regeneration still pending. Future Practice navigation/feedback UX requirements recorded in TASKS only.

R706 WIP: Approved dictionary search regeneration. Generator schema, lexical search documents, normalized pinyin/Hanzi keys, FTS target, read-only verification and regression coverage updated. Binary regeneration and runtime compatibility still blocked by missing local Cargo/DuckDB executable; the checked-in dictionary artifact and manifest remain unchanged. Next: run locked generator, native/WASM verification and install, then migrate runtime autocomplete.

R706 WIP: Refactored Profile Excluded Words into a single state reducer and repaired missing Profile icon imports. Locked WASM and interactive browser verification pending.

R706 WIP: image generation defaults to SDXL, with explicit model prerequisite and preview-mode rejection; no model weights bundled. Locked WASM/native tests remain pending.

R706 WIP: Profile autocomplete and compact Practice resume UX implemented. Browser acceptance and locked WASM compilation outstanding.

R706 WIP (2026-11-27): Implemented Profile route with Word Sets, Excluded Words and drawer visibility tabs. Word Sets uses existing storage and an icon-only add action; exclusions use existing storage; drawer preferences persist and keep Profile/Settings accessible. WASM and browser acceptance outstanding.

R705 WIP (2026-11-27): Consolidated frontend Tauri IPC behind `services/tauri/mod.rs` with a window-initialized `TauriService`, cached native bridge, typed command definitions, structured errors, and owned event-listener cleanup. Migrated native database, desktop service management, AI and image generation to the transport; removed duplicate reflection/invoke implementations. Target-specific APIs remain unchanged and no dependency or database policy changed. Contract tests added for command names and camelCase fields. Rust/WASM checks and desktop runtime acceptance remain pending.

R705 WIP (2026-11-27): Dragon status popup reduced and made dismissible by outside pointer and Escape. Desktop diagnostics now offer Tauri native personal database path, file size and file-manager reveal, while web displays OPFS. Profile/drawer configuration remains planned; Word Sets route remains functional. Pending WASM/Tauri compilation and browser verification.

R705 WIP (2026-11-27): Expanded the canonical assessed item evidence beyond Character Builder to Type Pinyin and Word Mosaics. Captures explicitly visible pinyin answer hints as assistance; tracks which mosaic pair targets participated in incorrect attempts; conservatively marks corrected or assisted aggregate rounds partial instead of fully correct. These capability results remain observations, not Knowledge Points. Expanded per-step observation size and multi-round evidence limit to retain up to 20 rounds of 8 mosaic targets. R705 is an unverified numbered release candidate: the 20-qualifying-Rust-file major-release gate and Rust/WASM/browser tests remain outstanding.

Next: authorized target manifests, transactionally applied KnowledgeLedger events, lossless Endless history compaction, and user-toolchain compilation. The existing 1,000-step limit, scoring thresholds and no-.unpack-backups behavior remain unchanged.

R705 WIP (2026-11-27): compact in-session vocabulary presentation, removed visible exercise/round counters and Character Builder instruction chrome, added mouse HTML drag and touch pointer-drop selection/reorder, centralized Mandarin native playback recovery with deduplicated toast notifications. Source integrity checked; WASM/browser acceptance pending. Existing scoring policy and Endless ceiling unchanged.
