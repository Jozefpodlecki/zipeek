# Settings workspace: implemented behavior

The Settings route is [`#/settings`](../../../crates/zipeek-web/src/route.rs). The active panel is managed by [`pages/system/settings/mod.rs`](../../../crates/zipeek-web/src/pages/system/settings/mod.rs) and its seven-tab [`navigation.rs`](../../../crates/zipeek-web/src/pages/system/settings/navigation.rs). This is the current layout, not the earlier five-section the historical source proposal.

| Tab | Current ownership |
| --- | --- |
| Appearance | Preset, light/dark and layout density via the shared preferences/theme state |
| Chinese Characters | Hanzi font preference and Character Builder sizes (Standard, Large, Extra large) |
| AI Translation | OpenAI-compatible endpoint, model ID, memory-only optional API key |
| Image Generation | Local image server configuration and test controls |
| Practice | Practice/matching audio preference |
| Learning | Practice history preference/status, saved outcome inspection, grouped backup/restore and deletion, and diagnostic worker check |


## Preferences and credentials

[`preferences.rs`](../../../crates/zipeek-web/src/preferences.rs) stores a JSON document at `zipeek.preferences.v1` with ten fields: `preset`, `density`, `dark`, `font`, `character_size`, `matching_autoplay`, `image_server_url`, `ai_url`, `ai_model`, `learning_history_enabled`. The source supplies defaults for omitted fields. The saved model and endpoint are shared with both the public translator and Practice generated translation; neither question should expose another editable model/URL form. [`zipeek-browser/src/ai_configuration.rs`](../../../crates/zipeek-browser/src/ai_configuration.rs) keeps an optional API key in thread-local browser module memory, not in the JSON preferences. It may be lost on reload.

Selecting a palette does not itself change dark/light mode. Font preference does not install a missing host font. Character Builder tiles share the selected font and size; pinyin labels are separate.

## Practice learning-history recording (unverified)

The Personal Data tab now exposes an explicit opt-in for saving completed Practice outcomes to a dedicated OPFS-backed personal DuckDB file (`zipeek-personal-learning-v1.duckdb`). The preference is off by default and stored as `learning_history_enabled` in the existing `zipeek.preferences.v1` JSON document. The former standalone consent key is no longer read or migrated; existing users who opted in using only that key must opt in again. The record includes session/step IDs, exercise IDs and outcomes only, not typed answers, AI-generated sentences, microphone audio or handwriting strokes. Completed temporary sessions are retained until a successful checkpoint, close and readback verification, with a retry control on failure. With recording off, the existing temporary cleanup remains the default.

The tab also offers a separately confirmed erase-history action. The existing portable JSON backup does **not** export or restore the personal DuckDB history. The database writer, Settings control, deletion flow and associated Rust code have not yet been browser/WASM tested; do not represent the checkpoint as a verified storage service. Authoritative progress aggregation remains unimplemented; the `/stats` page is currently a non-authoritative empty-state dashboard.

## worker availability diagnostics

The Pedagogy tab offers an explicit, one-shot health probe through the read-only the historical source monitor. The JavaScript bridge resolves `pedagogy-worker/` from `document.baseURI`, preserving `/` and `/zipeek/` deployment paths. The monitor starts an isolated worker for the probe and terminates it in `finally`. A missing generated worker WASM bundle is displayed as an error rather than a healthy or running engine. This diagnostic cannot enable tracking, read DuckDB history, enqueue analysis, or write mastery records. A successful probe confirms worker availability only. Live browser verification is pending.

## Test evidence and limitations

Rust browser suites for Settings, font persistence, Debug and panel navigation exist under [`crates/zipeek-e2e/tests/`](../../../crates/zipeek-e2e/tests). Tests authored in source are not the same as test runs: current Windows confirmation is listed in [CURRENT](../../project-management/current.md). Browser/screen-reader keyboard behavior and OPFS durability require their own acceptance evidence.

## Future storage work

Add verified portable history export/import, retention policy, schema migrations, user-data deletion integration, concurrency and quota recovery coverage before the personal database is considered fully production-ready. See [learning evidence](learning-evidence.md).

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/learning-practice-practice-settings-workspace.md). Dated implementation claims in that snapshot are not current build or browser verification.

## R596 Learning Data clarity checkpoint

The current settings UI displays the persisted `learning_history_enabled` preference as Practice history saving: On, Off or Unavailable. This preference is not a pedagogy-analysis enablement switch. A separate Pedagogical assessment status explicitly says the analysis is not connected to saved Practice evidence. The worker check remains a one-shot availability diagnostic under Technical diagnostics and cannot record or award proficiency. Backup/restore and deletion are grouped under collapsed sections, with separate history inspection and irreversible-history confirmation. Source changes are unverified by Rust/WASM/Chrome.

The approved long evidence-driven dragon in `docs/product-ux/foundation/header-and-dragon.md` is still open. Current `AppShell` uses a small conditional SVG and redundant centered route title; this does not satisfy the artwork or authoritative evidence contract.

## R600 Learning controls and ownership

The Settings > Learning page is limited to the persisted Practice recording preference, the current absence of authoritative pedagogical assessment, personal data backup/restore, deletion, and technical diagnostics. Counts, summaries, and activity history inspection belong to Statistics rather than Settings.

`settings/learning/recording.rs` owns the sole recording toggle and listens to the existing same-tab change event and cross-tab storage event. Persistence remains in `services/practice_history.rs`; a failed write does not display the requested value as saved. `settings/learning/progress.rs` presents no fictional proficiency or activity totals. `settings/learning/diagnostics.rs` owns only a local worker availability request. `personal_storage/` retains backup and deletion workflows; deleting Practice outcome history is a separate confirmation because the portable JSON backup excludes it.

R600 is source-only until Rust/WASM and browser accessibility tests pass.


## Recording preference versus assessed knowledge

The `learning_history_enabled` setting is a default-off preference to save completed Practice outcomes to personal DuckDB/OPFS. It is **not** a proficiency engine switch. A successful diagnostic worker health probe only confirms a reachable worker, not evidence ingestion, persisted assessment or active mastery. Recording status is refreshed through same-tab `zipeek-recording-change` and cross-tab `storage` events in the R596 source review, pending current browser verification. Backup/Restore, Delete and technical worker diagnostics are distinct settings responsibilities. The proposed R608 [Knowledge Points](../pedagogy/knowledge-points.md) model requires a dedicated policy/reset contract and durable validated evidence; it is not enabled by the recording toggle.
