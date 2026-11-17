# Learner end-to-end scenario matrix (R204)

All browser cases run against the managed `zipeek-e2e` Chrome + Trunk harness. Each case starts with cleared browser storage and an isolated fixture; never add hidden production behavior merely to satisfy an E2E test. Tests in `tests/suites/learner_journeys.rs` are new scenarios, while existing suites are cited where they already cover the requested behavior. **Listed tests are not claimed to have run successfully until Chrome/Trunk execution is recorded.**

| # | User journey | Current executable coverage | Gap to close |
| --- | --- | --- | --- |
| 1 | Initial loader is visible until dictionary ready | `bootstrap::mounted_app_waits_for_dictionary_download`; bootstrap blocked-WASM theme/layout test | Revalidate against the R196+ Rust appearance startup change |
| 2 | Initial loader fails and offers Retry | `database::dictionary_failure_retry_and_real_read_only_queries` blocks dictionary network request and retries | Add failure phases for malformed manifest and worker initialization if supported by fixture |
| 3 | Landing page loads; navigation tiles actually navigate | **New** `learner_journeys::homepage_tiles_navigate_to_product_areas`; `shell::landing_page_loads`; refreshed `shell::responsive_shell_preserves_navigation_theme_and_keyboard_access` for the fourth tile | Add visual snapshot baselines for new landing assets |
| 4 | Basic practice: choose a mode, finish five rounds, reach summary | `fixed_mcq::fixed_mcq_counts_stats_and_restart`, `practice::practice_rounds_normalize_and_replay` | R204 updated the old setup suite for the stepper; older MCQ/mixed test helpers still need migration. Complete an uninterrupted Home-to-five-round summary journey |
| 5 | Advanced AI practice: complete five exercises | `tests/ai_learner.rs` + `fixtures/ai-learner.cjs` test AI responses and errors; **not yet** full five-exercise browser journey | Deterministic 5-round test with controlled LM Studio-compatible AI fixture, stable vocabulary, success and timeout paths |
| 6 | Browse filter, audio button, add a word to a Word Set | **New** `learner_journeys::browse_filter_listen_and_save_word`; `recent_routes::dictionary_autocomplete_keyboard_and_pagination` | Acoustic TTS verification requires a browser speech fixture; current journey checks the click and saved-word persistence |
| 7 | Word Sets: create, rename, change selected set, delete | **New** `learner_journeys::word_set_rename_switch_and_delete`; existing persistence, backup and deletion cases | R518 persists the selected Dictionary save destination; Chrome test must confirm reload and deleted-target fallback |
| 8 | Write Hanzi | **New** route smoke in `secondary_product_routes_render`; existing writing component tests | Draw real pointer strokes, undo/clear and validate feedback |
| 9 | Translate sentence | **New** route smoke; `tests/translation_fixture.rs` + AI fixture | **R204:** `translator_diagnostics::{structured_rejection_returns_plain_translation, structured_success_shows_analysis}` now submits real page input to deterministic server fixture; Chrome execution still pending |
| 10 | Games | **New** route smoke; `recent_routes::crossword_click_to_place_and_reset` | Add Cloud Match end-to-end play/finish path |
| 11 | Random | **New** Home tile transition | Follow word details, reload behavior and audio |
| 12 | Settings | **New** route smoke; `recent_routes::settings_font_and_theme_are_independent` | Persist all key preference panels and test failure messaging |
| 13 | About | **New** route smoke | Verify attribution links and keyboard navigation |

## New progression contract

`learner_journeys::practice_progress_and_configuration_journey` proves that: future step circles are disabled; selecting Advanced opens Exercises; the six exercise tiles are available; tile selection reflects `aria-pressed` and `data-selected`; Continue first opens **Configure** (where the selection grid is absent), then Vocabulary; previous progress circles navigate back without losing selected exercises. This is the acceptance contract for the R202 navigation redesign.

## Injecting a loader failure

**Do not introduce `/#/?fail`** merely to make production fail. The current harness can reproduce real failures with Chrome DevTools Protocol `Network.setBlockedURLs`, as implemented in `tests/suites/database.rs`. This works in both debug and release builds without adding user-facing failure switches or an injection branch to application code. For deterministic error types unreachable through network blocking, use fixture server responses or a test-only CDP request interception. If a future Rust-only fault seam becomes necessary, gate it under `debug_assertions`, require a precise fault name, and verify that release builds ignore it. Never interpret a production-facing URL parameter as authorization to break startup.

## Determinism and isolation

No sleep-based synchronization: wait on `data-*` hooks or page state. Use fixture responses instead of live AI/audio/image service availability. Check a real outcome, not only that a page opened. Keep each scenario in a separate function under `tests/suites/`, and register it once in `suites/mod.rs`; the harness isolates storage and captures browser diagnostics. Run the root suite with:

```powershell
cargo make setup
cargo test -p zipeek-e2e --test smoke --locked -- --test-threads=1
```

The older MCQ/Matching/Mixed E2E selectors may still predate the R202 stepper and must be migrated before the entire root suite can be considered passing. The new paths extend coverage; they do not claim to complete all 13 journeys.

## R204 diagnostics and setup migration

The managed AI fixture now supports `translator-rejected-analysis` (HTTP 400
with a model context-limit reason, followed by a successful plain translation)
and `translator-valid-analysis` (valid structured JSON). The dedicated Chrome
cases verify the real `/translate-sentence` route, result rendering and
presence/absence of analysis. `tests/suites/setup.rs` now tests the actual
interactive progress rail, Choose/Configure substeps, selection guards,
Fixed 5 default, change of session length, and responsive screenshots.
Old removed `data-activity-picker` / `data-exclusions-reset` form selectors
were deleted from that suite. The other legacy practice suites have not all
been migrated or executed; source-contract tests alone are insufficient.

## R219 HSK selection load and Summary navigation

`learner_journeys::rapid_hsk_selection_remains_responsive` clicks several HSK levels in a short burst, checks that the reducer retains the correct final selections, proceeds directly to Summary, and verifies that Back and Start remain together while the page-owned debounced vocabulary preflight resolves. This scenario is registered in the managed Chrome suite but has not been executed in this packaging environment.

## R518 Word Sets UI acceptance

The Word Sets management sidebar shows all existing sets directly (no dropdown). Switching between sets updates the vocabulary list and clears any search query. Search matches simplified Hanzi, pinyin or English gloss case-insensitively. Rename is behind an explicit, keyboard-operable Rename disclosure; deleting a set requires a second, clearly labelled action. The document owns vertical scrolling on desktop and mobile, including long word lists; no nested word-list scrollbar. Export/import remains available under Data tools. R518 remembers the selected set as the default destination for Dictionary Add/Remove. Practice remains independently configured.
