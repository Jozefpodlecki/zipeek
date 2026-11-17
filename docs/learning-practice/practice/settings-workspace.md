# Settings workspace: implemented behavior

The Settings route is [`#/settings`](../../../crates/zipeek-web/src/route.rs). The active panel is managed by [`pages/system/settings/mod.rs`](../../../crates/zipeek-web/src/pages/system/settings/mod.rs) and its seven-tab [`navigation.rs`](../../../crates/zipeek-web/src/pages/system/settings/navigation.rs). This is the current layout, not the earlier five-section the historical source proposal.

| Tab | Current ownership |
| --- | --- |
| Appearance | Preset, light/dark and layout density via the shared preferences/theme state |
| Chinese Characters | Hanzi font preference and Character Builder sizes (Standard, Large, Extra large) |
| AI Translation | OpenAI-compatible endpoint, model ID, memory-only optional API key |
| Image Generation | Local image server configuration and test controls |
| Practice | Practice/matching audio preference |
| Personal Data | JSON export/import/preview, category deletion and experimental OPFS/probe operations |
| Pedagogy | Explicit read-only Rust/WASM worker availability check, no consent/recording activation |

## Preferences and credentials

[`preferences.rs`](../../../crates/zipeek-web/src/preferences.rs) stores a JSON document at `zipeek.preferences.v1` with ten fields: `preset`, `density`, `dark`, `font`, `character_size`, `matching_autoplay`, `image_server_url`, `ai_url`, `ai_model`, `learning_history_enabled`. The source supplies defaults for omitted fields. The saved model and endpoint are shared with both the public translator and Practice generated translation; neither question should expose another editable model/URL form. [`services/ai/config.js`](../../../crates/zipeek-web/src/services/ai/config.js) keeps an optional API key only in JS module memory, not in the JSON preferences. It may be lost on reload.

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
