## R697 WIP: Debug workspace and SQL result tabs (2026-11-26)

- Stabilized Development diagnostics at a consistent maximum width and desktop minimum height, so switching Overview, DuckDB, SQL and OPFS does not shift the outer panel.
- Added Yew/Lucide icons for the diagnostic tabs, using the provided `crates/zipeek-web/public/duckdb.webp` image for DuckDB.
- Removed redundant SQL-terminal and SQL-query headings.
- Replaced the long flat SQL example selector with a collapsed, grouped query-example menu.
- Added bounded, closable SQL result tabs, each retaining its own query output or error. An execution opens a new result tab. Eight recent result tabs are retained; the oldest is evicted after that limit.
- Kept the SQL editor and results grid visible as a stable workspace, with CSV export per selected result.
- Preserved the debug-only route guard, single SELECT validation, 100-row output cap and existing OPFS read-only inspector.

Validation: static source assertions and archive-content checks only. Rust/WASM and browser testing pending.

# R696 WIP: Debug OPFS inspector and SQL scope

Archive: `zipeek-R696-WIP-debug-opfs-sql-inspector-20261126-213905.zip`. Packaging date: 2026-11-26 UTC. Baseline: corrected R695 full-source archive.

## Changes

- Added a separate OPFS tab in the existing debug-only diagnostics route. It reads root directory names and types through `navigator.storage.getDirectory()` and an asynchronous iterator, with bounded 500-entry output, alphabetical ordering, Refresh, and loading, empty and error states. No OPFS files are opened or modified.
- Removed the hardcoded `words` / `word_relations` / `information_schema.tables` / `pragma_database_size()` SQL allowlist so ordinary read-only SELECT queries may address additional dictionary relations. The validator still restricts statements to a single SELECT with a 2 KiB input cap, rejects known external I/O and administrative expressions, and wraps results in a 100-row limit.
- Preserved existing Yew icons, browser-local data boundaries and debug-only route guard.
- No new dependencies or lockfile changes.

## Verification

Static source assertions, ZIP integrity and Unicode/case-insensitive collision checks performed. Rust compilation, actual DuckDB SQL and browser OPFS enumeration were not run in this environment. Run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`, `cargo test --locked -p zipeek-web` and check both app base paths.

---

# R695 WIP: Debug SQL terminal UX and relationship query correction

Packaging date: 2026-11-26 UTC. Source baseline: user-provided `zipeek-source-0.1.0-20261126-203949.zip`.

## Changes

- Enlarged the debug SQL editor to 12 rows with a 18rem minimum height, explicit resize support, and a wider SQL panel.
- Removed debug-only disclaimers and the unrelated AI testing note from the debug screen.
- Replaced the Run SELECT label with an accessible Yew/Lucide play icon and an in-flight spinner.
- Allowed one optional trailing SQL semicolon while still rejecting multiple statements and comments.
- Enabled read-only queries against `word_relations` and provided an example that groups `source_resolution` and `target_resolution`, matching the user's failing query.
- Reduced redundant query-error instructions while preserving the actual error text.
- Preserved the debug-build route guard, bounded 100-row results, and existing CSV export.

## Verification

Source assertions and ZIP integrity checks executed in this environment. Cargo, rustfmt, WASM compilation, DuckDB execution and browser interaction were unavailable; this remains an unverified source candidate. On Windows run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`, `cargo test --locked -p zipeek-web`, then retry the relationship-resolution SQL in Debug > SQL terminal.

---

# Settings Reset app source snapshot

Archive: `zipeek-source-0.1.0-20261126-203236.zip`. This is an unnumbered full-source snapshot, not an accepted installer release. Existing concurrent AI/speech workspace changes and their notes below are preserved; only the stated reset/integration checks were performed for this task.

Settings now has Reset app with an explicit destructive confirmation and **Clear all data and start over**. An isolated reload removes Zipeek-owned local/session storage, personal OPFS files and native personal DuckDB/WAL before returning to fresh landing/onboarding. Dictionary files, models, exported backups and unrelated origin data are retained. A dedicated confirmed native IPC command closes database connections before bounded deletion. Other open app sessions block reset, and partial errors remain visible with Retry. No dependencies were added or changed for reset.

Passed: locked offline WASM check, all 20 native library tests including reset fixtures, one ownership-policy test, and all eight managed Chrome reset scenarios at `/` and `/zipeek/` (confirmation/cancellation, deletion/onboarding, direct URL protection, another active tab and native failure/retry). Videos, screenshots and assertions were validated. Native filesystem tests used temporary fixtures; actual reset in an installed Tauri WebView was not run. The Pages harness initial URL now includes its required trailing slash; the test's reinjected silent-speech flag is accounted for without weakening user-data deletion checks.

Strict Clippy is not clean: dependency linting stops in existing core/pedagogy/database code; focused `--no-deps` runs report 96 web and four native lint errors outside the reset implementation. No reset-module lint references were reported. These unrelated lint changes were not folded into reset. Formatting and focused runtime tests passed.

Documentation validation still reports unrelated long dash punctuation in the existing AI-server document and the existing raw JSON key-order index comparison, plus frozen-history link warnings. A separate semantic index check is used after regeneration. Full CLI/workspace/model suites were not rerun for reset; earlier documented CLI failures and current AI/speech warnings were not repaired in this scope. The source ZIP excludes generated dictionaries, sidecars, vendor/model assets and build/evidence output; use the existing setup and desktop commands to prepare them.

The separate desktop-dev speed/log review made no build-system changes. Recommended follow-up is unchanged-artifact staging, no-op Docs/asset writes and quieter child-tool success logs. Removing unused direct server dependencies from the shell requires explicit dependency approval. No speedup has been benchmarked.

---

# R694 WIP: AI provider Settings interaction redesign

Packaging date: 2026-11-26 UTC. This source handoff updates the R693 AI provider settings presentation only; it does not replace the Llama.cpp inference backend or LM Studio client.

## Implemented

- Present Local Llama and LM Studio as restrained, keyboard-accessible provider cards. The selected provider uses a subtle violet border/background, with **no checkmark overlay** or duplicate connected badge.
- Display only fields applicable to the chosen provider; keep optional API-key controls collapsed, and move manual model entry into progressive disclosure when discovery returns models.
- Show discovery progress in the discovery control and display inline, retryable errors without a full-width alert panel.
- Give the real model-test action an explicit in-progress spinner and dismiss action; show a compact outcome after completion, with detailed request/response inspection collapsed.
- Keep Save and Test together at the bottom of a stable-height settings panel. Preserve the Tauri-only Local AI option and separate desktop process controls under Settings > Services.
- Preserve an existing tab-memory API key when saving an unchanged key field; avoid treating an empty replacement field as removal.
- Add reducer regressions for stale model discovery, discovery failure and manual model choice.

## Verification

- Source structure and focused static assertions checked in this workspace. Cargo, rustfmt, WASM compile, native Tauri WebView and actual Llama/LM Studio inference were not run because the Rust toolchain is unavailable.
- Verify locally: `cargo fmt --all -- --check`; `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`; `cargo test --locked -p zipeek-web`.
- In Tauri, switch providers, discover LM Studio models, test both healthy and failing endpoints, confirm the result is inline, verify no selected-provider check icons, and ensure Start/Stop still appears only in Settings > Services. Test small windows and keyboard focus.
- R693 limitations remain: LM Studio WebView CORS, local GGUF acceptance, the optional GGUF picker and Tauri HTTP proxy are still outstanding.

---

# R693 WIP: Local Llama.cpp and LM Studio provider selection

Packaging date: 2026-11-26 UTC. Full-source handoff archive: `zipeek-R693-WIP-local-llama-lmstudio-providers-20261126-154639.zip` (the actual archive name contains its packaging timestamp).

## Changes

- Replaced the production `zipeek-ai-server` fixture-only entry point with actual GGUF-backed llama.cpp inference, using the existing `llama-cpp-2` dependency. The executable starts only with a valid model and loopback bind.
- Added a bounded streaming token generation backend, context and output controls, model-native chat template formatting, temperature sampling, request cancellation, and error propagation through the existing generation trait.
- Retained deterministic fixture backend for `#[cfg(test)]` HTTP unit tests only; executable fixture mode no longer exists. Updated its process lifecycle test to check missing-model failure.
- Changed the Tauri sidecar to start the real service with `--bind` and granted AI model cold-loading the existing extended startup allowance.
- Replaced the AI test-server label with Local AI in the desktop Services panel; Start/Stop remain there rather than being duplicated.
- Added provider selection in Yew Settings > AI: Zipeek Local AI on port 1430 (Tauri only) versus editable OpenAI-compatible/LM Studio endpoint, initially `http://192.168.1.148:1234/v1/`.
- Added model discovery through bounded `/v1/models` requests and a selectable model list; preserved manual model ID entry.
- Provider switching resets stale diagnostics and clears tab-memory API keys when switching to local inference. No new Rust dependencies or lockfile modifications.
- Added model-installation and runtime documentation in `docs/architecture/modules/ai-server.md` and `models/ai/README.md`.

## Known constraints

- The local GGUF checkpoint must be supplied by the user and is not bundled. The default local inference GPU-layer count is zero unless `ZIPEEK_AI_GPU_LAYERS` is configured. Native llama.cpp startup and CUDA execution have not been tested here.
- LM Studio requests from Yew still use the existing WASM HTTP client and therefore depend on CORS permission from the configured endpoint. A native Tauri HTTP proxy and arbitrary-model file picker are deferred.
- The production executable no longer supports the old `--fixture` CLI mode. Tests/e2e scripts that launch that mode need updating to use their in-process or explicit test-only provider; this is an intentional development-time breaking change.
- The local model endpoint does not natively enforce JSON Schema; the existing translation fallback remains necessary.
- `cargo make desktop-sidecars` can reuse old binaries. Use `cargo make desktop-sidecars-rebuild` after installing this WIP. The model should not be added to the source ZIP.

## Verification

- Source inspection and static contract checks performed in this environment. There is no Rust toolchain, so Cargo, rustfmt, native WebView, CUDA and real GGUF inference were **not run**. The handoff remains an unverified WIP candidate.
- Windows acceptance: `cargo fmt --all -- --check`; `cargo check --locked -p zipeek-ai-server -p zipeek-app`; `cargo test --locked -p zipeek-ai-server -p zipeek-ai-client`; `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`; rebuild sidecars; manually test both provider modes and Service start/stop/health.
