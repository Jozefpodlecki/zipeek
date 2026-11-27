## R706 WIP - Root script removal and Rust task ownership

Removed the root `scripts/` directory (legacy Node.js and Python helpers), migrated active Makefile tasks to `zipeek-cli` or Rust tests, and repaired the geography catalog include path. Source archive pack/unpack now invoke the existing release CLI directly without hidden Cargo builds. The browser vendor asset build scripts under `crates/zipeek-web/scripts/` are separate required frontend build inputs and remain unchanged. Local Cargo/Windows acceptance is pending.

## R706 WIP | Explicit desktop build boundaries | 2026-11-27

- `scripts/run-existing-cli.cjs` only invokes existing `zipeek-cli` binaries. It no longer invokes `cargo build` when the CLI or service executables are missing. Missing binaries fail immediately with the explicit build task to run.
- Added `debug-cli-build`, and made `desktop-sidecars-rebuild` depend on both service and CLI build tasks. `desktop-sidecars` remains staging-only; `desktop-dev` does not silently compile missing sidecar prerequisites.
- `db-compress` continues to display human-readable MiB plus exact bytes; unchanged-content sidecars are not rewritten.

## R706 WIP | Compressed size reporting and Windows sidecar staging | 2026-11-27

- `zipeek-cli db-compress` reports human-readable MiB alongside the exact byte count in the existing completion message. User-generated candidate compressed to 44,129,059 bytes (42.09 MiB).
- Desktop sidecar staging compares existing files and avoids rewriting identical executable, DLL and native dictionary bytes. Staging copy errors now identify the affected file and suggest closing running Zipeek processes on Windows. Added a no-op staging regression test.
- Documented Windows Tauri `os error 32` recovery in the native-services architecture. The reported failure occurs in the Tauri build script; a lock holder cannot be identified from the log, so this change does not claim to resolve every such failure.
- Full source archive structurally verified; Rust formatting, compilation, tests and Windows desktop launch unverified in the assistant environment.

## R706 WIP | Remove obsolete Node database workflow (2026-11-27)

- Deleted `scripts/db-workflow.cjs`: generation, compression, installation and validation are owned by `zipeek-cli`, not a parallel Node implementation.
- Migrated `cargo make db-browse-check` to native `zipeek-cli db-browse-check`; the command validates the installed bundle and both new dictionary search tables.
- Added command parser and Makefile ownership assertions so the removed Node entry point cannot silently return.
- User-provided Windows generator run successfully imported 120,970 words and verified a 152,842,240-byte candidate after read-only reopening. Compression, installation and browser-WASM FTS execution are separate acceptance gates.
- ZIP/source validation only in assistant environment; Rust compilation and execution not available.

## R706 WIP | CLI release discovery and archive fixture repair | 2026-11-27

- Corrected numbered ZIP parsing to recognize `-WIP` between the release description and date, as well as the existing `WIP-` prefix, without weakening date/time validation.
- Repaired the archive restore regression fixture by creating the local practice-assets directory before writing its obsolete-file sentinel. The restore algorithm and preservation rules are unchanged.
- Added a CLI regression assertion tying the database Makefile tasks to `zipeek-cli` rather than a removed `zipeek-db-gen` executable. The existing `Makefile.toml` commands were confirmed aligned with the CLI route and required no further change.
- User-reported baseline: 88 CLI tests passed and 3 failed. Rust tests, formatting, Clippy and the Windows archive restoration test have not been executed in this environment.

R706 WIP generator boundary correction (source-only): `zipeek-db-gen` is library-only, with its former Clap commands hosted by `zipeek-cli db-gen`; generation workflow now uses that executable rather than a separate generator binary. Native session and transaction code moved under `zipeek-db`, eliminating the direct DuckDB driver dependency from the generator. Updated locked dependency graph, CLI regression tests, generator docs and Cargo semver guidance. No dictionary or personal database changes; Rust compilation, tests, formatting, CLI execution and database regeneration unverified here.

## R706 WIP: Artifact report and snapshot query ownership (2026-11-27)

- Moved the artifact attribution notice from `zipeek-db-gen/src/artifact/` to `docs/dictionary-data/schema/artifact-attribution.md`; generated `ATTRIBUTION.md` content remains byte-identical.
- Replaced the free artifact report writer with `ArtifactReport::new(ArtifactReportOptions)` and replaced the handwritten UTC calendar conversion with `chrono::Utc`. Added the already locked chrono 0.4.45 as a direct dependency and updated the crate dependency list in `Cargo.lock`.
- Centralized all 25 logical-snapshot SQL statements in formatted `artifact/queries.rs`, without changing projections, row ordering or empty-result policy.
- Removed the duplicate `DatabaseSession` import in `artifact/writer.rs` found in the previous checkpoint.
- No dictionary file or generated manifest has been regenerated; source changes and archive integrity are checked, Cargo formatting/tests/Clippy remain unverified.

## R706 WIP: Generator database boundary and transaction ownership

The generator now reuses the existing owned `zipeek_db::native::GeneratorDatabase` and introduces a scoped `DatabaseSession` for simple SQL execution, query-context errors, and RAII transactions. It does not create a duplicate owning connection type. Artifact SQL's free `execute` helper is removed; FTS and artifact writer operations use the session, while copy operations use the transaction directly. Dictionary import and cached classification updates are routed through the same transaction boundary. Specialized prepared statements and vector reads retain explicit raw DuckDB access inside the data-access implementation. The `database/` directory has its required `mod.rs`.

No DuckDB version, table definition, data semantics, packaged dictionary binary, or personal data is changed. The pinned DuckDB-WASM version remains `1.33.1-dev114.0`. User-supplied validation previously showed 78 library tests passing; the four artifact integration failures were attributed to old fixture pins, and this checkpoint has not been compiled or formatted in the assistant environment. Run `cargo fmt --all`, locked tests, and Clippy locally before acceptance.

## R706 WIP - Generator fixtures aligned with approved DuckDB-WASM pin (2026-11-27)

- Updated three test fixture input manifests from obsolete DuckDB-WASM 1.32.0 to the approved exact `@duckdb/duckdb-wasm@1.33.1-dev114.0` pin.
- Updated the policy-override regression test to start with the authoritative pin, then assert rejection of `latest`. Previous value no longer matched test input, so it could pass for the wrong reason.
- Preserved the generator policy, npm pin, Cargo.lock, production inputs and binary database artifacts.
- User's Windows run: 78/78 generator unit tests passed; 4/7 artifact integration tests failed at preflight because of stale fixture manifests. These updates address that preflight mismatch; integration tests and build were not rerun here.

## R706 WIP - Generator transaction, publication and streaming integrity refactor (2026-11-27)

- Introduced a single RAII `Transaction` lifecycle in `zipeek-db-gen/src/transaction.rs` and migrated dictionary insertion, classification cache application and artifact table copying away from manually issued SQL transaction strings. Explicit commit and rollback paths preserve original failures and surface rollback failures instead of silently swallowing them; dropped unfinished transactions roll back.
- Introduced owned `StagedArtifact` publication in `artifact/files.rs`. A failed generator operation automatically deletes its incomplete artifact staging directory, while successful publication preserves the output without replacing an existing artifact. Simplified recursive staging cleanup.
- Changed generated DuckDB artifact checksum reporting to buffered streaming SHA-256, avoiding loading the full `hanzi.duckdb` file into memory. The manifest checksum fields and byte representation remain unchanged. Also changed in-memory SHA-256 encoding to use the existing `hex` crate without per-byte formatting allocations.
- Added focused regression tests for commit, uncommitted rollback, failed writes, staged publication and cleanup, and streamed/in-memory checksum equivalence.
- Scope is generator internals only: no dictionary schema, word identity, FTS policy, personal data, or UI behavior changes. No dependencies changed. The previously reported relationship-import test failure is still pending execution of the revised tests.
- Source archive and static structural checks only; Cargo, rustfmt, native DuckDB generation, and benchmarks were not available in this environment. A regenerated binary dictionary is not included.

## R706 WIP - Dictionary artifact generator cleanup (2026-11-27)

- Refactored artifact writer into generation orchestration (`writer.rs`), FTS lifecycle and read-only checks (`fts.rs`), and small SQL execution/path quoting helpers (`sql.rs`). Existing schema, index configuration and generated artifact format remain unchanged.
- Replaced reference-dropping FTS count code with the existing scalar count reader, addressing the reported `dropping_references` warning.
- Reworked the relationship import regression to verify exact ambiguous, missing and resolved endpoints using row-count queries; previous fixture's `first_string` helper could fail with an opaque `null result`.
- Recorded future Practice controls/feedback UX requests in TASKS; no Practice UI changes in this checkpoint.
- Static source and archive checks only. Cargo/DuckDB and browser tests not available here. The user-provided prior generator run reported 69/70 tests passing; the updated test has not been executed. No generated DuckDB artifact was replaced.

## R706 WIP: dictionary full-text index generation

- Extend dictionary generation with per-word English search documents incorporating lexical senses and normalized search keys for simplified/traditional Hanzi, tone-folded pinyin and numeric pinyin.
- Build FTS on `word_search_documents`, preserving stable dictionary word IDs and user personal data.
- Include new tables in copy and logical snapshot verification; test search-key normalization, sense-derived documents and artifact coverage.
- Source ZIP only: generator execution, a new `hanzi.duckdb`, manifest replacement, native/WASM runtime checks and benchmarks are **not completed** because Cargo/DuckDB are unavailable in this environment. Existing static assets are intentionally untouched.

## R706 WIP — HSK Profile defaults and dictionary search relevance

- Added reusable HSK standard/level selector shared between Profile and Practice.
- Added auto-persisted Profile HSK preference; new dictionary Practice setup copies those bands once. Saved session snapshots and Practice-only changes remain independent.
- Mixed HSK keeps distinct 2.0 and 3.0 level groups. Each visible level button uses only its number.
- Excluded-word autocomplete ranks exact English meanings and word-boundary matches before incidental substrings, and uses a compact results viewport.
- No modification to scoring policy, exercise grading, native services, or unpack backups.
- Source-only validation; Cargo/WASM not run in this environment.

## R706 WIP — Profile exclusions reducer and WASM build repair

- Replaced the monolithic Excluded Words component with an explicit `excluded_words/mod.rs` and `excluded_words/state.rs` reducer module.
- Coupled search input, selected suggestion, dictionary loading, excluded IDs and save errors under one Yew reducer; stale dictionary responses are ignored.
- Exclusion add/remove operations now share one persistence boundary; exact Word IDs, dictionary autocomplete and keyboard selection remain intact.
- Fixed missing `yew_icons::Icon` import in Profile drawer and removed unsupported `LUCIDE_BAN` usage.
- Added reducer regression tests for stale query responses, exclusion saves, and stale excluded-word loads.
- Complete source ZIP integrity and static reference checks verified. Rust/WASM compilation and browser tests not run.

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

### R706 CLI runtime on Windows

- `cargo make unpack` and `cargo make pack` now depend on `duckdb-env` so the pinned native DuckDB DLL is on `PATH` when the prebuilt `zipeek-cli.exe` starts. Previously Windows failed before CLI argument parsing with `0xC0000135` (`-1073741515`).
- The same prerequisite applies to the remaining Makefile tasks that directly invoke the native CLI, including database compress/install and sidecar staging. The CLI launcher now emits a specific diagnostic for that missing-DLL process exit.
- No nested Cargo build was introduced by the launcher; release archive unpacking continues to use the prebuilt executable.
