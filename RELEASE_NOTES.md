## R706 WIP — Real image generation readiness

- Fixed the default image backend silently selecting SVG placeholder images. Default startup now requests SDXL; missing model files cause explicit startup failure instead of a misleading successful image preview.
- Settings health checks distinguish an inference-ready SDXL backend from a reachable preview-only placeholder. Browser generation checks backend readiness before submitting, and desktop Tauri image commands reject preview-only generation.
- Improved image sidecar startup diagnostics and documented model installation and CUDA requirements. Added regression coverage for backend selection and preview rejection.
- Does not bundle model weights or guarantee GPU availability. Existing explicitly configured preview-only backend remains useful for protocol tests.

R706 WIP UI refinement (2026-11-27): Replaced the exact-Hanzi exclusion form with a bounded dictionary autocomplete for Hanzi, pinyin and English. Suggestions identify individual Word IDs and show Chinese, reading and meaning, with keyboard navigation and direct add/remove. Profile and Word Sets remove redundant instructional labels; drawer configuration uses the existing navigation icons in a compact grid. Saved-session recovery uses a single heading, finite progress bar, Play/Resume and icon Delete. The global saved-session notice no longer duplicates the Practice setup recovery screen. The header dragon image is contained at a compact width instead of being cropped across the header. No schema or scoring changes. Build and interaction tests not run.

R706 WIP build correction: Browse word-card saved-word navigation now links to Profile instead of the removed WordSets route.

# R706 WIP — Profile configuration hub (2026-11-27)

- Added `#/profile`, replacing the standalone Word Sets drawer route with a user-icon Profile entry. Profile exposes Word Sets, Excluded Words, and Drawer Menu tabs; no avatar or account data is introduced.
- Reused the existing Word Sets provider, management panel, vocabulary list and persistence; the create button is now a `+` icon with accessible name and tooltip.
- Added an excluded-word manager that reads existing saved Word IDs, resolves dictionary labels, supports searching, exclusion by exact Hanzi spelling, and one-click removal. Uses the same exclusion storage key and respects the dictionary database boundary.
- Added configurable drawer visibility using the existing Preferences document and centralized navigation registry, with same-tab and cross-tab refresh. Profile and Settings are always available, and existing preferences remain default-visible.
- Added regression tests for persistent visibility and essential navigation. No new dependencies, user profiles, account system or data migrations.
- Source/ZIP integrity only. Rust/WASM build and browser interaction acceptance remain unverified; R706 is WIP, not an accepted major release. Existing R705 pedagogical and Endless outstanding work remains open.

# R705 WIP - Typed Tauri bridge and native service adapters (2026-11-27)

Complete source handoff `zipeek-R705-WIP-typed-tauri-transport-20261127-190200.zip`: one frontend Tauri transport under `services/tauri/mod.rs` with explicit `error.rs`, `invoke.rs`, `commands.rs` and `events.rs`; typed error classifications, cached successful bridge initialization, typed invoke/command selection and listener ownership. Migrated native database, history, service controls, AI and image generation; centralized Tauri detection. Removed redundant IPC error module. Added command-contract tests and kept the existing native Rust command names, data models, per-service fallbacks and no-`.unpack-backups` pack/unpack contract unchanged.

Source review and archive verification only. Rust compilation, WASM tests and Tauri runtime acceptance not run; user should check the WASM build and desktop service interactions. No dependencies changed. The durable KnowledgeLedger processor and lossless unlimited Endless session persistence remain open.

R705 WIP - 2026-11-27 - Compact Practice exercises, Character Builder dragging and Mandarin playback recovery

- Removed redundant Practice exercise-position and vocabulary round/mistake counters from the learner-facing workspace; assessment counters and saved outcomes are retained internally.
- Removed the extra vocabulary card wrapper in resumed sessions; shared ExerciseSurface is the sole card and omits the instruction chrome for Character Builder's self-explanatory task.
- Added slot-aware Character Builder drag placement and reordering using the same answer-selection reducer, with native HTML drag/drop for mouse and pointer-drop handling for touch. Kept click-based add/remove and keyboard-operable buttons. Added pure placement regression tests.
- Added centralized native Mandarin playback fallback to an explicitly Mandarin-capable browser voice with 60-second native retry throttling, request-identity cancellation guards, developer logging, and one-time non-blocking notifications through the existing ToastProvider. No generic non-Mandarin voice is used as silent fallback.
- No Knowledge Points scoring rules, no-.unpack-backups behavior, dependency versions, or session persistence format changed.
- Source/ZIP integrity checks only; Rust/WASM compilation and interactive mouse/touch browser acceptance not run. This remains an R705 WIP source candidate.

R705 WIP - 2026-11-27 - Desktop storage diagnostics and compact dragon status

- Reduced the dragon progress message to a small anchored status panel with click-outside and Escape dismissal, preserving the existing unassessed status until verified knowledge projections exist.
- Reused Tauri NativeDatabase's actual app-local personal DB path for native diagnostics. Added path, exists/size metadata, refresh, and platform file-manager reveal commands; retained browser OPFS inspection.
- Kept the existing Word Sets navigation route until the planned Profile configuration page is implemented. The existing navigation registry remains centralized and can later support configurable visibility.
- Preserved all R705 vocabulary evidence work, scoring contracts, session semantics, and no-.unpack-backups behavior.
- Source-level checks and ZIP integrity only; Rust, WASM and desktop UI builds not executed in this environment.

## R705 WIP candidate: Target-level vocabulary observations (2026-11-27)

- Implemented per-target Type Pinyin reading assessment with attempts, mistakes and exposure of the answer in the hint placeholder recorded as assisted evidence. This is a reading-input capability, not automatically Knowledge Points.
- Added pair-specific error provenance in Word Mosaics, with individual target outcomes. A wrong match marks both participating pairs as corrected/partial; other pairs can retain clean independent matches. The aggregate result is partial if any mistakes occurred.
- Carried hint/assistance and partial item verdicts to the multi-round aggregate instead of treating a corrected or assisted completion as an unqualified clean success.
- Raised the item evidence limit to 160 and the per-observation bound to 64 KiB to retain 20 rounds of 8 word matches without silently dropping evidence. The 1,000-step Endless ceiling is unchanged.
- Preserved exact existing Knowledge Points rules, one-click delete and temporary unpack rollback, with no `.unpack-backups`.
- Unverified R705 release candidate: this milestone does not satisfy the approved major-release 20-qualifying-Rust-path gate; Rust and WASM builds and browser tests were unavailable in the packaging environment.

R704 WIP (2026-11-27): Corrected a duplicate ExerciseReviewProps/ExerciseReview declaration in pages/practice/feedback.rs that prevented the wasm32 frontend build (E0428/E0119/E0592/E0277). Preserved the existing shared review markup, props, and behavior by removing only the identical second copy. Build verification remains pending.

## R704 WIP: Library warnings and durable observation inbox boundaries (2026-11-27)

- Kept R704 WIP. Based on the user-formatted 17:18 source archive, with no dependency or scoring-policy changes.
- Normalized explicit unused and unused-import crate allowances across all 22 library roots. The native pedagogy worker now excludes its wasm-only implementation modules instead of compiling dead worker code.
- Refactored consented Practice observations into separate event model, persisted inbox storage, orchestration and tests. Added an ordered inbox reader for future processing, strict step and timestamp checks and evaluator evidence validation at the persistence boundary.
- Preserved existing direct delete, no persistent `.unpack-backups`, Endless selection, construction-only observational semantics and established Knowledge Points thresholds.
- No `KnowledgeLedger` mastery projection is claimed: authorized target manifests and transactionally persisted processed IDs are still required. The 1,000-step Endless limit also remains.
- Archive/source checks only. Rust compile, WASM build, native tests, browser acceptance and Windows pack/unpack benchmarks were not run in this environment.

## R704 WIP | Character-sequence assessment evidence contract | 2026-11-27

- Moved the shared evaluator verdict, assessed outcome, and item-evidence types into an independent assessment module in `practice_session_store`.
- Character Builder now reports the actual Chinese target and a distinct character-sequence-construction capability, together with correct/partial/incorrect verdict and full attempt and mistake counts. A correct answer after mistakes remains a partial item-level performance instead of independent clean success.
- Vocabulary rounds retain item-level evidence across rounds, then send it through the existing assessed outcome and idempotent Practice-observation journal. Type Pinyin and Word Mosaics keep their genuine aggregate results without fabricated item targets.
- The saved observation carries the step activation timestamp separately from completion; this is not a verified first-pixel presentation-time measurement.
- Increased the per-observation storage bound to accommodate a full 20-round item-evidence record. Added validation and serialization regression coverage.
- Construction evidence remains observational and does **not** award Hanzi recognition, handwriting, or other Knowledge Points. The durable validated KnowledgeLedger processor, true presentation timing, and item-level coverage for other exercise adapters still require implementation.
- No scoring-policy, dependency, release-number, exercise UX, or `.unpack-backups` changes. Rust/WASM compilation and browser acceptance were not run in this environment.

## R704 WIP: Cross-exercise Endless setup and per-step observations (2026-11-27)

Source handoff: `zipeek-R704-WIP-endless-setup-observations-20261127-164816.zip` (complete source archive; integrity checked; Rust compilation and browser checks not run).

- Practice setup exposes Endless for single or mixed selections of every registered resumable exercise, with fixed 5/10/20 choices. Translation-only setup now shares the same `SessionLength` launch decision; its 20-sentence choice is supported.
- Added opt-in step-keyed observation recording immediately after session progress persistence. This is deduplicated, repaired on session restoration, and erased alongside saved Practice history. It records assessed and unassessed outcomes but does not award Hanzi/vocabulary mastery without qualifying target evidence. Recording remains off by default.
- Migrated generated Sentence Translation and the Sentence Builder non-retry review path to the shared `ExerciseReview` component without altering scoring or retry rules.
- Added reducer and exercise-registry tests for the new mode availability and translation count synchronization, plus an observation encoding test. Browser storage, WASM compile and runtime behavior have not been verified here.
- This is **R704 WIP**, not a new numbered release. The existing 1,000-step Endless ceiling, lossless chunked finalization, target-level KnowledgeLedger attribution, native MSVC link conflict, and browser acceptance remain unresolved. No dependencies or pack/unpack behavior changed. `.unpack-backups` remains unused.

R704 WIP (2026-11-27): Working source now exposes Endless for all registered exercises, uses session length as the launch authority, records consented step outcomes in a deduplicated observation journal, and migrates generated translation and builder reviews into `ExerciseReview`. These are unverified source changes. The current 1,000-step Endless limit still exists; bounded chunked history and qualified knowledge-target updates remain open. Rust/WASM/browser acceptance is pending on the user's toolchain.


### R705 WIP: Character Builder drag compilation and target safety (2026-11-27)

- Fixed Yew closure ownership in Character Builder: use separate reducer and selection handles for click removal and slot drops; share the drag-over callback by cloning it before markup consumes it. Addresses the three user-reported E0382 errors.
- Kept empty answer slots valid HTML drop targets while preventing meaningless click/keyboard actions on empty slots; suppresses unintended click after a completed native drop.
- Corrected shared ExerciseSurface conditional instruction markup and kept the empty-instruction path quiet.
- Extended deterministic tile-position regression tests for dropping into empty slots, reordering, capacity, and removing tiles.
- Compilation, WASM acceptance and drag-device testing are not run in this environment.
