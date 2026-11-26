# Native DuckDB for Tauri

The Yew dictionary service chooses its backend before any browser DuckDB module, Worker, or dictionary download is started. Browser builds use DuckDB/WASM; Tauri uses `database_open` and `database_query` via a Rust-owned IPC bridge. Shared web query methods continue to deserialize into existing typed vocabulary models.

The app registers `NativeDatabase` with `.manage()` during setup. A dedicated worker thread owns two separate DuckDB connections. The dictionary connection attaches only the packaged dictionary in read-only mode and handles the generic dictionary-query IPC command. The personal connection attaches only the writable desktop Practice-history file and is accessible solely through the dedicated history commands. No dictionary IPC SQL can query personal tables because those tables are not attached to its connection. The worker serializes requests and never passes connection handles across threads. It uses `ZIPEEK_NATIVE_DICTIONARY` when set, a packaged resource `dictionary/hanzi.duckdb` for installers, and `binaries/dictionary/hanzi.duckdb` staged from the installed dictionary manifest while developing.

Install a verified dictionary before `cargo make desktop-build`; reuse the current installation, or explicitly regenerate/install with `cargo make db-full` when required. `desktop-sidecars` selects exactly one uncompressed dictionary from `static/hanzi.manifest.json`, verifies engine/storage compatibility, size and SHA-256, then stages it as `binaries/dictionary/hanzi.duckdb`. It does not require a compressed browser copy or regenerate the dictionary. Tauri bundles its native database bytes and the pinned DuckDB runtime DLL in `bundle.resources`; the desktop frontend must not load the browser WASM database engine.

## Scope and outstanding work

Dictionary lookup and Practice learning-history commit/read/erase now route through native IPC. The native Practice evidence file is in Tauri's application-local data directory, and duplicate session IDs are checked against existing evidence. **Other storage remains browser-based**: word sets, exclusions, backups, recognition history, and personal-data snapshots still need a native data adapter. Debug's browser OPFS compatibility probe is skipped in Tauri. Do not interpret dictionary and Practice history IPC as complete desktop storage integration. Browser and desktop data are independent; no migration or fallback is implemented. Native read-only SQL must not be exposed to untrusted web content beyond the locally bundled frontend; the guard is defense in depth, not a SQL sandbox.

## Frontend and contract separation

Application initialization detects Tauri once and initializes `DatabaseBackend`. Dictionary and Practice-history services use that selection; a native IPC failure never selects the browser engine. Generated desktop HTML declares its platform and fails startup when Tauri IPC is missing. Browser HTML retains its existing preloads.

`zipeek-db::contracts` owns serializable `DatabaseInfo` and `HistoryRow`. `zipeek-db::PracticeOutcomeSummary` and `EvidenceError` are pure Rust and run in native tests. Native handlers and the frontend IPC adapter share these contracts. Browser evidence-validation errors adapt into the WASM transport error type. The native driver types are exposed through the existing `zipeek-db::native` facade; no dependency is added to the desktop shell.

The shared frontend still compiles Rust browser-adapter code. Its desktop asset output contains no DuckDB-WASM engine, Arrow module or database worker. This is runtime backend selection plus separate frontend assets, not a replacement of the Yew/WASM UI or completion of every browser-storage operation.

## Personal reset command

The user-approved Settings reset adds `personal_data_reset(confirmed: bool)`. It never accepts a filesystem path or deletes the dictionary. The existing serialized native worker closes its current connections before removing the known personal file and WAL. A subsequent dictionary open recreates empty personal storage through the normal path. Errors propagate as structured database failures and keep the frontend reset runner in its retry state. The frontend also clears owned WebView data in an isolated bootstrap, guarded by an exclusive application-session lock. This destructive reset boundary does not implement the remaining native Word Set, recognition or snapshot adapters. Full behavior is owned by [personal storage](../../dictionary-data/personal/personal-data-storage.md#confirmed-app-reset-2026-11-26).
