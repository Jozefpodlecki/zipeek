R706 WIP: image generation defaults to SDXL, with explicit model prerequisite and preview-mode rejection; no model weights bundled. Locked WASM/native tests remain pending.

R706 WIP: Profile autocomplete and compact Practice resume UX implemented. Browser acceptance and locked WASM compilation outstanding.

R706 WIP (2026-11-27): Implemented Profile route with Word Sets, Excluded Words and drawer visibility tabs. Word Sets uses existing storage and an icon-only add action; exclusions use existing storage; drawer preferences persist and keep Profile/Settings accessible. WASM and browser acceptance outstanding.

R705 WIP (2026-11-27): Consolidated frontend Tauri IPC behind `services/tauri/mod.rs` with a window-initialized `TauriService`, cached native bridge, typed command definitions, structured errors, and owned event-listener cleanup. Migrated native database, desktop service management, AI and image generation to the transport; removed duplicate reflection/invoke implementations. Target-specific APIs remain unchanged and no dependency or database policy changed. Contract tests added for command names and camelCase fields. Rust/WASM checks and desktop runtime acceptance remain pending.

R705 WIP (2026-11-27): Dragon status popup reduced and made dismissible by outside pointer and Escape. Desktop diagnostics now offer Tauri native personal database path, file size and file-manager reveal, while web displays OPFS. Profile/drawer configuration remains planned; Word Sets route remains functional. Pending WASM/Tauri compilation and browser verification.

R705 WIP (2026-11-27): Expanded the canonical assessed item evidence beyond Character Builder to Type Pinyin and Word Mosaics. Captures explicitly visible pinyin answer hints as assistance; tracks which mosaic pair targets participated in incorrect attempts; conservatively marks corrected or assisted aggregate rounds partial instead of fully correct. These capability results remain observations, not Knowledge Points. Expanded per-step observation size and multi-round evidence limit to retain up to 20 rounds of 8 mosaic targets. R705 is an unverified numbered release candidate: the 20-qualifying-Rust-file major-release gate and Rust/WASM/browser tests remain outstanding.

Next: authorized target manifests, transactionally applied KnowledgeLedger events, lossless Endless history compaction, and user-toolchain compilation. The existing 1,000-step limit, scoring thresholds and no-.unpack-backups behavior remain unchanged.

R705 WIP (2026-11-27): compact in-session vocabulary presentation, removed visible exercise/round counters and Character Builder instruction chrome, added mouse HTML drag and touch pointer-drop selection/reorder, centralized Mandarin native playback recovery with deduplicated toast notifications. Source integrity checked; WASM/browser acceptance pending. Existing scoring policy and Endless ceiling unchanged.
